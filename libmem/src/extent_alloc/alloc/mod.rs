use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::extent_alloc::{
    extent::{Extent, InternalExtent, RawExtent, TakenExtent, ScopedExtent},
    layout::{ExtentLayout, LayoutDescriptor},
    raw_alloc::{RawExtentAlloc, AlreadyInitialized},
    ExtentAllocMarker, ExtentAlloc,
};



/// An implementation of an extent allocator that operates with
/// a predetermined page size, various sizes of allocated blocks,
/// and any number of memory blocks.
///
/// `ExtentAllocator` does not use any kind of cache. If you want
/// to use any kind of allocation cache, use `CachedExtentAllocator`
///
/// > **IMPORTANT**: `ExtentAllocator` uses the heap,
/// therefore the heap must be initialized first
///
/// This implementation is optimized only for allocation and deallocation.
/// - Deallocation of parts of allocated frames is also supported!
///
/// > Note: The generic constant `ALIGN` must be a power of
/// two; violating this rule may result in undefined behavior
pub struct ExtentAllocator<const ALIGN: usize,
    Ext: Extent<ALIGN> + RawExtent<ALIGN> = InternalExtent<ALIGN>,
    Lay: LayoutDescriptor<ALIGN> = ExtentLayout<ALIGN>>
{
    alloc: RawExtentAlloc<ALIGN>,
    _ext: PhantomData<Ext>,
    _lay: PhantomData<Lay>,
}


//  mark extent allocator
impl<const ALIGN: usize, Ext: Extent<ALIGN> + RawExtent<ALIGN>, Lay: LayoutDescriptor<ALIGN>>
ExtentAllocMarker for ExtentAllocator<ALIGN, Ext, Lay> {}

//  trait for use in generics
impl<const ALIGN: usize, Ext: Extent<ALIGN> + RawExtent<ALIGN>, Lay: LayoutDescriptor<ALIGN>>
ExtentAlloc<ALIGN> for ExtentAllocator<ALIGN, Ext, Lay> {

}

impl<const ALIGN: usize, Ext: Extent<ALIGN> + RawExtent<ALIGN>, Lay: LayoutDescriptor<ALIGN>>
ExtentAllocator<ALIGN, Ext, Lay> {

    /// Constructs an uninitialized `CachedExtentAllocator`
    pub const fn uninit() -> Self {
        Self {
            alloc: RawExtentAlloc::uninit(),
            _ext: PhantomData,
            _lay: PhantomData
        }
    }

    pub(crate) fn inner(&self) -> &RawExtentAlloc<ALIGN> { &self.alloc }
    pub(crate) unsafe fn inner_mut(&mut self) -> &mut RawExtentAlloc<ALIGN> { &mut self.alloc }


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
        unsafe { self.alloc.initialize(iter) }
    }

    /// Allocates an extent as described by the given `layout`
    ///
    /// The returned extent must be deallocated by calling the
    /// `deallocate()` function or transformed into a `OwnedExtent`
    #[inline]
    pub fn alloc_contignous<'me>(&'me mut self, layout: Lay) -> Option<ScopedExtent<'me, ALIGN, Ext, Self>> {
        self.alloc.allocate(layout).map(|ext| unsafe {
            ScopedExtent::<'me, ALIGN, Ext, Self>::from_extent(Ext::new(ext.address(), ext.size()))
        } )
    }


    pub fn alloc<'me>(&'me mut self, layout: Lay) -> Option<Vec<ScopedExtent<'me, ALIGN, Ext, Self>>> {
        todo!();
    }


}
