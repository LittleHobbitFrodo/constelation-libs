use super::{PdEntry, PdSizedEntry};

/// Used to initialize regular kernel pd entries
#[repr(transparent)]
pub struct PdKernelBits(pub(crate) u64);

impl PdKernelBits {

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PdEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PdEntry::BITS_XD)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PdEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PdEntry::ADDRESS_MASK))
    }

    /// Converts into `PdSizedEntry`
    /// - Returns `None` if the entry is executable and writeable at the same time
    pub fn into_entry(self) -> Option<PdEntry> {
        PdEntry::new_kernel_entry(self)
    }

}





/// used to initialize regular user pd entries
#[repr(transparent)]
pub struct PdUserBits(pub(crate) u64);

impl PdUserBits {

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PdEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PdEntry::BITS_XD)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PdEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PdEntry::ADDRESS_MASK))
    }

    /// Sets the protection key
    pub const fn protection_key(self, pk: u8) -> Self {
        Self(self.0 | (pk as u64 & 0xF) << 59)
    }

    /// Converts into `PdSizedEntry`
    /// - Returns `None` if the entry is executable and writeable at the same time
    pub fn into_entry(self) -> Option<PdEntry> {
        PdEntry::new_user_entry(self)
    }

}







/// Used to initialize page-sized kernel pd entries
#[repr(transparent)]
pub struct PdSizedKernelBits(pub(crate) u64);

impl PdSizedKernelBits {

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





/// used to initialize page-sized user pd entries
#[repr(transparent)]
pub struct PdSizedUserBits(pub(crate) u64);


impl PdSizedUserBits {

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
