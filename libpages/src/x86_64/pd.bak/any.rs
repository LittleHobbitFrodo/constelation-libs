use crate::{AtomicPageEntry, Either, PageEntry, PageEntryUnion, PdEntry, PdSizedEntry};



/// Entry located in the pd level that is used to distinguish
/// between regular and page-sized pd entries
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PdEntryUnion(pub(crate) u64);


impl PageEntry for PdEntryUnion {}

impl PageEntryUnion for PdEntryUnion {}


impl PdEntryUnion {

    /// Indicates whether the entry is page-sized
    #[inline]
    pub fn is_page_sized(&self) -> bool {
        self.0 & PdSizedEntry::BITS_PS != 0
    }

    /// Indicates whether the entry is present in memory
    #[inline]
    pub fn is_present(&self) -> bool {
        self.0 & PdSizedEntry::BITS_PRESENT != 0
    }





}
