use core::{cmp::Ordering::{Equal, Greater, Less}, num::NonZero};

use crate::{AlignedAddress, AlignedNonNull, extent_alloc::Extent};




#[derive(Clone)]
#[repr(C)]
pub struct MutableExtent<const ALIGN: usize> {
    start: AlignedNonNull<NonZero<u64>, ALIGN>,
    size: NonZero<u64>
}

impl<const ALIGN: usize> MutableExtent<ALIGN> {

    /// Constructs a new `MutableExtent`
    #[inline(always)]
    pub const fn new(address: AlignedNonNull<NonZero<u64>, ALIGN>, pages: NonZero<u64>) -> Self {
        Self { size: pages, start: address }
    }

    /// Returns the starting address of the extent
    #[inline(always)]
    pub fn address(&self) -> AlignedNonNull<NonZero<u64>, ALIGN> { self.start }

    /// Calculates the ending address of the frame
    #[inline(always)]
    pub fn end_address(&self) -> AlignedNonNull<NonZero<u64>, ALIGN> {
        self.start.aligned_add(self.size().get() as usize)
    }

    /// Returns the size in pages
    #[inline(always)]
    pub fn size(&self) -> NonZero<u64> { self.size }

    /// Indicates whether `self` and `other` are right next to each
    /// other and are then "touching" each other
    /// - Returns `false` if the extents are overlapping
    #[inline]
    pub fn is_touching<Ext: Extent<ALIGN>>(&self, other: &'_ Ext) -> bool {
        //  NOTE: Extents cannot be overlapping
        *self.end_address() == *other.address()     //  |--SELF--||--OTHER--|
        || *self.address() == *other.end_address()  //  |--OTHER--||--SELF--|
    }


    /// Indicates whether `self` fits into `other`
    /// - Uses starting and ending addresses of
    /// the extents to determine the result
    /// - Returns `true` if the extents are equivalent
    ///   - Therefore `assert!(extent.fits_into(extent.clone()))` will pass
    #[inline]
    pub fn fits_into<Ext: Extent<ALIGN>>(&self, other: &'_ Ext) -> bool {
        *self.address() >= *other.address() && *self.end_address() <= *other.address()
    }

    /// Indicates whether the two extents are overlapping
    #[inline]
    pub fn is_overlapping<Ext: Extent<ALIGN>>(&self, other: &'_ Ext) -> bool {
        !(self.end_address() <= other.address() || self.address() >= other.end_address())
    }


    /// Splits the `MutableExtent` at the given size, returning
    /// the new extent if there is any space left
    /// - Returns `Err` if `size` is greater than the size of `self`
    pub fn split(&mut self, size: NonZero<u64>) -> Result<Option<Self>, ()> {
        match self.size().cmp(&size) {
            Less => Err(()),
            Equal => Ok(None),
            Greater => {

                let new_addr = self.address().aligned_add(size.get() as usize);

                let new_size = unsafe {
                    let new_size = self.size().get() - size.get();

                    debug_assert!(new_size > 0);
                    NonZero::new_unchecked(new_size)
                };

                self.size = size;

                Ok(Some(Self::new(new_addr, new_size)))
            }
        }
    }

    /// Same as `MutableExtent::split_at()`, but does not check whether `size` fits into `self`
    /// - Returns `None` if `self.size()` is equal to `size`
    ///
    /// # Safety
    /// It is up to the caller to guarantee that `size` is less than or equal to `self.size()`. Violating this rule may introduce undefined behaviour
    pub unsafe fn split_unchecked(&mut self, size: NonZero<u64>) -> Option<Self> {
        if self.size() == size {
            None
        } else {
            let new_addr = self.address().aligned_add(size.get() as usize);

            let new_size = unsafe {
                let new_size = self.size().get() - size.get();

                debug_assert!(new_size > 0);
                NonZero::new_unchecked(new_size)
            };

            self.size = size;

            Some(Self::new(new_addr, new_size))
        }
    }


    /// Consumes the `MutableExtent` and converts it into a regular extent
    #[inline]
    pub fn into_regular<Ext: Extent<ALIGN>>(self) -> Ext {
        let Self { size, start } = self;
        Ext::new(start, size)
    }

}


#[test]
fn split_unchecked() {

    use libtestrand::TestRng;

    let mut rand = TestRng::new();

    for _ in 0..200000 {

        let addr = NonZero::new(rand.next_range(1..10240) as u64).unwrap();

        let mut ext: MutableExtent<1024> = MutableExtent::new(
            AlignedNonNull::new_up(addr),
            NonZero::new(rand.next_range(2..10240) as u64).unwrap());
        let original = ext.clone();

        let split_size = NonZero::new(rand.next_range(1..ext.size().get() as usize) as u64).unwrap();

        let splitted = unsafe {
            ext.split_unchecked(split_size)
                .expect("split failed")
        };

        assert!(ext.address() == original.address());
        assert!(ext.size() == split_size);
        assert!(*splitted.address() == *ext.address().aligned_add(split_size.get() as usize));
        assert!(splitted.size().get() == original.size().get() - split_size.get());
    }
}



#[test]
fn split() {

    use libtestrand::TestRng;

    let mut rand = TestRng::new();

    for _ in 0..2000 {

        let mut ext: MutableExtent<1024> = MutableExtent::new(
            AlignedNonNull::new_up(NonZero::new(rand.next_range(1..10240) as u64).unwrap()),
            NonZero::new(rand.next_range(1..10240) as u64).unwrap());
        let original = ext.clone();

        let split_size = NonZero::new(rand.next_range(1..ext.size().get() as usize * 2) as u64)
            .unwrap();

        let splitted = match split_size.cmp(&ext.size()) {
            Less => match ext.split(split_size) {
                Ok(Some(ext)) => ext,
                Ok(None) => panic!("split produced Ok(None) instead of Ok(Some())"),
                Err(_) => panic!("split failed when it should have succeeded"),
            },
            Equal => match ext.split(split_size) {
                Ok(None) => continue,   //  expected
                Ok(Some(_)) => panic!("split produced Ok(Some()) instead of Ok(None)"),
                Err(_) => panic!("split failed when it should have succeeded"),
            },
            Greater => {
                assert!(matches!(ext.split(split_size), Err(())));
                continue
            },
        };

        assert!(ext.address() == original.address());
        assert!(ext.size() == split_size);
        assert!(*splitted.address() == *ext.address().aligned_add(split_size.get() as usize));
        assert!(splitted.size().get() == original.size().get() - split_size.get());
    }

}
