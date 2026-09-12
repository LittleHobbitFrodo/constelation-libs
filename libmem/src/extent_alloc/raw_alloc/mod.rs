use alloc::collections::BTreeMap;

use crate::{AlignedNonNull, extent_alloc::{
    extent::{Extent, InternalExtent, MutableExtent, RemainingExtents, ScopedExtent, TakenExtent}, layout::LayoutDescriptor,
}};
use super::helpers::AllocMap;
use core::{hint::cold_path, num::NonZero};
use crate::cold_panic;


//use functions::{find_suitable_in, extract_extent_from, insert_extent_to};

/// The raw extent allocator internally used by the `ExtentAllocator` and `CachedExtentAllocator`
#[repr(C)]
pub struct RawExtentAlloc<const ALIGN: usize> {
    free: AllocMap<ALIGN>,
    used: BTreeMap<AlignedNonNull<NonZero<u64>, ALIGN>, NonZero<u64>>
    //used: AllocMap<ALIGN>,
}

impl<const ALIGN: usize> RawExtentAlloc<ALIGN> {
    pub const fn uninit() -> Self {
        Self { used: /*AllocMap::uninit()*/ BTreeMap::new(), free: AllocMap::uninit() }
    }
}


impl<const ALIGN: usize> RawExtentAlloc<ALIGN> {

    /// Returns a reference to the free map
    #[inline(always)]
    pub(crate) fn free_map(&self) -> &AllocMap<ALIGN> { &self.free }

    /// Returns a mutable reference to the free map
    #[inline(always)]
    pub(crate) unsafe fn free_map_mut(&mut self) -> &mut AllocMap<ALIGN> { &mut self.free }


    /// Returns a reference to the used map
    #[inline(always)]
    pub(crate) fn used_map(&self) -> &BTreeMap<AlignedNonNull<NonZero<u64>, ALIGN>, NonZero<u64>> {
        &self.used
    }
    //pub(crate) fn used_map(&self) -> &AllocMap<ALIGN> { &self.used }


    /// Returns a mutable reference to the used map
    #[inline(always)]
    pub(crate) unsafe fn used_map_mut(&mut self) -> &mut BTreeMap<AlignedNonNull<NonZero<u64>, ALIGN>, NonZero<u64>> {
        &mut self.used
    }
    //pub(crate) unsafe fn used_map_mut(&mut self) -> &mut AllocMap<ALIGN> { &mut self.used }



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
    /// The `AlreadyInitialized` error is returned only if this function has already been called in this instance
    #[cold]
    #[inline(never)]
    pub(crate) unsafe fn initialize<I>(&mut self, iter: I) -> Result<(), AlreadyInitialized>
    where I: IntoIterator<Item = TakenExtent<ALIGN, InternalExtent<ALIGN>>> {
        //  TODO: create an `initialize_with_stats()` variant that initializes statistics

        if !self.free.map.is_empty() || !self.used/*.map*/.is_empty() {
            return Err(AlreadyInitialized)
        }


        //let mut total_pages: u64 = 0;
        //let mut used_pages: u64 = 0;


        for ext in iter.into_iter() {

            let (map, ext) = match ext {
                TakenExtent::Free(ext) => (&mut self.free, ext),
                TakenExtent::Used(ext) => {//(&mut self.used, ext)
                    match self.used.insert(ext.address(), ext.size()) {
                        Some(_) => panic!("found overlapping extents"),
                        None => continue,
                    }
                },
                /*TakenExtent::Used(ext) => {
                    used_pages = used_pages.saturating_add(ext.size().get());
                    (&mut self.used, ext)
                }*/
            };

            //total_pages = total_pages.saturating_add(ext.size().get());


            if let Some(_) = map.map.insert(ext.address(), ext.size()) {
                //  no overlapping extents => addresses are unique
                panic!("found overlapping extents")
            }

            let by_size = map.size_index.entry(ext.size()).or_default();
            let by_align = by_size.entry(ext.address().get_align()).or_default();
            if !by_align.insert(ext.address()) {
                //  no overlapping extents => addresses are unique
                panic!("found overlapping extents")
            }

        }

        //  save statistics
        //_ = self.stats.total().store(total_pages, Release);
        //_ = self.stats.used().store(used_pages, Release);

        Ok(())
    }

    fn clear(&mut self) {
        self.free.map.clear();
        self.free.size_index.clear();
        self.used/*.map*/.clear();
        //self.used.size_index.clear();
    }




    /// Allocates one contignous extent as described by the `layout`
    /// - The returned extent needs to be deallocated manually
    #[inline(never)]
    pub fn allocate(&mut self, layout: impl LayoutDescriptor<ALIGN>) -> Option<InternalExtent<ALIGN>> {

        //  finds suitable extent for the allocation
        //  - suitable means at least as big as `layout` dictates + correct layout
        let suitable = self.free.find_suitable(layout.clone())?;

        //  removes an extent described by the `layout` from the free tree
        let allocated = match self.free.extract_extent(suitable, layout) {
            Ok(ext) => ext,
            Err(_) => {
                //  if the extent was found, it can be extracted
                cold_path();
                panic!("failed to extract from free map");
            }
        };

        //  adds the extent described by the `layout` to the used tree

        if let Some(_) = self.used.insert(allocated.address(), allocated.size()) {
            cold_path();
            panic!("failed to insert into used map")
        }
        /*if let Err(_) = self.used.insert_extent(allocated.clone()) {
            cold_path();
            panic!("failed to insert into used map")
        };*/

        //  statistics
        //  _ = self.stats.used().fetch_add(allocated.size().get(), AcqRel);

        Some(allocated.into_regular())
    }


    /// Deallocates the extent
    #[inline]
    pub fn deallocate<'me>(&mut self, ext: InternalExtent<ALIGN>) -> Result<(), ()> {

        let ext = MutableExtent::from_regular(ext);

        {   //  remove ext from used map
            let used_map = unsafe { self.used_map_mut() };
            used_map.remove_entry(&ext.address()).ok_or(())?;
            //used_map.extract_exact_extent(ext.clone())?;
        }

        {   //  insert ext into free map (defragment)
            match unsafe { self.free_map_mut().insert_extent(ext.clone()) } {
                Ok(_) => Ok(()),
                Err(_) => {
                    cold_path();

                    //  recovery: re-insert ext into used
                    let free_map = unsafe { self.free_map_mut() };
                    if let Err(_) = free_map.insert_extent(ext) {
                        //  the same error message as for `cold_recovery_panic!()`
                        cold_panic!("ExtentAllocator error recovery failed: failed to re-insert the extent into used map");
                    }

                    Err(())
                }
            }
        }
    }


    /// Deallocates the `part` of the given extent
    pub fn deallocate_part(&mut self, ext: MutableExtent<ALIGN>, part: MutableExtent<ALIGN>) -> Result<(), PartDeallocError> {

        //  extract from used
        if let None = self.used.remove(&ext.address()) {
            return Err(PartDeallocError::UnregisteredExtent)
        }
        //self.used.extract_extent_part(ext.clone(), part.clone()).map_err(|e| e.into_dealloc_error() )?;

        //  since the function above returned `Ok`
        //      it is guaranteed that `part` fits into `ext`

        //  insert part into free map
        if let Err(_) = self.free.insert_extent_fragmented(&part) {
            cold_panic!("failed to insert the free part of the extent");
        }

        Ok(())
    }


}


/// Indicates that the extent allocator has already been initialized
pub struct AlreadyInitialized;


/// Returned by the `deallocate_part()` function
#[repr(u32, align(4))]
//  NOTE: this enum is directly corresponding to `crate::extent_alloc::helpers::PartExtractionError`
pub enum PartDeallocError {
    /// The given extent was not found in allocator
    UnregisteredExtent = 0,
    /// The part extent does not fit into the bigger extent
    PartDoesNotFit = 1,
    /// Something went wrong, but the system was able to recover
    InternalError = 2,
}
