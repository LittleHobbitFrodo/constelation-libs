#![no_std]


/// Contains either a regular page entry or page-sized page entry
pub enum Either<R: Sized, S: Sized> {
    Regular(R),
    Sized(S)
}


//#[cfg(any(target_arch = "x86_64", feature = "testing"))]
mod x86_64;

//#[cfg(any(target_arch = "x86_64", feature = "testing"))]
pub use x86_64::*;


/// Marks a page entry of any kind (even non-atomic ones)
pub(crate) trait PageEntry where Self: Sized {}


/*/// Marker trait used to distinguish atomic page entry of any kind
pub(crate) trait AtomicPageEntry where Self: Sized {}*/


/// Marker trait used to distinguish atomic page entries that can be found in tables
/// - Eq. entries that can work as both page-sized and regular entries
pub(crate) trait PageEntryUnion where Self: PageEntry {}


/// A thread-safe page table
#[allow(private_bounds)]
pub struct PageTable<E: PageEntryUnion> {
    raw: [E; PAGE_TABLE_ELEMENT_COUNT]
}



//  TODO: implement liblock
