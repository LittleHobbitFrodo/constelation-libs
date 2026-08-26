mod physical;
pub use physical::*;

use crate::AlignedAddress;
#[cfg(debug_assertions)]
use core::fmt::Debug;
use core::num::NonZero;


/// Marks all memory layout descriptors to work with the `ExtentAllocator`
pub trait LayoutDescriptor<const ALIGN: usize> where Self: Sized + Clone {
    /// Maximum possible amount of pages that the descriptor can hold
    const MAX_PAGE_COUNT: u64;

    /// An error type used by the `from_pages()` function
    type Err: Sized;

    /// Size (in pages) of the frame to allocate
    fn size(&self) -> NonZero<u64>;

    /// Constructs the descriptor from a count of pages
    fn from_pages(count: NonZero<u64>) -> Result<Self, Self::Err>;

    /// Constructs the descriptor without checking any invariant
    unsafe fn from_pages_unchecked(count: NonZero<u64>) -> Self;

    /// Returns the alignment requirements of the allocated frame
    fn align(&self) -> NonZero<u64>;

    /// Splits a bigger extent into smaller pieces if it cannot be allocated
    ///
    /// # Behaviour
    /// This function is invoked by the `ExtentAllocator` when the current
    /// block is too large to be allocated and is intended to work as
    /// iterator. It thus may be called multiple times on the same frame
    ///
    /// `split()` searches for a valid frame size that is smaller than `self`.
    /// If found, it proceeds to shrink `self` by that size and returns the newly created frame
    /// - > **Note for physical allocators**: It is reccomended that the calculation is architecture-specific and tries to find a value
    /// that can be easily grabbed by the current paging implementaion
    ///
    /// Returns `None` if the smaller frame size cannot be found
    fn split(&mut self) -> Option<Self>;
}

/// An error type returned by `LayoutDescriptor::from_pages()`
/// as error by default
pub struct IntegerOverflow;
impl core::fmt::Debug for IntegerOverflow {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "integer overflow")
    }
}


/// Used by the `ExtentAllocator` to allocate contignous extents
/// - The layout alignment is always 1
#[repr(transparent)]
#[derive(Clone)]
pub struct ExtentLayout<const ALIGN: usize>(pub(crate) NonZero<u64>);

impl<const ALIGN: usize> LayoutDescriptor<ALIGN> for ExtentLayout<ALIGN> {

    const MAX_PAGE_COUNT: u64 = 0xFFFFFFFFFFFFF;

    type Err = IntegerOverflow;

    #[inline(always)]
    fn size(&self) -> NonZero<u64> { self.0 }

    #[inline(always)]
    fn from_pages(count: NonZero<u64>) -> Result<Self, Self::Err> {
        if count.get() <= Self::MAX_PAGE_COUNT {
            Ok(Self(count))
        } else {
            Err(IntegerOverflow)
        }
    }

    #[inline(always)]
    unsafe fn from_pages_unchecked(count: NonZero<u64>) -> Self { Self(count) }

    #[inline(always)]
    fn align(&self) -> NonZero<u64> {
        unsafe {
            NonZero::new_unchecked(1)
        }
    }

    fn split(&mut self) -> Option<Self> {
        let half = NonZero::new(self.0.get().saturating_div(2))?;

        *self = unsafe {
            //  safety:
            let count = NonZero::new_unchecked(self.0.get() - half.get());
            Self::from_pages_unchecked(count)
        };

        Some(Self(half))
    }

}

#[cfg(debug_assertions)]
impl<const ALIGN: usize> Debug for ExtentLayout<ALIGN> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ExtentLayout {{ size: {}, align: {} }}", self.size(), self.align())
    }
}
