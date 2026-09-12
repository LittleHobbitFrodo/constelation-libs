//! Provides memory management related subsystems for the Constelation kernel
//!
//! This includes:
//! - Extent allocator (work in progress)
//! - Global allocator (not yet implemented)
//! - And more!


#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_op_in_unsafe_fn)]

#![no_std]

#[cfg(feature = "extent_alloc")]
pub mod extent_alloc;

mod misc;
pub use misc::*;

mod address;
pub use address::*;

//use crate::extent_alloc::allocator::ExtentAllocator;

extern crate alloc;


/// Same as the `panic!()` macro, but hints a `cold_path()`
/// to tell the compiler to optimize other paths
///
/// Panics in a kernel may not be unlikely (early development
/// stage, etc.), but given that a panic only occurs once in a
/// program lifetime (and terminates the panicking program),
/// calling `cold_panic!()` insead of `panic!()` may boost
/// kernel performance by a little
#[macro_export]
macro_rules! cold_panic {
    () => {{
        core::hint::cold_path();
        panic!();
    }};
    ($($arg:tt)*) => {{
        core::hint::cold_path();
        panic!($($arg)*);
    }};
}
