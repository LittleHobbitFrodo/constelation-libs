use crate::extent_alloc::{
    raw_alloc::RawExtentAlloc,
    layout::LayoutDescriptor,
    extent::MutableExtent,
};
use core::marker::PhantomData;

use super::{ExtentCache, SourceAllocation, RefuelRequest};

/// Extent cache used by the `CachedExtentAllocator` by default
pub struct DefaultCache<const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> {
    cached: Option<MutableExtent<ALIGN>>,
    _lay: PhantomData<Lay>
}

impl<const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> ExtentCache<ALIGN, Lay> for DefaultCache<ALIGN, Lay> {

    const NEW: Self = Self { cached: None, _lay: PhantomData };

    fn allocate_cached(&mut self, layout: Lay) -> Result<Option<SourceAllocation<ALIGN>>, RefuelRequest> {
        todo!();
    }

    #[inline(always)]
    fn refuel(&mut self, gas_station: &mut RawExtentAlloc<ALIGN>) {
        todo!();
    }

}
