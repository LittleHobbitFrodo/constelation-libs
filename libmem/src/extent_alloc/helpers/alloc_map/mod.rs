use super::{AddressMap, SizeMap};
use crate::{cold_panic, extent_alloc::{extent::RemainingExtents, raw_alloc::PartDeallocError}};

use core::{hint::cold_path, num::NonZero};

use crate::extent_alloc::{
    extent::{Extent, InternalExtent, MutableExtent, RawExtent}, layout::LayoutDescriptor};


#[cfg(test)]
mod tests;

/// Invokes `cold_panic!()` with custom panic message
macro_rules! cold_recovery_panic {
    ($msg:literal) => {{
        cold_panic!("ExtentAllocator error recovery failed: {}", $msg)
    }};
}


#[derive(Debug)]
/// A self of either free or used extents
pub(crate) struct AllocMap<const ALIGN: usize> {
    /// Resolves addresses to extent sizes
    pub(crate) map: AddressMap<ALIGN>,

    /// Resolves extent size -> align -> addresses
    pub(crate) size_index: SizeMap<ALIGN>,
}

impl<const ALIGN: usize> AllocMap<ALIGN> {
    pub const fn uninit() -> Self {
        Self { map: AddressMap::new(), size_index: SizeMap::new() }
    }


    /// Finds a suitable extent in the given map to satisfy the given `layout`
    ///
    /// The returned extent is the one deemed suitable (thus the one found in the map)
    /// - The returned extent does not exactly fit the `layout`
    pub(crate) fn find_suitable(&self, layout: impl LayoutDescriptor<ALIGN>) -> Option<MutableExtent<ALIGN>> {

        'by_size: for (size, by_align) in self.size_index.iter() {
            if *size < layout.size() { continue 'by_size }

            'by_align: for (align, addresses) in by_align.iter() {
                if *align < layout.align() { continue 'by_align }

                //  there shall not be any empty self
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
    ///
    /// An returned `Err`or indicates that the extent was not found
    pub(crate) fn extract_exact_extent(&mut self, extent: MutableExtent<ALIGN>) -> Result<(), ()> {

        let by_align = match self.size_index.get_mut(&extent.size()) {
            Some(by_size) => by_size,
            None => return Err(()),
        };

        //  remove the address from size_index
        //  - addresses are unique, so this would eventually have to happen anyway
        match by_align.get_mut(&(&extent.address().get_align())) {
            Some(addresses) => {
                if addresses.remove(&extent.address()) {

                    if addresses.is_empty() {
                        _ = by_align.remove(&extent.size());
                    }

                } else {
                    cold_path();
                    //  address not found
                    return Err(())
                }
            },
            None => {   //  alignment not found
                return Err(())
            },
        };

        //  remove the extent from the map
        //  - addresses are unique, so this would eventually have to happen anyway
        match self.map.remove(&extent.address()) {
            Some(size) => {
                //  helps with debugging
                debug_assert!(size == extent.size(), "map contains corrupted record: unexpected extent size");
                Ok(())
            },
            None => {
                cold_panic!("extent is not registered in the map, but was found in the size_index, this state is considered corrupted")
            },
        }
    }



    /// Same as `extract_exact_extent_from()`, but this splits the extent based on
    /// the `layout` requirements and inserts the remaining space into the map again
    ///
    /// An returned `Err`or indicates that the extent was not found
    ///
    /// # Parameters
    /// - `extent` is the extent found in the passed `map` that satisfies `layout`
    ///   - It is thus needed to be properly aligned and at least the same size as dictated by the `layout`
    pub(crate) fn extract_extent(&mut self, mut extent: MutableExtent<ALIGN>, layout: impl LayoutDescriptor<ALIGN>) -> Result<MutableExtent<ALIGN>, ()> {

        //  used in recovery
        let original = extent.clone();

        if let Err(_) = self.extract_exact_extent(extent.clone()) {
            cold_path();
            return Err(())
        }

        debug_assert!(extent.size() >= layout.size());

        match unsafe { extent.split_unchecked(layout.size()) } {
            Some(put_back) => {

                //  create new entries in size_index and map
                //  DEfragmentation is undesired

                if let Err(_) = self.insert_extent_fragmented(&put_back) {
                    //  recovery: re-insert original
                    cold_path();
                    match self.insert_extent_fragmented(&original) {
                        Ok(_) => return Err(()),
                        Err(_) => cold_recovery_panic!("failed to re-insert the original extent"),
                    }
                }
            },
            None => {   //  OK
                //  layout.size() == extent.size() => no entry to put back
                //  - entries from both size_index and map are already removed
            },
        }

        Ok(extent)

    }


    /// Extracts the `part` of the `ext`ent from the map
    /// - Removes the `part` from `ext` and inserts the newly created extents
    pub(crate) fn extract_extent_part(&mut self, ext: MutableExtent<ALIGN>, part: MutableExtent<ALIGN>) -> Result<(), PartExtractionError> {

        let RemainingExtents { front, remainder } = match part.clone().remove_from(ext.clone()) {
            Ok(rem_exts) => rem_exts,
            Err(_) => return Err(PartExtractionError::PartDoesNotFit)
        };

        //  remove the whole extent
        if let Err(_) = self.extract_exact_extent(ext.clone()) {
            return Err(PartExtractionError::UnregisteredExtent)
        }

        //  insert the front into (DEfragmentation is undesired)
        if let Some(front) = &front {
            if let Err(_) = self.insert_extent_fragmented(&front) {
                cold_path();

                //  recovery: re-insert ext back
                if let Err(_) = self.insert_extent_fragmented(&ext) {
                    cold_recovery_panic!("failed to re-insert the extent")
                }

                return Err(PartExtractionError::InternalError)
            }
        }

        //  insert the remainder (DEfragmentation is undesired)
        if let Some(rem) = remainder {
            if let Err(_) = self.insert_extent_fragmented(&rem) {
                cold_path();

                //  recovery: extract front, insert ext
                if let Some(front) = front {
                    if let Err(_) = self.extract_exact_extent(front) {
                        cold_recovery_panic!("failed to re-extract front extent")
                    }
                }

                if let Err(_) = self.insert_extent_fragmented(&ext) {
                    cold_recovery_panic!("failed to re-insert the extent")
                }

                return Err(PartExtractionError::InternalError)
            }
        }

        Ok(())

    }



    /// Inserts the extent into the `map` and `size_index`
    /// - Does not take care of fragmentation
    pub(crate) fn insert_extent_fragmented(&mut self, ext: &MutableExtent<ALIGN>) -> Result<(), ()> {

        //  insert into the map
        //      addresses are unique => fail if alread in there
        if let Some(_) = self.map.insert(ext.address(), ext.size()) {
            //panic!("conflicting stuff");
            return Err(())
        }

        let by_align = self.size_index.entry(ext.size()).or_default();

        let addresses = by_align.entry(ext.address().get_align()).or_default();
        if !addresses.insert(ext.address()) {
            //  insertion failed, but the extent is already in the map
            //  - Remove the extent from the map to keep the system sound
            if let None = self.map.remove(&ext.address()) {
                cold_recovery_panic!("failed to remove the extent from the map")
            }
                //.expect("recovery failed: failed to remove the extent from the map");
            return Err(())
        }

        Ok(())
    }


    /// Inserts the extent into the given map and performs defragmentation routine if needed
    ///
    /// Since any state where this function is unable to perform its
    /// task is considered unexpected and/or undefined, it will simply panic when such state is detected
    pub(crate) fn insert_extent(&mut self, mut ext: MutableExtent<ALIGN>) -> Result<(), ()> {
        let end_address = ext.end_address();

        //  used in recovery
        let original = ext.clone();

        //  remove from map
        match self.map.remove(&end_address) {
            Some(neigh_size) => {
                //  has neighbor => defragment

                {   //  remove the neighbor from size_index
                    //      - neighbor is already removed from the map

                    //  panics here are intended: since the neighboring extent was
                    // found in the map, it must always be in the size index.
                    // - Otherwise the state of the allocator is considered corrupted

                    let by_align = match self.size_index.get_mut(&neigh_size) {
                        Some(by_align) => by_align,
                        None => cold_panic!("failed to index size_index"),
                    };

                    let addresses = match by_align.get_mut(&end_address.get_align()) {
                        Some(a) => a,
                        None => cold_panic!("failed to index size_index:by_align"),
                    };

                    if !addresses.remove(&end_address) {
                        cold_panic!("size_index:addresses does not contain the address");
                    }

                    if addresses.is_empty() {
                        if let None = by_align.remove(&end_address.get_align()) {
                            cold_panic!("failed to remove the map from size_index");
                        }
                    }
                }

                let neighbor = MutableExtent::new(end_address, neigh_size);

                //  defragment => merge ext and neighbor
                let merged = {
                    debug_assert!(ext.is_touching(&neighbor));

                    unsafe {
                        ext.merge_unchecked(neighbor.clone())
                    }
                    ext
                };

                //  defragmentation complete, now insert
                if let Err(_) = self.insert_extent_fragmented(&merged) {
                    //  recovery: re-insert the original and neighbor

                    cold_path();
                    if let Err(_) = self.insert_extent_fragmented(&original) {
                        cold_recovery_panic!("failed to reinsert the original extent")
                    }

                    if let Err(_) = self.insert_extent_fragmented(&MutableExtent::from_regular(neighbor.into_regular::<InternalExtent<ALIGN>>())) {
                        cold_recovery_panic!("failed to reinsert the neighboring extent")
                    }

                    return Err(())
                }

                Ok(())
            },
            None => {   //  no neighbor => no defragmentation
                self.insert_extent_fragmented(&ext)
            }
        }
    }


    /// Splits the given extent to the given `size` and inserts
    /// the shrinked and remainder extent into the map
    ///
    /// Returns `Err` if the extent could not be found
    ///
    /// Panics if `size` is larger than the extent size
    pub(crate) fn split_extent(&mut self, mut ext: MutableExtent<ALIGN>, size: NonZero<u64>) -> Result<(), ()> {

        assert!(size < ext.size());

        //  remove the extent because overlapping extents in the map are forbidden
        self.extract_exact_extent(ext.clone())?;

        let original = ext.clone();

        let remainder = match unsafe { ext.split_unchecked(size) } {
            Some(rem) => rem,
            None => {
                //  given that `size < ext.size()` this branch shall never be taken
                cold_path();
                unreachable!("split_unchecked() returned None, this should never happen")
            }
        };

        //  since the user expects that the allocated extent
        // must be deallocated whole, DEfragmentation undesired
        if let Err(_) = self.insert_extent_fragmented(&ext) {
            cold_path();

            //  recovery: insert back ext
            match self.insert_extent_fragmented(&ext) {
                Ok(_) => return Err(()),
                Err(_) => cold_recovery_panic!("failed to insert the splitted extent")
            }
        }


        //  the remainder extent is allocated by the cache
        // so it needs to be in the used map as well
        //
        // remainder extent becomes the next suitable extent (variable)
        // for the next call, so DEfragmentation is undesired once again
        match self.insert_extent_fragmented(&remainder) {
           Ok(_) => Ok(()),
           Err(_) => {
               cold_path();

               //   recovery: remove ext, insert original
               if let Err(_) = self.extract_exact_extent(ext) {
                   cold_recovery_panic!("failed to insert remainder extent")
               }

               match self.insert_extent_fragmented(&original) {
                   Ok(_) => return Err(()),
                   Err(_) => cold_panic!("failed to insert"),
               }
           }
        }

    }
}




/// Returned by the `extract_extent_part()` function
/// - Convertable into `PartDeallocError`
#[repr(u32, align(4))]
pub(crate) enum PartExtractionError {
    /// The given extent was not found in allocator
    UnregisteredExtent = 0,
    /// The part extent does not fit into the bigger extent
    PartDoesNotFit = 1,
    /// Something went wrong, but the system was able to recover
    InternalError = 2,
}


impl PartExtractionError {
    pub(crate) fn into_dealloc_error(self) -> PartDeallocError {
        unsafe {
            core::mem::transmute(self)
        }
    }
}
