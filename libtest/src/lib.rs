#![no_std]

pub use macros::*;

#[cfg(any(feature = "testing", feature = "harness"))]
pub mod exposed;

#[cfg(not(any(feature = "testing", feature = "harness")))]
mod exposed;



mod output;
#[cfg(not(feature = "harness"))]
pub use output::*;
