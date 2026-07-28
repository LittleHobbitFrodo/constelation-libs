
use super::{PdSizedEntry};


/// Used to initialize kernel `PdSizedEntry`
#[repr(transparent)]
pub struct PdSizedKernelBits(pub(crate) u64);


impl PdSizedKernelBits {


    /// Gives the user default bits to construct kernel `PdSizedEntry`
    ///
    /// Defaults: The entry is present, read-only, page-sized, accessible only by the kernel and **NOT** executable
    pub const fn new() -> Self {
        Self(PdSizedEntry::BITS_PRESENT | PdSizedEntry::BITS_PS | PdSizedEntry::BITS_XD)
    }

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PdSizedEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PdSizedEntry::BITS_XD)
    }

    /// Disables the `G`lobal bit
    pub const fn local(self) -> Self {
        Self(self.0 & !PdSizedEntry::BITS_G)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PdSizedEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PdSizedEntry::ADDRESS_MASK))
    }

    /// Converts into `PdSizedEntry`
    /// - Returns `None` if the entry is executable and writeable at the same time
    pub fn into_entry(self) -> Option<PdSizedEntry> {
        PdSizedEntry::new_kernel_entry(self)
    }

}



/// Used to initialize user `PdSizedEntry`
#[repr(transparent)]
pub struct PdSizedUserBits(pub(crate) u64);


impl PdSizedUserBits {

    /// Gives the user default bits to construct user `PdSizedEntry`
    ///
    /// Defaults: The entry is present, read-only, page-sized, accessible from userspace and **NOT** executable
    pub const fn new() -> Self {
        Self(PdSizedEntry::BITS_PRESENT | PdSizedEntry::BITS_US | PdSizedEntry::BITS_PS | PdSizedEntry::BITS_XD)
    }

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PdSizedEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PdSizedEntry::BITS_XD)
    }

    /// Disables the `G`lobal bit
    pub const fn local(self) -> Self {
        Self(self.0 & !PdSizedEntry::BITS_G)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PdSizedEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PdSizedEntry::ADDRESS_MASK))
    }

    /// Sets the protection key
    pub const fn protection_key(self, pk: u8) -> Self {
        Self(self.0 | (pk as u64 & 0xF) << 59)
    }

    /// Converts into `PdSizedEntry`
    /// - Returns `None` if the entry is executable and writeable at the same time
    pub fn into_entry(self) -> Option<PdSizedEntry> {
        PdSizedEntry::new_user_entry(self)
    }

}
