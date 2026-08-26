use core::num::NonZero;

use crate::{AlignedAddress, AlignedNonNull};
use super::Extent;



/// An extent type used by the `ExtentAllocator` by default
#[derive(Clone)]
#[repr(C)]
pub struct RawExtent<const ALIGN: usize> {
    start: AlignedNonNull<NonZero<u64>, ALIGN>,
    size: NonZero<u64>
}

impl<const ALIGN: usize> Extent<ALIGN> for RawExtent<ALIGN> {

    #[inline(always)]
    fn new(address: AlignedNonNull<NonZero<u64>, ALIGN>, pages: NonZero<u64>) -> Self {
        Self { size: pages, start: address }
    }

    #[inline(always)]
    fn address(&self) -> AlignedNonNull<NonZero<u64>, ALIGN> { self.start }

    #[inline(always)]
    fn size(&self) -> NonZero<u64> { self.size }
}
