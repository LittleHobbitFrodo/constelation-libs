use core::{fmt::Pointer, ops::Add};

use crate::{PageAlignment, PAGE_SIZE};






/// Marker trait that is internally used to indicate any address type
trait AddressMarker
where Self: PageAlignment + Sized + Copy + Default {
    #[cfg(debug_assertions)]
    fn print(&self, f: &'_ mut core::fmt::Formatter) -> core::fmt::Result;
}

impl AddressMarker for usize {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn print(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:X}", self)
    }
}
impl AddressMarker for u64 {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn print(&self, f: &'_ mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:X}", self)
    }
}
impl<T> AddressMarker for *const T {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn print(&self, f: &'_ mut core::fmt::Formatter) -> core::fmt::Result {
        Pointer::fmt(&self, f)
    }
}
impl<T> AddressMarker for *mut T {
    #[cfg(debug_assertions)]
    #[inline(always)]
    fn print(&self, f: &'_ mut core::fmt::Formatter) -> core::fmt::Result {
        Pointer::fmt(&self, f)
    }
}





/// Marks any address type and provides functions to work with its alignment
pub trait Address where Self: AddressMarker {
    /// Aligns the address up to a specific boundary
    /// - `align` must be a power of two
    fn align_up(self, align: usize) -> Self;
    /// Aligns the address down to a specific boundary
    /// - `align` must be a power of two
    fn align_down(self, align: usize) -> Self;
    /// Indicates whether the address is aligned to a specific boundary
    /// - `align` must be a power of two
    fn is_aligned(&self, align: usize) -> bool;
}


impl Address for usize {
    #[inline(always)]
    fn align_up(self, align: usize) -> Self { (self + (align-1)) & !(align-1) }
    #[inline(always)]
    fn align_down(self, align: usize) -> Self { self & !(align-1) }
    #[inline(always)]
    fn is_aligned(&self, align: usize) -> bool { (self & !(align-1)) == *self }
}

impl Address for u64 {
    #[inline(always)]
    fn align_up(self, align: usize) -> Self { (self + ((align as u64)-1)) & !((align as u64)-1) }
    #[inline(always)]
    fn align_down(self, align: usize) -> Self { self & !((align as u64)-1) }
    #[inline(always)]
    fn is_aligned(&self, align: usize) -> bool { (self & !((align as u64)-1)) == *self }
}

impl<T> Address for *const T {
    #[inline(always)]
    fn align_up(self, align: usize) -> Self { (((self as usize) + (align-1)) & !(align-1)) as *const T }
    #[inline(always)]
    fn align_down(self, align: usize) -> Self { ((self as usize) & !(align-1)) as *const T }
    #[inline(always)]
    fn is_aligned(&self, align: usize) -> bool { ((*self as usize) & !(align-1)) == *self as usize }
}
impl<T> Address for *mut T {
    #[inline(always)]
    fn align_up(self, align: usize) -> Self { (((self as usize) + (align-1)) & !(align-1)) as *mut T }
    #[inline(always)]
    fn align_down(self, align: usize) -> Self { ((self as usize) & !(align-1)) as *mut T }
    #[inline(always)]
    fn is_aligned(&self, align: usize) -> bool { ((*self as usize) & !(align-1)) == *self as usize }
}







/// An address that is guaranteed to be aligned to a specific boundary
/// - The generic parameter `ALIGN` must always be a power of two
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[allow(private_bounds)]
pub struct AlignedAddress<Addr: Address, const ALIGN: usize = PAGE_SIZE>(Addr);

#[allow(private_bounds)]
impl<Addr: Address, const ALIGN: usize> AlignedAddress<Addr, ALIGN> {

    /// Constructs a new `AlignedAddress` if the given address is aligned to a page boundary
    #[inline]
    pub fn new(address: Addr) -> Option<Self> {
        if address.is_aligned(ALIGN) {
            Some(Self(address))
        } else {
            None
        }
    }

    /// Constructs a new `AlignedAddress` without checking proper alignment
    ///
    /// # Safety
    /// It is up to the caller to guarantee that the `address` is properly
    /// aligned to a page boundary. If this rule is violated, the system
    /// can fall into undefined behaviour
    pub const unsafe fn new_unchecked(address: Addr) -> Self { Self(address) }

    /// Creates a new `AlignedAddress` by aligning the given `address` down to a page boundary
    #[inline]
    pub fn new_down(address: Addr) -> Self { Self(address.align_down(ALIGN)) }

    /// Creates a new `AlignedAddress` by aligning the given `address` up to a page boundary
    #[inline]
    pub fn new_up(address: Addr) -> Self { Self(address.align_up(ALIGN)) }

}

impl<Addr: Address, const ALIGN: usize> core::fmt::Debug for AlignedAddress<Addr, ALIGN> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.print(f)
    }
}

impl<Addr: Address, const ALIGN: usize> core::ops::Deref for AlignedAddress<Addr, ALIGN> {
    type Target = Addr;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.0 }
}


impl<Addr: Address, const ALIGN: usize> From<Addr> for AlignedAddress<Addr, ALIGN> {
    fn from(value: Addr) -> Self {
        if value.is_aligned(ALIGN) {
            Self(value)
        } else {
            panic!("unaligned address")
        }
    }
}
