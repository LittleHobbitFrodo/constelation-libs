

//#[cfg(target_arch = "x86_64")]
mod x86_64;
//#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

/// One kilobyte (1024 bytes)
pub const KB: usize = 1024;

/// One megabyte (1024 kilobytes)
pub const MB: usize = 1024 * KB;

/// One gigabyte (1024 megabytes)
pub const GB: usize = 1024 * MB;

/// One terabyte (1024 gigabytes)
pub const TB: usize = 1024 * GB;


/// Trait used to align addresses to a architecture-specific page boundary
pub trait PageAlignment where Self: Sized + Copy {
    /// Aligns the address up to a page boundary
    fn page_align_up(self) -> Self;
    /// Aligns the address down to a page boundary
    fn page_align_down(self) -> Self;
    /// Indicates whether an address is aligned to a page boundary
    fn is_page_aligned(&self) -> bool;
}

impl PageAlignment for usize {
    #[inline]
    fn page_align_up(self) -> Self { (self + (PAGE_SIZE-1)) & !(PAGE_SIZE-1) }

    #[inline]
    fn page_align_down(self) -> Self { self & !(PAGE_SIZE-1) }

    #[inline]
    fn is_page_aligned(&self) -> bool { (self & !(PAGE_SIZE-1)) == *self }
}

impl PageAlignment for u64 {
    #[inline]
    fn page_align_up(self) -> Self { (self + ((PAGE_SIZE as u64)-1)) & !((PAGE_SIZE as u64)-1) }

    #[inline]
    fn page_align_down(self) -> Self { self & !((PAGE_SIZE as u64)-1) }

    #[inline]
    fn is_page_aligned(&self) -> bool { (self & !((PAGE_SIZE as u64)-1)) == *self }
}

impl<T> PageAlignment for *const T {

    #[inline]
    fn page_align_up(self) -> Self {
        (((self as usize) + (PAGE_SIZE-1)) & !(PAGE_SIZE-1)) as *const T
    }

    #[inline]
    fn page_align_down(self) -> Self { ((self as usize) & !(PAGE_SIZE-1)) as *const T }

    #[inline]
    fn is_page_aligned(&self) -> bool { ((*self as usize) & !(PAGE_SIZE-1)) == *self as usize }
}

impl<T> PageAlignment for *mut T {

    #[inline]
    fn page_align_up(self) -> Self {
        (((self as usize) + (PAGE_SIZE-1)) & !(PAGE_SIZE-1)) as *mut T
    }

    #[inline]
    fn page_align_down(self) -> Self { ((self as usize) & !(PAGE_SIZE-1)) as *mut T }

    #[inline]
    fn is_page_aligned(&self) -> bool { ((*self as usize) & !(PAGE_SIZE-1)) == *self as usize }
}


macro_rules! generate_page_alignment_test {
    ($fn_name:ident, $type:ty) => {
        #[test]
        fn $fn_name() {

            let value: $type = (2*PAGE_SIZE + 5) as $type;
            assert!(value.is_page_aligned() == false);
            assert!(value.page_align_up() == (3*PAGE_SIZE) as $type);
            assert!(value.page_align_down() == (2*PAGE_SIZE) as $type);

        }
    };
}

generate_page_alignment_test!(page_alignment_usize, usize);
generate_page_alignment_test!(page_alignment_u64, u64);
generate_page_alignment_test!(page_alignment_const_ptr, *const ());
generate_page_alignment_test!(page_alignment_mut_ptr, *mut ());
