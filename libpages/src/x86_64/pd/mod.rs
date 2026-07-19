use crate::{Either, PageEntry};
use core::mem::ManuallyDrop;
use core::sync::atomic::{AtomicU64, Ordering};


mod regular;
pub use regular::*;

mod sized;
pub use sized::*;

mod bits;
pub use bits::*;


/// Pd table entry - an union distingushing between regular and page-sized entry
#[repr(transparent)]
pub struct PdEntry(pub(crate) PdEntryUnion);

pub(crate) union PdEntryUnion {
    pub(crate) raw: u64,
    pub(crate) reg: PdRegularEntry,
    pub(crate) sized: PdSizedEntry,
}

impl Clone for PdEntryUnion {
    fn clone(&self) -> Self {
        Self {
            raw: unsafe { self.raw }
        }
    }
}

impl PageEntry for PdEntry {}


impl PdEntry {

    /// Indicates whether the entry is page-sized
    #[inline]
    pub fn is_page_sized(&self) -> bool {
        unsafe { self.0.raw & PdSizedEntry::BITS_PS != 0 }
    }

    /// Indicates whether the entry is present in memory
    #[inline]
    pub fn is_present(&self) -> bool {
        unsafe { self.0.raw & PdRegularEntry::BITS_PRESENT != 0 }
    }


    /// Checks the page size bit and reinterprets itself as regular xor sized entry
    pub fn entry(&self) -> Either<&PdRegularEntry, &PdSizedEntry> {
        if self.is_page_sized() {
            Either::Regular(unsafe { &self.0.reg })
        } else {
            Either::Sized(unsafe { &self.0.sized })
        }
    }

    /// Checks the page size bit and loads corresponding entry
    pub fn entry_copied(&self) -> Either<PdRegularEntry, PdSizedEntry> {
        let bits = unsafe { self.0.raw };

        if bits & PdSizedEntry::BITS_PS == 0 {
            Either::Regular(PdRegularEntry(bits))
        } else {
            Either::Sized(PdSizedEntry(bits))
        }
    }

    /// Reinterprets itself as regular entry
    pub fn as_regular_entry(&self) -> Option<&PdRegularEntry> {
        if self.is_page_sized() {
            None
        } else {
            Some(unsafe { &self.0.reg })
        }
    }

    /// Loads and reinterprets itself as regular entry
    pub fn regular_entry_copied(&self) -> Option<PdRegularEntry> {
        let bits = unsafe { self.0.raw };

        if bits & PdSizedEntry::BITS_PS == 0 {
            Some(PdRegularEntry(bits))
        } else {
            None
        }
    }

    /// Reinterprets itself as page-sized entry
    pub fn as_sized_entry(&self) -> Option<&PdSizedEntry> {
        if self.is_page_sized() {
            Some(unsafe { &self.0.sized })
        } else {
            None
        }
    }

    /// Loads and reinterprets itself as page-sized entry
    pub fn sized_entry_copied(&self) -> Option<PdSizedEntry> {
        let bits = unsafe { self.0.raw };

        if bits & PdSizedEntry::BITS_PS != 0 {
            Some(PdSizedEntry(bits))
        } else {
            None
        }
    }

    /// Creates new entry directly from given bits
    ///
    /// # Safety
    /// This function is safe on its own, but libpages has its own way
    /// of doing things and it does not expect any different approach
    /// - Calling this function may result in undefined behaviour
    pub const unsafe fn new_raw(bits: u64) -> Self {
        Self(PdEntryUnion { raw: bits })
    }

}
