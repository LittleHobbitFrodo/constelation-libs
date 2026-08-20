use core::num::NonZero;

use crate::{AlignedAddress, extent_alloc::ExtentAllocator};

mod raw;
pub use raw::*;

mod scoped;
pub use scoped::*;

/// Marks all extents (including `ScopedExtent` and `AllocatedExtent`)
/// and bounds them with a raw extent
pub(crate) trait ExtentMarker<const ALIGN: usize, Ext: Extent<ALIGN>> {}

impl<T, const ALIGN: usize> ExtentMarker<ALIGN, Self> for T
where T: Extent<ALIGN> {}

/// An extent describes a chunk of memory by its starting address and its size
/// - Individual blocks of memory are more often reffered to as **frames**
///
/// > NOTE: Functions that are automatically implemented
/// by this trait are free to use `debug_assert!`ions
#[allow(private_bounds)]
pub trait Extent<const ALIGN: usize>
where Self: Sized + Clone + ExtentMarker<ALIGN, Self> {

    /// Constructs a new extent
    fn new(address: AlignedAddress<usize, ALIGN>, pages: NonZero<usize>) -> Self;

    /// Indicates whether `self` and `other` are right next to each
    /// other and are then "touching" each other
    /// - Returns `false` if the extents are overlapping
    #[inline]
    fn is_touching(&self, other: &'_ Self) -> bool {
        //  NOTE: Extents cannot be overlapping
        *self.end_address() == *other.address()     //  |--SELF--||--OTHER--|
        || *self.address() == *other.end_address()  //  |--OTHER--||--SELF--|
    }

    /// Returns the starting address of the extent
    fn address(&self) -> AlignedAddress<usize, ALIGN>;

    /*/// Moves the extent by overwriting its address (size is kept)
    fn move_to(&mut self, new_address: AlignedAddress<usize, ALIGN>);*/

    /// Calculates the ending address of the frame
    #[inline]
    fn end_address(&self) -> AlignedAddress<usize, ALIGN> {
        unsafe {
            AlignedAddress::new_unchecked(self.address().saturating_add(self.size().get() * ALIGN))
        }
    }

    /// Returns the size in pages
    fn size(&self) -> NonZero<usize>;

    /// Indicates whether `self` fits into `other`
    /// - Uses starting and ending addresses of
    /// the extents to determine the result
    /// - Returns `true` if the extents are equivalent
    ///   - Therefore `assert!(extent.fits_into(extent.clone()))` will pass
    #[inline]
    fn fits_into(&self, other: &'_ Self) -> bool {
        *self.address() >= *other.address() && *self.end_address() <= *other.address()
    }


    /// Indicates whether the two extents are overlapping
    #[inline]
    fn is_overlapping(&self, other: &'_ Self) -> bool {
        !(self.end_address() <= other.address() || self.address() >= other.end_address())
    }

}


/*/// Returned by the `ExtentDefinition::remove_from()` function on success
pub struct RemovedExtents<Ext: Extent<ALIGN>, const ALIGN: usize> {
    /// The original frame - shrinked to create a space for `self`
    pub shrinked: Option<Ext>,
    /// The remaining space after removing
    pub remainder: Option<Ext>
}*/
