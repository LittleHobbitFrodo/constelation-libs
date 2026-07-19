use super::PtEntry;

/// Used to initialize kernel `PtEntry`
#[repr(transparent)]
pub struct PtKernelBits(pub(super) u64);


impl PtKernelBits {


    /// Gives the user default bits to construct kernel `PtEntry`
    ///
    /// Defaults: The entry is present, global, read-only and **NOT** executable
    pub const fn new() -> PtKernelBits {
        PtKernelBits(PtEntry::BITS_PRESENT | PtEntry::BITS_G | PtEntry::BITS_XD)
    }


    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PtEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PtEntry::BITS_XD)
    }

    /// Disables the `G`lobal bit
    pub const fn local(self) -> Self {
        Self(self.0 & !PtEntry::BITS_G)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PtEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PtEntry::ADDRESS_MASK))
    }

    /// Converts into `PtEntry`
    /// - Returns `None` if the entry is executable and writeable at the same time
    pub fn into_entry(self) -> Option<PtEntry> {
        PtEntry::new_kernel_entry(self)
    }

}



/// Used to initialize user `PtEntry`
#[repr(transparent)]
pub struct PtUserBits(pub(super) u64);


impl PtUserBits {

    /// Gives the user default bits to constructs userspace `PtEntry`
    ///
    /// Defaults: The entry is present, read-only, accessible from userspace and **NOT** executable
    pub const fn new() -> PtUserBits {
        PtUserBits(PtEntry::BITS_PRESENT | PtEntry::BITS_US | PtEntry::BITS_XD)
    }

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PtEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PtEntry::BITS_XD)
    }

    /// Disables the `G`lobal bit
    pub const fn local(self) -> Self {
        Self(self.0 & !PtEntry::BITS_G)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PtEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PtEntry::ADDRESS_MASK))
    }

    /// Sets the protection key
    pub const fn protection_key(self, pk: u8) -> Self {
        Self(self.0 | (pk as u64 & 0xF) << 59)
    }

    /// Converts into `PtEntry`
    /// - Returns `None` if the entry is executable and writeable at the same time
    pub fn into_entry(self) -> Option<PtEntry> {
        PtEntry::new_user_entry(self)
    }

}
