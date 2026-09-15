use alloc::collections::BTreeMap;

use crate::{AlignedNonNull, extent_alloc::{
    extent::{Extent, InternalExtent, MutableExtent, RemainingExtents, ScopedExtent, TakenExtent}, helpers::SuitableExtent, layout::LayoutDescriptor,
}};
use super::helpers::AllocMap;
use core::{hint::cold_path, num::NonZero};
use crate::cold_panic;



/// The raw extent allocator internally used by the `ExtentAllocator` and `CachedExtentAllocator`
#[repr(C)]
pub struct RawExtentAlloc<const ALIGN: usize> {
    free: AllocMap<ALIGN>,
    used: BTreeMap<AlignedNonNull<NonZero<u64>, ALIGN>, NonZero<u64>>
}

impl<const ALIGN: usize> RawExtentAlloc<ALIGN> {
    pub const fn uninit() -> Self {
        Self { used: BTreeMap::new(), free: AllocMap::uninit() }
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


    /// Returns a mutable reference to the used map
    #[inline(always)]
    pub(crate) unsafe fn used_map_mut(&mut self) -> &mut BTreeMap<AlignedNonNull<NonZero<u64>, ALIGN>, NonZero<u64>> {
        &mut self.used
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
    /// The `AlreadyInitialized` error is returned only if this function has already been called in this instance
    #[cold]
    #[inline(never)]
    pub(crate) unsafe fn initialize<I>(&mut self, iter: I) -> Result<(), AlreadyInitialized>
    where I: IntoIterator<Item = TakenExtent<ALIGN, InternalExtent<ALIGN>>> {

        if !self.free.size_index.is_empty() || !self.free.map.is_empty() || !self.used.is_empty() {
            return Err(AlreadyInitialized)
        }

        for ext in iter.into_iter() {

            match ext {
                TakenExtent::Free(ext) => {
                    if let Err(_) = self.free.insert_extent_fragmented(&MutableExtent::from_regular(ext)) {
                        cold_panic!("found overlapping extents")
                    }
                },
                TakenExtent::Used(ext) => {

                    //  insert into the used map
                    if let Some(_) = self.used.insert(ext.address(), ext.size()) {
                        cold_panic!("found overlapping extents")
                    }
                },
            }
        }

        Ok(())
    }

    fn clear(&mut self) {
        self.free.map.clear();
        self.free.size_index.clear();
        self.used.clear();
    }




    /// Allocates one contignous extent as described by the `layout`
    /// - The returned extent needs to be deallocated manually
    #[inline(never)]
    pub fn allocate_exact(&mut self, layout: impl LayoutDescriptor<ALIGN>) -> Option<InternalExtent<ALIGN>> {

        debug_assert!(layout.align().is_power_of_two());

        let SuitableExtent { suitable, allocated } = self.free.find_suitable(layout)?;

        debug_assert!(allocated.fits_into(&suitable));


        //  remove the suitable extent from the free map
        if let Err(_) = self.free.extract_exact_extent(suitable.clone()) {
            cold_path();
            return None
        }

        //  remove the allocated extent from the suitable extent
        match allocated.clone().remove_from(suitable.clone()) {
            Ok(RemainingExtents { front, remainder }) => {
                //  insert the front and remainder extents into the used map

                if let Some(front) = front {
                    if let Some(_) = self.used.insert(front.address(), front.size()) {
                        //  recovery: re-insert suitable into free

                        cold_path();
                        if let Err(_) = self.free.insert_extent_fragmented(&suitable) {
                            cold_panic!("ExtentAllocator recovery failed: could not re-insert suitable extent");
                        }
                    }
                }

                if let Some(rem) = remainder {
                    if let Some(_) = self.used.insert(rem.address(), rem.size()) {
                        cold_panic!("ExtentAllocator recovery failed: could not insert remainder extent")
                    }
                }


                Some(allocated.into_regular())
            },
            Err(_) => None,
        }
    }


    /// Allocates one contignous extent as described by the `layout`. If there
    /// is no extent that can fit the `layout`, this function will instead
    /// choose an extent that is closest to meeting the required `layout`
    #[inline(never)]
    pub fn allocate_closest(&mut self, layout: impl LayoutDescriptor<ALIGN>) -> Option<InternalExtent<ALIGN>> {
        todo!("AllocMap::find_closest_to() is unimplemented");
    }


    /// Deallocates the extent
    #[inline]
    pub fn deallocate<'me>(&mut self, ext: InternalExtent<ALIGN>) -> Result<(), ()> {

        let ext = MutableExtent::from_regular(ext);

        {   //  remove ext from used map
            let used_map = unsafe { self.used_map_mut() };
            used_map.remove_entry(&ext.address()).ok_or(())?;
        }

        {   //  insert ext into free map (defragment)
            match unsafe { self.free_map_mut().insert_extent(ext.clone()) } {
                Ok(_) => Ok(()),
                Err(_) => {
                    cold_path();

                    //  recovery: re-insert ext into used
                    let used_map = unsafe { self.used_map_mut() };
                    if let Some(_) = used_map.insert(ext.address(), ext.size()) {
                        cold_panic!("ExtentAllocator error recovery failed: failed to re-insert the extent into used map")
                    }

                    Err(())
                }
            }
        }
    }


    /// Deallocates the `part` of the given extent
    pub fn deallocate_part(&mut self, ext: MutableExtent<ALIGN>, part: MutableExtent<ALIGN>) -> Result<(), PartDeallocError> {
        todo!();
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
