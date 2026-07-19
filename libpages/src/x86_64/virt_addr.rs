
/// Virtual address is used to traverse the page tables
/// - TODO: implement index manpulation
pub struct VirtualAddress(u64);

const NINE_BIT_MASK: u16 = 0x1ff;

impl VirtualAddress {

    pub const OFFSET_MASK: u64 = 0xfff;
    pub const PT_SHIFT: i32 = 12;
    pub const PD_SHIFT: i32 = 21;
    pub const PDPT_SHIFT: i32 = 30;
    pub const PML4_SHIFT: i32 = 38;

    /// Extracts the page offset (lowest 12 bits) from the virtual address
    #[inline]
    pub fn offset(&self) -> u16 { (self.0 & Self::OFFSET_MASK) as u16 }

    /// Returns the index of the `pt` table
    #[inline]
    pub fn index_pt(&self) -> u16 { (self.0 >> Self::PT_SHIFT) as u16 & NINE_BIT_MASK }

    /// Returns the index of the `pd` table
    #[inline]
    pub fn index_pd(&self) -> u16 { (self.0 >> Self::PD_SHIFT) as u16 & NINE_BIT_MASK }

    /// Returns the index of the `pdpt` table
    #[inline]
    pub fn index_pdpt(&self) -> u16 { (self.0 >> Self::PDPT_SHIFT) as u16 & NINE_BIT_MASK }

    /// Returns the index of the `pml4` table
    #[inline]
    pub fn index_pml4(&self) -> u16 { (self.0 >> Self::PML4_SHIFT) as u16 & NINE_BIT_MASK }

    /// Returns the highest unused bits
    #[inline]
    pub fn unused_bits(&self) -> u16 { (self.0 >> 48) as u16 }

    /// Indicates whether this address is kernelspace address by looking at the unused bits
    #[inline]
    pub fn is_kernel_address(&self) -> bool { self.unused_bits() == u16::MAX }

}
