use core::num::NonZero;

use crate::{AlignedAddress};
use super::Extent;



/// An extent type used by the `ExtentAllocator` internally
#[derive(Clone)]
#[repr(C)]
pub struct RawExtent<const ALIGN: usize> {
    size: NonZero<usize>,
    start: AlignedAddress<usize, ALIGN>
}

impl<const ALIGN: usize> Extent<ALIGN> for RawExtent<ALIGN> {

    #[inline(always)]
    fn new(address: AlignedAddress<usize, ALIGN>, pages: NonZero<usize>) -> Self {
        Self { size: pages, start: address }
    }

    #[inline(always)]
    fn address(&self) -> AlignedAddress<usize, ALIGN> { self.start }

    #[inline(always)]
    fn size(&self) -> NonZero<usize> { self.size }
}
