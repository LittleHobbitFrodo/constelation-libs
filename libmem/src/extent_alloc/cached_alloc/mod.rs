use core::{hint::cold_path, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::{cold_panic, extent_alloc::{
    ExtentAlloc, ExtentAllocMarker, cache::{DefaultCache, ExtentCache, RefuelRequest}, extent::{AbstractExtent, Extent, InternalExtent, MutableExtent, /*OwnedExtent, */RawExtent, ScopedExtent, TakenExtent}, layout::{ExtentLayout, LayoutDescriptor}, raw_alloc::{AlreadyInitialized, RawExtentAlloc}
}};

use alloc::vec::Vec;

/// An implementation of an extent allocator that operates with
/// a predetermined page size, various sizes of allocated blocks,
/// and any number of memory blocks.
///
/// `CachedExtentAllocator` uses a cache to speed up lookup
/// for small extents. The cache can be implemented by the user,
/// so feel free to optimize it for your own usecase
///
/// > **IMPORTANT**: `CachedExtentAllocator` uses the heap,
/// therefore the heap must be initialized first
///
/// This implementation is optimized only for allocation and deallocation.
/// - Deallocation of parts of allocated frames is also supported!
///
/// > Note: The generic constant `ALIGN` must be a power of
/// two; violating this rule may result in undefined behavior
pub struct CachedExtentAllocator<const ALIGN: usize,
    Ext: Extent<ALIGN> + RawExtent<ALIGN> = InternalExtent<ALIGN>,
    Lay: LayoutDescriptor<ALIGN> = ExtentLayout<ALIGN>,
    Cache: ExtentCache<ALIGN, Lay> = DefaultCache<ALIGN, Lay>>
{
    alloc: RawExtentAlloc<ALIGN>,
    cache: Cache,
    _ext: PhantomData<Ext>,
    _lay: PhantomData<Lay>,
}

//  mark extent allocator
impl<const ALIGN: usize, Ext: Extent<ALIGN> + RawExtent<ALIGN>, Lay: LayoutDescriptor<ALIGN>, Cache: ExtentCache<ALIGN, Lay>>
ExtentAllocMarker for CachedExtentAllocator<ALIGN, Ext, Lay, Cache> {}

//  trait for use in generics
impl<const ALIGN: usize, Ext: Extent<ALIGN> + RawExtent<ALIGN>, Lay: LayoutDescriptor<ALIGN>, Cache: ExtentCache<ALIGN, Lay>>
ExtentAlloc<ALIGN> for CachedExtentAllocator<ALIGN, Ext, Lay, Cache> {

}


impl<const ALIGN: usize, Ext: Extent<ALIGN> + RawExtent<ALIGN>, Lay: LayoutDescriptor<ALIGN>, Cache: ExtentCache<ALIGN, Lay>>
CachedExtentAllocator<ALIGN, Ext, Lay, Cache> {

    /// Returns a pointer to this instance
    #[inline]
    fn as_ptr(&self) -> NonNull<Self> {
        unsafe {
            NonNull::new_unchecked(self as *const Self as *mut Self)
        }
    }

    /// Constructs an uninitialized `CachedExtentAllocator`
    pub const fn uninit() -> Self {
        Self {
            alloc: RawExtentAlloc::uninit(),
            cache: Cache::NEW,
            _ext: PhantomData,
            _lay: PhantomData
        }
    }

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
    #[inline(never)]
    pub fn allocate_contignous<'me>(&'me mut self, layout: Lay) -> Option<ScopedExtent<'me, ALIGN, Ext, Self>> {
        match self.cache.allocate_cached(&layout) {
            Ok(Some(suitable)) => { //  fulfilled by the cache

                debug_assert!(suitable.size() >= layout.size());

                let mut allocated = MutableExtent::from_regular(suitable.clone());

                match unsafe { allocated.split_unchecked(layout.size()) } {
                    Some(remainder) => {
                        //  the allocated extent is smaller than the one deemed suitable
                        //      => split of the suitable extent is required

                        let suitable = MutableExtent::from_regular(suitable);

                        let used_map = unsafe { self.alloc.used_map_mut() };

                        //  remove the
                        if let None = used_map.remove(&suitable.address()) {
                            cold_panic!("splitting suitable extent failed")
                        }

                        if let Some(_) = used_map.insert(remainder.address(), remainder.size()) {
                            cold_panic!("splitting suitable extent failed")
                        }

                        /*if let Err(_) = used_map.split_extent(suitable, allocated.size()) {
                            cold_panic!("splitting of suitable extent failed");
                        }*/
                    },
                    None => {
                        //  no remainder
                        //  the suitable extent is already in the used map
                        //      => no work to do
                    }
                }

                unsafe {
                    Some(ScopedExtent::from_extent(allocated.into_regular()))
                }
            },
            Ok(None) => {   //  cache miss

                self.alloc.allocate(layout).map(|ext| unsafe {
                    ScopedExtent::from_extent(Ext::new(ext.address(), ext.size()))
                })
            },
            Err(RefuelRequest) => {  //  cache is out of memory
                // refuel and allocate

                core::hint::cold_path();

                self.cache.refuel(&mut self.alloc);

                self.alloc.allocate(layout).map(|ext| unsafe {
                    ScopedExtent::from_extent(Ext::new(ext.address(), ext.size()))
                })
            }
        }
    }


    pub fn allocate<'me>(&'me mut self, layout: Lay) -> Option<Vec<ScopedExtent<'me, ALIGN, Ext, Self>>> {
        todo!();
    }


    /// Deallocates the extent
    #[inline]
    #[must_use]
    pub fn deallocate<'me>(&mut self, ext: ScopedExtent<'me, ALIGN, Ext, Self>) -> Result<(), ()> {
        let ext = {
            let ext = ManuallyDrop::new(ext);
            InternalExtent::new(ext.address(), ext.size())
        };
        self.alloc.deallocate(ext)
    }

}
