use core::mem::ManuallyDrop;

use alloc::vec::Vec;

use crate::extent_alloc::{OwnedExtent, Extent, ExtentAllocator, ExtentMarker, layout::LayoutDescriptor};



/// A collection of extents allocated for one request
///
/// The difference between `Batch` and `Vec<OwnedExtent>` is:
/// - `Batch` stores exactly one reference to its allocator
/// - `Vec<OwnedExtent>` stores the reference within every of its extents
pub struct Batch<'alloc, const ALIGN: usize, Ext: Extent<ALIGN>, Lay: LayoutDescriptor<ALIGN>> {
    extents: Vec<Ext>,
    alloc: &'alloc ExtentAllocator<ALIGN, Ext, Lay>,
}


impl<'alloc, const ALIGN: usize, Ext: Extent<ALIGN>, Lay: LayoutDescriptor<ALIGN>>
Batch<'alloc, ALIGN, Ext, Lay> {

    pub(super) const fn new(extents: Vec<Ext>, alloc: &'alloc ExtentAllocator<ALIGN, Ext, Lay>) -> Self {
        Self { extents: extents, alloc }
    }

    /// Returns the reference to the allocator
    #[inline(always)]
    pub fn allocator(&self) -> &'alloc ExtentAllocator<ALIGN, Ext, Lay> { self.alloc }

    /// Returns the vector and a reference to the allocator
    ///
    /// # Safety
    /// This function is not unsafe on its own, but calling it may introduce unwanted consequences.
    ///
    /// It is up to the caller to guarantee that the returned `Vec` does not outlive the allocator
    #[inline]
    pub unsafe fn into_inner(self) -> (Vec<Ext>, &'alloc ExtentAllocator<ALIGN, Ext, Lay>) {
        let me = ManuallyDrop::new(self);

        //  Safety: fields can be read safely, because the struct is not used anymore
        let vec = unsafe { core::ptr::read(&me.extents) };
        let alloc = unsafe { core::ptr::read(&me.alloc) };
        _ = me;

        (vec, alloc)
    }

    /// Returns the inner slice
    #[inline(always)]
    pub fn extents(&self) -> &[Ext] { self.extents.as_ref() }

    /// Converts `Batch` into a vector of `OwnedExtent`
    ///
    /// An vector of `OwnedExtent` takes more memory than a vector of `ScopedExtent`
    /// because each of the `AllocatedExtents` contains a reference to its allocator
    pub fn into_vec(self) -> Vec<OwnedExtent<'alloc, ALIGN, Ext, Lay>> {
        let (vec, alloc) = unsafe { self.into_inner() };

        vec.into_iter().map(|ext| OwnedExtent::new(ext, alloc) ).collect()
    }

}


impl<'alloc, const ALIGN: usize, Ext: Extent<ALIGN>, Lay: LayoutDescriptor<ALIGN>>
Drop for Batch<'alloc, ALIGN, Ext, Lay> {
    fn drop(&mut self) {
        for extent in self.extents.iter() {
            self.alloc.deallocate(extent.clone())
                .expect("extent deallocation failed")
        }
    }
}
