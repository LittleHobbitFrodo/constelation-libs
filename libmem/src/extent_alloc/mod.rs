//! Provides the `ExtentAllocator` and `PhysicalAllocator`

//  ExtentAlloc<ALIGN> works with RawExtent<ALIGN>
//
//  Traits:
//    - ExtentDefinition<ALIGN>: raw trait for extents (NO LIFETIME)
//    - Extent<'alloc, ALIGN>: exposed trait (LIFETIME)
//
//  Structs:
//    - RawExtent<ALIGN>: used by the allocator internally and for initialization
//    - Frame<'alloc, ALIGN>: exposed (eg. returned by alloc() and taken by dealloc())
//    - AllocatedFrame<'alloc, ALIGN>: exposed (eg. returned by allocate(), deallocation using Frame)

mod allocator;
pub use allocator::*;

mod extent;
pub use extent::*;


pub mod layout;
