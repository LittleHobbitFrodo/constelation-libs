use core::{marker::PhantomData, num::NonZero};

use crate::extent_alloc::{extent::{Extent, InternalExtent, MutableExtent}, layout::LayoutDescriptor, raw_alloc::RawExtentAlloc};



/// Makes any cache type work with the `CachedExtentAllocator`
///
/// The extent caches serve one purpose: to eliminate the
/// cost of searching for suitable extent when allocating
pub trait ExtentCache<const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> {

    /// Used to initialize the cache at compiltime
    const NEW: Self;

    /// Tries to find any extent that can serve the allocation request
    ///
    /// # Behaviour
    /// ### Success
    /// If the allocation can be fulfilled by the pre-allocated memory, this function is required to return `Ok(Some(suitable))`
    /// - The `suitable` extent is the one deemed suitable to fulfill the allocation request
    ///   - The allocator will then calculate the allocated extent by moving and shrinking the `suitable` extent by `layout.size()` pages
    /// - The allocator is responsible to modify its tables to account with the `chipped` extent
    ///
    /// ### Failure
    /// If the allocation cannot be fulfilled by the pre-allocated memory, this shall return `Ok(None)`
    ///
    /// ### Out of memory
    /// If the cache runs out of memory, it shall return `Err(())`
    /// - The inner allocator is then passed to the `refuel()` function
    fn allocate_cached(&mut self, layout: &Lay) -> Result<Option<InternalExtent<ALIGN>>, RefuelRequest>;

    /// Inserts the pre-allocated extent into the cache
    ///
    /// **NOTE**: This function may be triggered multiple times
    ///
    /// 1. If the cache runs out of memory, the `allocate_cached()` function returns `Err(layout)`,
    /// where the `layout` describes the extent the allocator should pre-allocate for the cache
    /// 2. The pre-allocated extent is then handed to this function
    fn refuel(&mut self, gas_station: &mut RawExtentAlloc<ALIGN>);

    //fn refule_2(&mut self, allocator: &mut RawExtentAlloc<ALIGN>);

}

/// Indicates that the cache has run out of memory and requires refuelling
pub struct RefuelRequest;






/// Extent cache used by the `CachedExtentAllocator` by default
pub struct DefaultCache<const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> {
    cached: Option<MutableExtent<ALIGN>>,
    _lay: PhantomData<Lay>
}

impl<const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> ExtentCache<ALIGN, Lay> for DefaultCache<ALIGN, Lay> {

    const NEW: Self = Self { cached: None, _lay: PhantomData };

    fn allocate_cached(&mut self, layout: &Lay) -> Result<Option<InternalExtent<ALIGN>>, RefuelRequest> {

        match &mut self.cached {
            Some(cached) => {
                if cached.size() >= layout.size() && cached.address().get_align() >= layout.align() {

                    //  the function returns the original extent
                    let original = cached.clone().into_regular();

                    //  the first `layout.size()` pages are
                    // chipped off to be returned to the user
                    match unsafe { cached.split_unchecked(layout.size()) } {
                        Some(remainder) => *cached = remainder,
                        None => self.cached = None,
                    }

                    Ok(Some(original))

                } else {    //  cannot fulfill
                    Ok(None)
                }
            },
            None => {   //  Out of memory
                core::hint::cold_path();

                Err(RefuelRequest)
            }
        }

    }

    #[inline(always)]
    fn refuel(&mut self, gas_station: &mut RawExtentAlloc<ALIGN>) {
        debug_assert!(matches!(self.cached, None));


        todo!();
        //self.cached = Some(MutableExtent::from_regular(fuel))
    }

}
