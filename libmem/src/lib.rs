//! Provides memory management related subsystems for the Constelation kernel
//!
//! This includes:
//! - Physical allocator (work in progress)
//! - Virtual address space mapper (not yet merged, work in progress)
//! - Global allocator (not yet implemented)
//! - And more!

#![no_std]

pub mod extent_alloc;

mod misc;
pub use misc::*;

mod address;
pub use address::*;

//use crate::extent_alloc::allocator::ExtentAllocator;

extern crate alloc;



// /// The exposed physical allocator - the `#[global_alloc]` for allocating physical memory
//pub static PHYSICAL: ExtentAllocator<PAGE_SIZE> = ExtentAllocator::uninit();


//  allocator concept
//      init:
//          1) divide each usable memoey chunks into categories by size
//              - smallest for a few-page-allocations, bigger for more pages, biggest for 1GB frames
//
//      algo: divide each usable chunk into separate used/free blocks
//          - (de)alloc, search used/free blocks
//              - BtreeMap<size, ...>?
//
