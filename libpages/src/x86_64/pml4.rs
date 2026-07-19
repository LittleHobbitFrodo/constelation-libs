use core::ptr::NonNull;

use crate::{PAGE_TABLE_ELEMENT_COUNT, PageEntry, PageTable};



/// PML4 (**P**age **M**ap **L**evel **4**) entry is the highest level of the x86_64 four-level paging
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Pml4Entry(u64);

impl PageEntry for Pml4Entry {}

impl Pml4Entry {

    const ADDRESS_MASK: u64 = 0xFFFFFFFFF000;

    /// Creates new unused entry
    pub const fn new_unused() -> Self { Self(0) }

    /// Creates new entry directly from given bits
    pub const fn new_raw(bits: u64) -> Self { Self(bits) }




    /// Indicates whether this entry is unused/null
    #[inline(always)]
    pub fn is_unused(&self) -> bool { self.0 == 0 }







    /// Mask of the `P`resent bit
    /// - If set, the page is present and loaded in memory
    /// - If cleared, the page is swapped and the MMU will trigger page fault on access
    pub const BITS_PRESENT: u64 = 0b1;

    /// Indicates whether the page is actually stored in memory at the moment
    /// - The `P`resent (`P`) bit
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
            self.0 |= Self::BITS_PRESENT
        } else {
            self.0 &= !Self::BITS_PRESENT
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
    /// - If set, write-back caching is used for the page
    /// - If cleared, write-through caching is used for the page
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
    /// - If set, caching will be disabled for this page
    /// - If cleared, caching will be done accordingly to the state of the `PWT` bit
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

    //  Dirty bit is unused in pml4


    /// Indicates whether the `PS` bit is set
    /// - If so, this library will most likely run into udefined behaviour
    pub(crate) fn is_page_sized(&self) -> bool { self.0 & (0b1 << 7) != 0 }

    /*/// Indicates whether the page has been written to
    /// - The **D**irty (`D`) bit
    #[inline]
    pub fn is_dirty(&self) -> bool { self.0 & (0b1 << 6) != 0 }

    //  PS:  for pml4 entries is disabled for Constellation

    /// Global pages are not invalidated upon setting the `cr3` register
    /// - Constellation spcific: global pages are used only for kernel pages
    ///   - Shared pages too?
    #[inline]
    pub fn is_global(&self) -> bool { self.0 & (0b1 << 7) != 0 }*/


    /// Returns the pointer to the `pdpt` table pointed to by this entry
    /// - TODO: Return AtomicPageTable
    #[inline]
    pub fn address(&self) -> Option<NonNull</*AtomicPageTable<Pdpt>*/[u64; PAGE_TABLE_ELEMENT_COUNT]>> {
        #[cfg(feature = "bootloader")]
        assert!(!self.is_page_sized(), "Constellation does not support 1GB pages");

        NonNull::new((self.0 & Self::ADDRESS_MASK) as usize as *mut [u64; PAGE_TABLE_ELEMENT_COUNT])
    }

    /// Returns the pointer to the raw page table
    #[inline]
    pub(crate) fn address_raw(&self) -> Option<NonNull</*RawPageTable<Pdpt>*/[u64; PAGE_TABLE_ELEMENT_COUNT]>> {
        #[cfg(feature = "bootloader")]
        assert!(!self.is_page_sized(), "Constellation does not support 1GB pages");

        NonNull::new((self.0 & Self::ADDRESS_MASK) as usize as *mut [u64; PAGE_TABLE_ELEMENT_COUNT])
    }


}
