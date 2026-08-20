
use alloc::{collections::{BTreeMap, BTreeSet}, vec::Vec};
use liblock::Mutex;

use super::{Extent, ScopedExtent, OwnedExtent};

use crate::{Address, AlignedAddress, address, extent_alloc::{ExtentMarker, RawExtent, layout::{ExtentLayout, LayoutDescriptor}}};
use core::{marker::PhantomData, num::NonZero};

mod helpers;
use helpers::{Map, Stats};


mod batch;
pub use batch::*;



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
pub struct ExtentAllocator<const ALIGN: usize,
    Ext: Extent<ALIGN> = RawExtent<ALIGN>,
    Lay: LayoutDescriptor<ALIGN> = ExtentLayout<ALIGN>>
{
    map: Mutex<Map<ALIGN>>,
    stats: Stats,
    _lay: PhantomData<Lay>,
    _ext: PhantomData<Ext>,
}


impl<const ALIGN: usize, Ext: Extent<ALIGN>, Lay: LayoutDescriptor<ALIGN>> ExtentAllocator<ALIGN, Ext, Lay> {

    /// Constructs a new uninitialized `ExtentAllocator`
    pub const fn uninit() -> Self {
        Self {
            map: Mutex::new(Map::uninit()),
            stats: Stats::uninit(),
            _lay: PhantomData,
            _ext: PhantomData,
        }
    }


    /// Initializes the `ExtentAllocator` by consuming an iterator that yields an extent
    ///
    /// # Safety
    /// This function is not unsafe on its own, however if the allocator
    /// is fed with invalid data it may introduce undefined behaviour.
    /// Since there is no way to universally validate an extent, this
    /// function is marked as `unsafe`.
    ///
    /// It is up to the caller to guarantee that all extents yielded by the given
    /// iterator are valid at least for the lifetime of the allocator
    #[cold]
    #[inline(never)]
    pub unsafe fn initialize<I>(&self, iter: I) -> Result<(), InitError>
    where I: IntoIterator<Item = Ext> {
        let mut map = self.map.lock();

        let free = &mut map.free;

        if !free.map.is_empty() || !free.size_index.is_empty() {
            return Err(InitError::AlreadyInitialized)
        }

        for frame in iter.into_iter() {

            {   //  find overlapping extents
                let is_overlapping_map = |param: (&AlignedAddress<usize, ALIGN>, &NonZero<usize>)| {
                    let (addr, size) = param;

                    frame.is_overlapping(&Ext::new(addr.clone(), *size))
                };

                if free.map.iter().any(is_overlapping_map) {
                    return Err(InitError::OverlappingExtents)
                }
            }


            //  add to the registry
            if let Some(_) = free.map.insert(frame.address(), frame.size()) {
                return Err(InitError::OverlappingExtents)
            }

            if !free.size_index.entry(frame.size()).or_default()
            .insert(frame.address()) {  //  value was present
                return Err(InitError::OverlappingExtents)
            }
        }

        Ok(())
    }

    /// Initializes the `ExtentAllocator` without checking for overlapping regions
    /// - This function will still return error if it is already initialized
    ///
    /// # Safety
    /// > NOTE: The safety regulations of `ExtentAllocator::initialize()`
    /// still hold place
    ///
    /// It is up to the caller to guarantee that all the
    /// extents yielded by the iterator are not overlapping
    pub unsafe fn initialize_unchecked<I>(&self, iter: I) -> Result<(), InitError>
    where I: IntoIterator<Item = Ext> {
        let mut map = self.map.lock();

        let free = &mut map.free;

        if !free.map.is_empty() || !free.size_index.is_empty() {
            return Err(InitError::AlreadyInitialized)
        }

        for frame in iter.into_iter() {
            //  add to the registry
            if let Some(_) = free.map.insert(frame.address(), frame.size()) {
                return Err(InitError::OverlappingExtents)
            }

            if !free.size_index.entry(frame.size()).or_default()
            .insert(frame.address()) {  //  value was present
                return Err(InitError::OverlappingExtents)
            }
        }

        Ok(())
    }


    /// Allocates one contignous extent as described by the `layout`
    /// - The returned extent needs to be deallocated manually
    #[inline(never)]
    pub fn alloc_contignous(&self, layout: Lay) -> Option<ScopedExtent<'_, ALIGN, Ext>> {
        todo!();
    }


    /// Allocates any amount of extents to satisfy the requested `layout`
    /// - Returned extents must be deallocated manually
    #[inline(never)]
    pub fn alloc(&self, mut layout: Lay) -> Option<Vec<ScopedExtent<'_, ALIGN, Ext>>> {
        todo!();
    }



    /// Allocates one contignous extent as described by the `layout`
    /// - The returned extent is automatically deallocated when `drop`ped
    #[inline(never)]
    pub fn allocate_contignous(&self, layout: Lay) -> Option<OwnedExtent<'_, ALIGN, Ext, Lay>> {
        todo!();
    }


    /// Allocates any number of extents to satisfy the requested `layout`
    #[inline(never)]
    pub fn allocate(&self, layout: Lay) -> Option<Batch<'_, ALIGN, Ext, Lay>> {
        todo!();
    }


    /// Deallocates the extent
    #[allow(private_bounds)]
    #[inline(never)]
    pub fn deallocate<E: ExtentMarker<ALIGN, Ext>>(&self, extent: E) -> Result<(), ()> {
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
