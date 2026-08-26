
use alloc::{alloc::alloc, vec::Vec};
use liblock::Mutex;

#[cfg(debug_assertions)]
use core::{fmt::Debug, iter::Take};
use core::{mem::ManuallyDrop, num::NonZero, sync::atomic::Ordering::AcqRel};

use super::{Extent, ScopedExtent, OwnedExtent};

use crate::{Alignment, address, extent_alloc::{ExtentMarker, MutableExtent, RawExtent, layout::{ExtentLayout, LayoutDescriptor}}};
use core::{marker::PhantomData, sync::atomic::Ordering::Release};

mod helpers;
use helpers::{Stats, ExtentAlloc, AllocMap};


mod batch;
pub use batch::*;


#[cfg(test)]
mod tests;



/// An implementation of an extent allocator that operates with
/// a predetermined page size, various sizes of allocated blocks,
/// and any number of memory blocks. Its primary purpose is to serve
/// as a physical memory allocator for the Constellation kernel, but
/// it can also be used for other purposes. For example, in file systems
///
/// > **IMPORTANT**: `ExtentAllocator` uses the heap,
/// therefore the heap must be initialized first
///
/// This implementation is optimized only for allocation and deallocation.
/// - Deallocation of parts of allocated frames is also supported!
///
/// > Note: The generic constant `ALIGN` must be a power of
/// two; violating this rule may result in undefined behavior
#[repr(C)]
#[allow(private_bounds)]
pub struct ExtentAllocator<const ALIGN: usize,
    Ext: Extent<ALIGN> + ExtentMarker<ALIGN, Ext> = RawExtent<ALIGN>,
    Lay: LayoutDescriptor<ALIGN> = ExtentLayout<ALIGN>>
{
    map: Mutex<ExtentAlloc<ALIGN>>,
    stats: Stats,
    _lay: PhantomData<Lay>,
    _ext: PhantomData<Ext>,
}


impl<const ALIGN: usize, Ext: Extent<ALIGN>, Lay: LayoutDescriptor<ALIGN>> ExtentAllocator<ALIGN, Ext, Lay> {

    /// Constructs a new uninitialized `ExtentAllocator`
    pub const fn uninit() -> Self {
        Self {
            map: Mutex::new(ExtentAlloc::uninit()),
            stats: Stats::uninit(),
            _lay: PhantomData,
            _ext: PhantomData,
        }
    }

    /// Initializes the `ExtentAllocator` by consuming an iterator that yields `TakenExtent`
    /// - `TakenExtent` is thin abstraction over `RawExtent` that
    /// indicates whether the extent is already used or still free
    ///
    /// # Safety
    /// This function itself is not unsafe, but if the iterator passed to it provides
    /// invalid data, the system may fall into undefined behavior. For this reason,
    /// the caller must guarantee that the passed extents are valid at least for the lifetime
    /// of the allocator and that they do not overlap with one another.
    ///
    /// # Errors
    /// This function does not manually check whether the passed extents are overlapping,
    /// the `InitError::OverlappingExtents` error is returned only if the underlying
    /// implementation is unable to process the extents due to its design.
    /// - The circumstances are undefined
    ///
    /// The `InitError::AlreadyInitialized` error is returned only if this function has already been called in this instance
    #[cold]
    #[inline(never)]
    pub unsafe fn initialize<I>(&self, iter: I) -> Result<(), InitError>
    where I: IntoIterator<Item = TakenExtent<ALIGN, Ext>> {

        let mut alloc = self.map.lock();

        if !alloc.free.map.is_empty() || !alloc.used.map.is_empty() {
            return Err(InitError::AlreadyInitialized)
        }


        let mut total_pages: u64 = 0;
        let mut used_pages: u64 = 0;


        for ext in iter.into_iter() {

            let (map, ext) = match ext {
                TakenExtent::Free(ext) => (&mut alloc.free, ext),
                TakenExtent::Used(ext) => {
                    used_pages = used_pages.saturating_add(ext.size().get());
                    (&mut alloc.used, ext)
                },
            };

            total_pages = total_pages.saturating_add(ext.size().get());


            if let Some(_) = map.map.insert(ext.address(), ext.size()) {
                //  no overlapping extents => addresses are unique
                Self::clear_inner(&mut alloc);
                return Err(InitError::OverlappingExtents)
            }

            let by_size = map.size_index.entry(ext.size()).or_default();
            let by_align = by_size.entry(ext.address().get_align()).or_default();
            if !by_align.insert(ext.address()) {
                //  no overlapping extents => addresses are unique
                Self::clear_inner(&mut alloc);
                return Err(InitError::OverlappingExtents)
            }


        }

        //  save statistics
        _ = self.stats.total().store(total_pages, Release);
        _ = self.stats.used().store(used_pages, Release);

        Ok(())

    }

    /// Removes all extents from the map
    #[inline(never)]
    pub(crate) fn clear_inner(map: &mut ExtentAlloc<ALIGN>) {
        map.free.map.clear();
        map.free.size_index.clear();
        map.used.map.clear();
        map.used.size_index.clear();
    }



    /// Allocates one contignous extent as described by the `layout`
    /// - The returned extent needs to be deallocated manually
    #[inline(never)]
    pub fn alloc_contignous(&self, layout: Lay) -> Option<ScopedExtent<'_, ALIGN, Ext>> {
        let mut alloc = self.map.lock();

        //  finds suitable extent for the allocation
        //  - suitable means at least as big as `layout` dictates + correct layout
        let suitable = find_suitable_in(layout.clone(), &alloc.free)?;

        //  removes an extent described by the `layout` from the free tree
        let allocated = remove_extent_from(suitable, layout, &mut alloc.free)
            .ok()?;

        //  adds the extent described by the `layout` to the used tree
        insert_extent_to(&allocated, &mut alloc.used);

        //  statistics
        _ = self.stats.used().fetch_add(allocated.size().get(), AcqRel);

        Some(unsafe { ScopedExtent::from_extent(Ext::new(allocated.address(), allocated.size())) })
    }


    /// Allocates any amount of extents to satisfy the requested `layout`
    /// - Returned extents must be deallocated manually
    #[inline(never)]
    pub fn alloc(&self, mut layout: Lay) -> Option<Vec<ScopedExtent<'_, ALIGN, Ext>>> {
        todo!();
    }



    /// Allocates one contignous extent as described by the `layout`
    /// - The returned extent is automatically deallocated when `drop`ped
    #[inline]
    pub fn allocate_contignous(&self, layout: Lay) -> Option<OwnedExtent<'_, ALIGN, Ext, Lay>> {
        self.alloc_contignous(layout)
        .map(|ext| {
            let ext = unsafe { ext.into_extent() };
            OwnedExtent::new(ext, &self)}
        )
    }


    /// Allocates any number of extents to satisfy the requested `layout`
    #[inline(never)]
    pub fn allocate(&self, layout: Lay) -> Option<Batch<'_, ALIGN, Ext, Lay>> {
        todo!();
    }


    /// Deallocates the extent
    #[allow(private_bounds)]
    #[inline(never)]
    pub fn deallocate<E: ExtentMarker<ALIGN, Ext> + Extent<ALIGN>>(&self, extent: E) -> Result<(), ()> {
        let mut alloc = self.map.lock();

        let raw = RawExtent::new(extent.address(), extent.size());
        //  prevent double free (OwnedExtent)
        _ = ManuallyDrop::new(extent);


        remove_exact_extent_from(raw.clone(), &mut alloc.used)?;
        insert_extent_to(&raw, &mut alloc.free);

        Ok(())
    }

    /// Deallocates a `part` of the `original` extent
    /// - Returns `Err` if `part` does not fit into `original` or if `original` cannot be found
    #[allow(private_bounds)]
    #[inline(never)]
    pub fn deallocate_part<E: ExtentMarker<ALIGN, Ext>>(&self, original: E, part: E) -> Result<(), ()> {
        todo!();
    }

    /// Deallocates a `part` of the `original` extent
    /// - Does not check whether `part` fits into `original`
    #[allow(private_bounds)]
    #[inline(never)]
    pub unsafe fn deallocate_part_unchecked<E: ExtentMarker<ALIGN, Ext>>(&self, original: E, part: E) -> Result<(), ()> {
        todo!();
    }

}


/// Indicates an error while initializing the `ExtentAllocator`
pub enum InitError {
    /// The allocator is already initialized
    AlreadyInitialized,
    /// the iterator has yielded overlapping extents
    OverlappingExtents,
}


/// Indicates whether the extent is free or used
#[derive(Clone)]
#[allow(private_bounds)]
pub enum TakenExtent<const ALIGN: usize, Ext: Extent<ALIGN> + ExtentMarker<ALIGN, Ext>> {
    Free(Ext),
    Used(Ext)
}

#[allow(private_bounds)]
impl<const ALIGN: usize, Ext: Extent<ALIGN> + ExtentMarker<ALIGN, Ext>> TakenExtent<ALIGN, Ext> {
    /// Returns a reference to the underlying extent
    fn regular(&self) -> &Ext {
        match self {
            Self::Free(ext) => ext,
            Self::Used(ext) => ext,
        }
    }
}

#[cfg(debug_assertions)]
impl<const ALIGN: usize, Ext: Extent<ALIGN>> Debug for TakenExtent<ALIGN, Ext> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TakenExtent::Free(ext) => write!(f, "TakenExtent::Free(addr: 0x{:x}, size: {})", *ext.address(), ext.size()),
            TakenExtent::Used(ext) => write!(f, "TakenExtent::Used(addr: 0x{:x}, size: {})", *ext.address(), ext.size()),
        }
    }
}

/// Finds a suitable extent in the given map to satisfy the given `layout`
///
/// The returned extent is the one deemed suitable (thus the one found in the map)
/// - The returned extent does not exactly fit the `layout`
fn find_suitable_in<const ALIGN: usize>(layout: impl LayoutDescriptor<ALIGN>, map: &AllocMap<ALIGN>) -> Option<MutableExtent<ALIGN>> {

    'by_size: for (size, by_align) in map.size_index.iter() {
        if *size < layout.size() { continue 'by_size }

        'by_align: for (align, addresses) in by_align.iter() {
            if *align < layout.align() { continue 'by_align }

            //  there shall not be any empty map
            debug_assert!(!addresses.is_empty());

            if let Some(addr) = addresses.iter().rev().next() {
                return Some(MutableExtent::new(*addr, *size))
            } else {
                continue
            }
        }
    }

    None    //  no suitable extent found
}

/// Removes the extent from the map
fn remove_exact_extent_from<const ALIGN: usize>(extent: RawExtent<ALIGN>, map: &mut AllocMap<ALIGN>) -> Result<(), ()> {

    let by_size = match map.size_index.get_mut(&extent.size()) {
        Some(by_size) => by_size,
        None => return Err(()),
    };

    //  remove the address from size_index
    //  - addresses are unique, so this would eventually have to happen anyway
    match by_size.get_mut(&(&extent.address().get_align())) {
        Some(by_align) => if by_align.remove(&extent.address()) {

            if by_align.is_empty() {
                _ = by_size.remove(&extent.size());
            }
        } else {
            return Err(())
        },
        None => return Err(()),
    };

    //  remove the extent from the map
    //  - addresses are unique, so this would eventually have to happen anyway
    match map.map.remove(&extent.address()) {
        Some(size) => {
            //  helps with debugging
            debug_assert!(size == extent.size());
            Ok(())
        },
        None => panic!("UNEXPECTED: address was not present in the map (map)"),
    }

}


/// Same as `remove_exact_extent_from()`, but this splits the extent based on
/// the `layout` requirements and inserts the remaining space into the map again
///
/// Since any state where this function is unable to perform its
/// task is considered unexpected and/or undefined, it will simply panic when such state is detected
///
/// # Parameters
/// - `extent` is the extent found in the passed `map` that satisfies `layout`
///   - It is thus needed to be properly aligned and at least the same size as dictated by the `layout`
fn remove_extent_from<const ALIGN: usize>(mut extent: MutableExtent<ALIGN>, layout: impl LayoutDescriptor<ALIGN>, map: &mut AllocMap<ALIGN>) -> Result<RawExtent<ALIGN>, ()> {

    remove_exact_extent_from(extent.clone().into_regular(), map)?;

    debug_assert!(extent.size() >= layout.size());

    match unsafe { extent.split_unchecked(layout.size()) } {
        Some(put_back) => {
            //  create new entries in size_index and map

            let put_back: RawExtent<ALIGN> = put_back.into_regular();

            //  DEfragmentation is undesired
            insert_extent_to(&put_back, map);
        },
        None => {   //  OK
            //  layout.size() == extent.size() => no entry to put back
            //  - entries from both size_index and map are already removed
        },
    }

    Ok(extent.into_regular())

}


/// Inserts the extent into the `map` and `size_index`
/// - Does not take care of fragmentation
///
/// Since any state where this function is unable to perform its
/// task is considered unexpected and/or undefined, it will simply panic when such state is detected
fn insert_extent_to_fragmented<const ALIGN: usize, Ext: Extent<ALIGN>>(ext: &Ext, map: &mut AllocMap<ALIGN>) {

    //  addresses are unique => panic if it already exists
    if let Some(_) = map.map.insert(ext.address(), ext.size()) {
        panic!("unable to insert into map (map)");
    }

    let by_align = map.size_index.entry(ext.size()).or_default();
    let addresses = by_align.entry(ext.address().get_align()).or_default();
    if !addresses.insert(ext.address()) {
        panic!("unable to insert address into set (size_index)");
    }
}



/// Inserts the extent into the given map and performs defragmentation routine if needed
///
/// Since any state where this function is unable to perform its
/// task is considered unexpected and/or undefined, it will simply panic when such state is detected
fn insert_extent_to<const ALIGN: usize, Ext: Extent<ALIGN>>(ext: &Ext, map: &mut AllocMap<ALIGN>) {
    let end_address = ext.end_address();

    match map.map.remove(&end_address) {
        Some(neigh_size) => {
            //  has neighbor => defragment

            {   //  remove from size_index
                let by_size = map.size_index.get_mut(&neigh_size)
                    .expect("failed to index size_index");
                let by_align = by_size.get_mut(&end_address.get_align())
                    .expect("failed to index size_index:by_align");

                if !by_align.remove(&end_address) {
                    panic!("size_index:addresses does not contain the address");
                }

                if by_align.is_empty() {
                    if let None = by_size.remove(&neigh_size) {
                        panic!("failed to remove the map from size_index");
                    }
                }
            }

            //  defragmentation complete, now insert
            let merged = Ext::new(ext.address(), neigh_size.saturating_add(neigh_size.get()));
            insert_extent_to_fragmented(&merged, map);

        },
        None => {
            //  no neighbor => no defragmentation
            insert_extent_to_fragmented(ext, map);
        }
    }
}
