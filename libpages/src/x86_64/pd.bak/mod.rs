use core::ptr::NonNull;

use crate::{AtomicPtEntry, PageEntry, PageTable, PtEntry};


mod sized;
pub use sized::*;

mod bits;
pub use bits::*;

mod any;
pub use any::*;


/// A regular pd table entry
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PdEntry(pub(crate) u64);

impl PageEntry for PdEntry {}

impl PdEntry {

    /// Returns the entry bits as integer
    #[inline]
    pub fn as_u64(&self) -> u64 { self.0 }

    const ADDRESS_MASK: u64 = 0xFFFFFFFFF000;



    /// Gives the user default bits to construct user `PdEntry`
    ///
    /// Defaults: The entry is present, read-only, accessible from userspace and **NOT** executable
    pub const fn user_bits() -> PdUserBits {
        PdUserBits(PdEntry::BITS_PRESENT | PdEntry::BITS_US | PdEntry::BITS_XD)
    }



    /// Gives the user default bits to construct kernel `PdEntry`
    ///
    /// Default: The entry is present, read-only, accessible only by the kernel and **NOT** executable
    pub const fn kernel_bits() -> PdKernelBits {
        PdKernelBits(Self::BITS_PRESENT | Self::BITS_XD)
    }

    /// Constructs new kernel `PdEntry` with given bits
    /// - Returns `None` if the entry is executable and writeable
    pub fn new_kernel_entry(bits: PdKernelBits) -> Option<Self> {
        let entry = Self(bits.as_u64());
        if entry.is_executable() && entry.is_writeable() {
            None
        } else {
            Some(entry)
        }
    }


    /// Constructs new user `PdEntry` with given bits
    /// - Returns `None` if the entry is executable and writeable
    pub fn new_user_entry(bits: PdUserBits) -> Option<Self> {
        let entry = Self(bits.as_u64());
        if entry.is_executable() && entry.is_writeable() {
            None
        } else {
            Some(entry)
        }
    }

    /// Creates new unused entry
    pub const fn new_unused() -> Self { Self(0) }


    /// Creates new entry directly from given bits
    ///
    /// # Safety
    /// This function is safe on its own, but libpages has its own way
    /// of doing things and it does not expect any different approach
    /// - Calling this function may result in undefined behaviour
    pub const unsafe fn new_raw(bits: u64) -> Self { Self(bits) }






    /// Indicates whther this entry is unused/null
    #[inline]
    pub fn is_unused(&self) -> bool { self.0 == 0 }




    /// Mask of the `P`resent bit
    /// - If set, the page is present and loaded in memory
    /// - If cleared, the page is swapped and the MMU will trigger page fault on access
    pub const BITS_PRESENT: u64 = 0b1;

    /// Indicates whether the page is actually stored in memory at the moment
    #[inline]
    pub fn is_present(&self) -> bool { self.0 & Self::BITS_PRESENT != 0 }

    /// Sets the `P`resent bit
    #[inline]
    pub fn make_present(&mut self) { self.0 |= Self::BITS_PRESENT }

    /// Clears the `P`resent bit
    #[inline]
    pub fn make_unpresent(&mut self) { self.0 &= !Self::BITS_PRESENT }

    /// Sets or clears the `P`resent bits
    pub fn set_present(&mut self, present: bool) {
        if present {
            self.make_present();
        } else {
            self.make_unpresent();
        }
    }







    /// Mask of the **R**ead/**W**rite (`RW`) bit
    /// - If set, the page can be written to
    ///   - The CPU will set the `D`irty bit if written to
    /// - If cleared, the page is read-only
    pub const BITS_RW: u64 = 0b1 << 1;

    /// Indicates whether the page this entry is pointing to is writeable, or read only
    /// - The **R**ead/**W**rite (`RW`) bit
    /// - The `WP` bit in the `cr0` register indicates whether write-protection is applied to userland
    ///   - The kernel has always write access by default
    /// - The RW bit is also checked on parent tables
    #[inline]
    pub fn is_writeable(&self) -> bool { self.0 & Self::BITS_RW != 0 }

    /// Sets the `RW` bit and makes the page writeable
    #[inline]
    pub fn make_writeable(&mut self) { self.0 |= Self::BITS_RW }

    /// Clears the `RW` bit and makes the page read-only
    #[inline]
    pub fn make_read_only(&mut self) { self.0 &= !Self::BITS_RW }

    /// Sets or clears the **R**ead/**W**rite (`RW`) bit
    #[inline]
    pub fn set_writeable(&mut self, writeable: bool) {
        if writeable {
            self.make_writeable();
        } else {
            self.make_read_only();
        }
    }








    /// Mask of the **U**ser/**S**upervisor (`US`) bit
    /// - If set, the page can be accessed by both kernel and userspace
    /// - If cleared, the page can be accessed by only the kernel
    pub const BITS_US: u64 = 0b1 << 2;

    /// Indicates whether userspace has access to the page
    /// - The **U**ser/**S**upervisor (`US`) bit
    ///
    /// - TO make a page accesible by userspace, all levels must have this bit set
    ///   - **TODO**: factcheck
    #[inline]
    pub fn is_userspace(&self) -> bool { self.0 & Self::BITS_US != 0 }

    /// Sets the **U**ser/**S**upervisor (`US`) bit and makes the page accessible by userspace
    #[inline]
    pub fn make_userspace(&mut self) { self.0 |= Self::BITS_US }

    /// Clears the **U**ser/**S**upervisor (`US`) bit and makes the page accessible only by the kernel
    #[inline]
    pub fn make_kernel_only(&mut self) { self.0 &= !Self::BITS_US }

    /// Sets or clears the **U**ser/**S**upervisor bit
    #[inline]
    pub fn set_userspace(&mut self, userspace: bool) {
        if userspace {
            self.make_userspace();
        } else {
            self.make_kernel_only();
        }
    }











    /// Mask of the `PWT` bit
    /// - If `PAT` is not supported or the `PAT` bit is cleared, must be cleared too
    ///
    /// If set, write-back caching is used for the page
    /// If cleared, write-through caching is used for the page
    ///
    /// Use the `PCD` bit to disable caching
    pub const BITS_PWT: u64 = 0b1 << 3;

    /// indicates whether the page is cached by write-through, or write-back
    /// - The `PWT` bit
    #[inline]
    pub fn is_write_through_cached(&self) -> bool { self.0 & Self::BITS_PWT != 0 }


    /// Clears the `PWT` bit to use write-through caching
    #[inline]
    pub fn cache_write_through(&mut self) { self.0 &= !Self::BITS_PWT }

    /// Sets the `PWT` bit to use write-back caching
    #[inline]
    pub fn cache_write_back(&mut self) { self.0 |= Self::BITS_PWT }

    /// Sets write-through or write-back caching method
    #[inline]
    pub fn set_caching_method(&mut self, write_through: bool) {
        if write_through {
            self.cache_write_through();
        } else {
            self.cache_write_back();
        }
    }







    /// Mask for the cache-disable (`PCD`) bit
    /// - If `PAT` is not supported or the `PAT` bit is cleared, must be cleared too
    ///
    /// If set, caching will be disabled for this page
    /// If cleared, caching will be done accordingly to the state of the `PWT` bit
    pub const BITS_PCD: u64 = 0b1 << 4;

    /// Indicates whether the page is cached
    #[inline]
    pub fn is_cached(&self) -> bool { self.0 & Self::BITS_PCD == 0 }

    /// Clears the cache-disable (`PCD`) bit to enable caching
    #[inline]
    pub fn caching_enable(&mut self) { self.0 &= !Self::BITS_PCD }

    /// Sets the cache-disable (`PCD`) bit to disable caching
    #[inline]
    pub fn caching_disable(&mut self) { self.0 |= Self::BITS_PCD }

    /// Enables or disables caching by manipulating the cache-disable (`PCD`) bit
    pub fn set_caching(&mut self, enable: bool) {
        if enable {
            self.caching_enable();
        } else {
            self.caching_disable();
        }
    }








    /// Mask of the `A`ccessed bit
    /// - Set by the CPU if this entry has been accessed by the
    /// MMU while translating virtual addresses to physical addresses
    pub const BITS_A: u64 = 0b1 << 5;

    /// Indicates whether this entry has been accessed by the MMU
    /// while translating virtual addresses to physical addresses
    #[inline]
    pub fn is_accessed(&self) -> bool { self.0 & Self::BITS_A != 0 }




    //  D and PS are cleared for non-page-sized pd entries
    //  bits 8..=11 are available






    /// Returns the address of the page this entry is pointing to
    #[inline]
    pub(crate) fn table_address(&self) -> Option<NonNull<PageTable<AtomicPtEntry>>> {
        NonNull::new((self.0 & Self::ADDRESS_MASK) as usize as *mut _)
    }

    /// Returns the address of the page this entry is pointing to
    #[inline]
    pub(crate) fn table_address_raw(&self) -> *const PageTable<AtomicPtEntry> {
        (self.0 & Self::ADDRESS_MASK) as usize as *const _
    }

    /// Returns the address of the page this entry is pointing to
    #[inline]
    pub fn address(&self) -> Option<NonNull<PageTable<AtomicPtEntry>>> {
        NonNull::new((self.0 & Self::ADDRESS_MASK) as usize as *mut _)
    }

    /// Returns the address of the page this entry is pointing to
    #[inline]
    pub fn address_raw(&self) -> *const PageTable<AtomicPtEntry> {
        (self.0 & Self::ADDRESS_MASK) as usize as *const _
    }







    /// Mask of the **E**xecut **D**isable (`XD`) bit
    /// - If set, instructions are not allowed to be executed from this page
    ///
    /// If the `NXE` bit in the `EFER` is set, pages marked with `XD` cannot be executed
    /// - If it is cleared, the `XD` bit is reserved and should be cleared
    pub const BITS_XD: u64 = 0b1 << 63;


    /// Indicates whether the page is executable
    #[inline]
    pub fn is_executable(&self) -> bool { self.0 & Self::BITS_XD == 0 }

    /// Makes the page executable
    #[inline]
    pub fn make_executable(&mut self) { self.0 |= Self::BITS_XD }

    /// Disables execution for this page
    #[inline]
    pub fn disable_execute(&mut self) { self.0 &= !Self::BITS_XD }


    /// Enables or disables execution on this page
    #[inline]
    pub fn set_executable(&mut self, exec: bool) {
        if exec {
            self.make_executable();
        } else {
            self.disable_execute();
        }
    }
}






#[cfg(debug_assertions)]
impl core::fmt::Debug for PdEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PDEntry({}, {}, {}, {}, {}, {}, address: {:p}, {})",
            if self.is_present() { "present" } else { "unpresent" },
            if self.is_writeable() { "write" } else { "read" },
            if self.is_userspace() { "user" } else { "kernel" },
            if self.is_write_through_cached() { "write-through" } else { "write-back" },
            if self.is_cached() { "cached" } else { "not cached" },
            if self.is_accessed() { "accessed" } else { "not accessed" },
            self.table_address_raw(),
            if self.is_executable() { "exec" } else { "no exec" }
        )
    }
}
