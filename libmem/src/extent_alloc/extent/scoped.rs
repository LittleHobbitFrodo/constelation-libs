use core::{marker::PhantomData, mem::ManuallyDrop, ops::{Deref, DerefMut}};

use crate::{AlignedAddress, extent_alloc::{ExtentMarker, layout::LayoutDescriptor}};
use core::num::NonZero;

use super::{RawExtent, Extent, ExtentAllocator};

/// An extent allocated by the `ExtentAllocator`
///
/// The main difference between this and `RawExtent`
/// is that `Extent` is prevented to outlive its allocator
#[derive(Clone)]
#[repr(C)]
pub struct ScopedExtent<'alloc, const ALIGN: usize, Ext: Extent<ALIGN>> {
    ext: Ext,
    _life: PhantomData<&'alloc ()>,
}
impl<Ext: Extent<ALIGN>, const ALIGN: usize> ExtentMarker<ALIGN, Ext> for ScopedExtent<'_, ALIGN, Ext> {}

impl<Ext: Extent<ALIGN>, const ALIGN: usize> ScopedExtent<'_, ALIGN, Ext> {

    pub(crate) fn new<'alloc>(ext: Ext) -> ScopedExtent<'alloc, ALIGN, Ext>
    where Self: 'alloc {
        Self { ext, _life: PhantomData }
    }

    /// Converts the `Frame` into its inner extent
    /// - Does not `drop` `self`
    ///
    /// # Safety
    /// This function is not unsafe on its own, but the
    /// consequences of calling it may be.
    /// Since `into_extent()` converts a `Frame` — which
    /// has a lifetime bound to its allocator — into an
    /// extent that lacks such a binding, there is a risk
    /// that the converted extent will outlive its allocator,
    /// leading to undefined behavior.
    #[inline]
    pub unsafe fn into_extent(self) -> Ext {
        let ret = self.ext.clone();
        _ = ManuallyDrop::new(self);
        ret
    }

    /// Converts an extent into a `Frame` with a bound to its allocator
    pub unsafe fn from_extent<'alloc>(extent: Ext) -> ScopedExtent<'alloc, ALIGN, Ext>
    where Self: 'alloc {
        Self { ext: extent, _life: PhantomData }
    }

}

impl<Ext: Extent<ALIGN>, const ALIGN: usize> Deref for ScopedExtent<'_, ALIGN, Ext> {
    type Target = Ext;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.ext }
}







/// Simillarly to `ScopedExtent`, it prevents the extent from
/// outliving its allocator. `OwnedExtent` also
/// automatically deallocates the extent when `drop`ped
#[derive(Clone)]
#[repr(C)]
pub struct OwnedExtent<'alloc, const ALIGN: usize, Ext: Extent<ALIGN>, Lay: LayoutDescriptor<ALIGN>> {
    ext: Ext,
    alloc: &'alloc ExtentAllocator<ALIGN, Ext, Lay>,
}
impl<Ext: Extent<ALIGN>, const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> ExtentMarker<ALIGN, Ext> for OwnedExtent<'_, ALIGN, Ext, Lay> {}



impl<'alloc, Ext: Extent<ALIGN>, const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> OwnedExtent<'alloc, ALIGN, Ext, Lay> {
    pub(crate) const fn new(ext: Ext, alloc: &'alloc ExtentAllocator<ALIGN, Ext, Lay>) -> Self {
        Self { ext, alloc }
    }

    pub fn into_scoped(self) -> ScopedExtent<'alloc, ALIGN, Ext> {
        todo!();
    }

}

impl<Ext: Extent<ALIGN>, const ALIGN: usize, Lay: LayoutDescriptor<ALIGN>> Deref for OwnedExtent<'_, ALIGN, Ext, Lay> {
    type Target = Ext;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.ext }
}
