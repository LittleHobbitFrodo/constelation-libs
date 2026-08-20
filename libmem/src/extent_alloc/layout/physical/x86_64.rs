use super::PhysicalLayout;
use super::super::LayoutDescriptor;
//use crate::extent_alloc::{layout::LayoutDescriptor};
use crate::{Address, MB, PAGE_SIZE, PAGE_TABLE_ENTRY_COUNT, extent_alloc::layout::IntegerOverflow};
use core::num::NonZero;


impl LayoutDescriptor<PAGE_SIZE> for PhysicalLayout {

    const MAX_PAGE_COUNT: u64 = Self::MASK_COMBINED;

    type Err = IntegerOverflow;

    fn size(&self) -> u64 { self.0.get() & Self::MASK_COMBINED }

    fn from_pages(count: NonZero<u64>) -> Result<Self, Self::Err> {
        if count.get() > Self::MAX_PAGE_COUNT {
            Err(IntegerOverflow)
        } else {
            unsafe {
                Ok(Self(NonZero::new_unchecked(count.get() | (Self::get_align(count) << Self::SHIFT_ALIGN))))
            }
        }
    }

    unsafe fn from_pages_unchecked(count: NonZero<u64>) -> Self { Self(count) }

    #[inline(always)]
    fn align(&self) -> NonZero<u64> {
        unsafe {
            NonZero::new_unchecked(1 << (self.0.get() >> Self::SHIFT_ALIGN))
        }
    }

    fn split(&mut self) -> Option<Self> {

        let count = self.size();

        let half = count.saturating_div(2);

        //  try to optimize the layout for x86_64 paging ()
        let aligned = match half.align_down(PAGE_TABLE_ENTRY_COUNT) {
            0 => {  //  cannot align down
                let aligned = half.align_up(PAGE_TABLE_ENTRY_COUNT);
                if aligned >= count {
                    half
                } else {
                    aligned
                }
            },
            aligned => aligned,
        };

        //  cannot be splitted if zero
        let new_chunk = NonZero::new(aligned)?;

        let resized = {
            debug_assert!(count-aligned > 0);
            unsafe {    //  safety: aligned is less than count
                NonZero::new_unchecked(count-aligned)
            }
        };

        *self = {
            debug_assert!(resized.get() <= Self::MAX_PAGE_COUNT);
            unsafe {
                //  safety: resized does never overflow Self::MAX_PAGE_COUNT
                // because it is less than original page count
                Self::from_pages_unchecked(resized)
            }
        };


        debug_assert!(new_chunk.get() <= Self::MAX_PAGE_COUNT);
        unsafe {
            //  Safety new_chunk never overflows Self::MAX_PAGE_COUNT
            //  because it is less than origin page count
            Some(Self::from_pages_unchecked(new_chunk))
        }
    }

}


impl PhysicalLayout {

    /// Mask of the 1GB page counter
    /// - Highest 12 bits are unused
    const MASK_1GB: u64 = 0xFFFFFFFFC0000;

    /// Mask of the 2MB page counter
    const MASK_2MB: u64 = 0x3FE00;

    /// Mask of the 4KB page counter
    const MASK_4KB: u64 = 0x1FF;

    /// Mask of the alignment shift
    /// - Alignment shift is used to determine the `PhysicalLayout`s alignment
    const MASK_ALIGN: u64 = 0xFFF0000000000000;

    /// Mask of all counters combined
    /// - Indicates the maximum page count any `PhysicalLayout` can hold
    const MASK_COMBINED: u64 = Self::MASK_1GB | Self::MASK_2MB | Self::MASK_4KB;

    /// Used to get/set the count of 1GB pages
    const SHIFT_1GB: u64 = 18;

    /// Used to get/set the count of 4KB pages
    const SHIFT_2MB: u64 = 9;

    /// Used to get the alignment
    const SHIFT_ALIGN: usize = 64-12;



    /// Used to get the alignment for a page count
    fn get_align(count: NonZero<u64>) -> u64 {
        //  optimize?
        if count.get() & Self::MASK_1GB != 0 {
            30  //  1 << 30 == 1GB
        } else if count.get() & Self::MASK_2MB != 0 {
            21  //  1 << 21 == 2MB
        } else {
            12  //  1 << 12 == 4KB
        }
    }

}


/*impl PhysicalLayout {

    /// Returns the count of the 1GB pages
    #[inline(always)]
    fn pages_1gig(&self) -> usize {
        (self.0.get() & Self::MASK_1GB) >> Self::SHIFT_1GB
    }

    /// Returns the count of the 2MB pages
    #[inline(always)]
    fn pages_2mb(&self) -> usize {
        (self.0.get() & Self::MASK_2MB) >> Self::MASK_2MB
    }

    /// Returns the count of the 4KB pages
    #[inline(always)]
    fn pages_4kb(&self) -> usize {
        self.0.get() & Self::MASK_4KB
    }
}*/





#[test]
fn kb4() {

    let layout = PhysicalLayout::from_pages(NonZero::new(PhysicalLayout::MASK_4KB).unwrap())
        .expect("page count overflow");
    assert!(layout.size() == PhysicalLayout::MASK_4KB);
    assert!(layout.0.get() & PhysicalLayout::MASK_4KB == PhysicalLayout::MASK_4KB);

    assert!(layout.align().get() == 4096);    //  4KB
    assert!(((layout.0.get() & PhysicalLayout::MASK_ALIGN) >> PhysicalLayout::SHIFT_ALIGN) == 12);
}


#[test]
fn mb2() {

    let layout = PhysicalLayout::from_pages(NonZero::new(PhysicalLayout::MASK_2MB).unwrap())
        .expect("page count overflow");
    assert!(layout.size() == PhysicalLayout::MASK_2MB);
    assert!(layout.0.get() & PhysicalLayout::MASK_2MB == PhysicalLayout::MASK_2MB);

    assert!(layout.align().get() == 2048 * 1024);  //  2MB
    assert!(((layout.0.get() & PhysicalLayout::MASK_ALIGN) >> PhysicalLayout::SHIFT_ALIGN) == 21);
}

#[test]
fn gb1() {

    let layout = PhysicalLayout::from_pages(NonZero::new(PhysicalLayout::MASK_1GB).unwrap())
        .expect("page count overflow");
    assert!(layout.size() == PhysicalLayout::MASK_1GB);
    assert!(layout.0.get() & PhysicalLayout::MASK_1GB == PhysicalLayout::MASK_1GB);

    assert!(layout.align().get() == 1024 * 1024 * 1024);
    assert!(((layout.0.get() & PhysicalLayout::MASK_ALIGN) >> PhysicalLayout::SHIFT_ALIGN) == 30);

}
