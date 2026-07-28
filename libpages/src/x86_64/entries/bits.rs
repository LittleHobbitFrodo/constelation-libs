use crate::PdRegularEntry;


/// Used to initialize any kernel regular page table entry
#[repr(transparent)]
pub struct KernelBits(pub(crate) u64);

impl KernelBits {

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }

    /// Gives the user default bits to construct user `PdRegularEntry`
    ///
    /// Defaults: The entry is present, read-only, accessible only from kernel and **NOT** executable
    pub const fn new() -> Self {
        Self(PdRegularEntry::BITS_PRESENT | PdRegularEntry::BITS_XD)
    }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PdRegularEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PdRegularEntry::BITS_XD)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PdRegularEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PdRegularEntry::ADDRESS_MASK))
    }

}




/// Used to initialize user `PdRegularEntry`
#[repr(transparent)]
pub struct UserBits(pub(crate) u64);


impl UserBits {

    /// Gives the user default bits to construct user `PdSizedEntry`
    ///
    /// Defaults: The entry is present, read-only, accessible from userspace and **NOT** executable
    pub const fn new() -> Self {
        Self(PdRegularEntry::BITS_PRESENT | PdRegularEntry::BITS_US | PdRegularEntry::BITS_XD)
    }

    /// Returns the underlying bits as integer
    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }


    /// Makes the entry not present
    pub const fn unpresent(self) -> Self {
        Self(self.0 & !PdRegularEntry::BITS_PRESENT)
    }

    /// Makes the entry executable
    pub const fn executable(self) -> Self {
        Self(self.0 & !PdRegularEntry::BITS_XD)
    }

    /// Makes the page writeable
    pub const fn writeable(self) -> Self {
        Self(self.0 | PdRegularEntry::BITS_RW)
    }

    /// Sets the address of the entry
    pub const fn address(self, a: usize) -> Self {
        Self(self.0 | (a as u64 & PdRegularEntry::ADDRESS_MASK))
    }

    /// Sets the protection key
    pub const fn protection_key(self, pk: u8) -> Self {
        Self(self.0 | (pk as u64 & 0xF) << 59)
    }

}
