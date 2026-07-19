mod pml4;
pub use pml4::*;

mod pt;
pub use pt::*;

mod pd;
pub use pd::*;

mod virt_addr;
pub use virt_addr::*;

pub const PAGE_TABLE_ELEMENT_COUNT: usize = 512;
