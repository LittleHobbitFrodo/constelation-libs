//! Provides the `TestRng` - a pseudo random number generator used only for testing

use alloc::string::String;

use core::{ops::{Bound, Range, RangeBounds}, sync::atomic::Ordering};

use crate::SEXTANT_SEED;


//#[cfg(feature = "testing")]
#[derive(Clone, Debug)]
/// Simple pseudo random number generator made for unit testing and (maybe) fuzzing
/// - This is **NOT** cryptographically safe RNG
pub struct TestRng;

//#[cfg(feature = "testing")]
impl TestRng {

    #[cfg(feature = "testing")]
    /// Constructs the `TestRng` and initializes the global seed
    pub fn new() -> Self {
        unsafe { super::__sextant_randomization_hook(&SEXTANT_SEED) }
        Self
    }

    #[cfg(not(feature = "testing"))]
    /// Constructs the `TestRng` and initializes the global seed
    pub fn new() -> Self {
        panic!("TestRng can only be used when feature \"testing\" is enabled");
    }


    /// Returns next random number
    pub fn next(&mut self) -> usize {

        let mut z = {
            SEXTANT_SEED.fetch_add(0x9E3779B97F4A7C15, Ordering::AcqRel)
            + 0x9E3779B97F4A7C15
            //self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
            //self.state

        };

        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);

        let result = z ^ (z >> 31);
        _ = SEXTANT_SEED.fetch_xor(result, Ordering::AcqRel);
        result
    }

    #[inline]
    /// Returns next random number within given range
    pub fn next_range<X: RangeBounds<usize>>(&mut self, range: X) -> usize {

        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Excluded(start) => start + 1,
            Bound::Included(start) => *start,
        };

        let end = match range.end_bound() {
            Bound::Unbounded => usize::MAX,
            Bound::Excluded(end) => *end,
            Bound::Included(end) => *end + 1,
        };

        let dif = end.saturating_sub(start);

        //  catch division by zero
        if dif == 0 {
            0
        } else {
            (self.next() % dif) + start
        }
    }


    /// Generates random printable unicode character
    pub fn next_char(&mut self) -> char {

        loop {
            const RANGES: &[(usize, usize)] = &[
                (0x20, 0x7E),           // ASCII printable
                (0xA0, 0x024F),         // Latin extended
                (0x0370, 0x052F),       // Greek + Cyrillic
                (0x25A1, 0x2603),       // 3-byte chars - shapes + misc
                (0x1F600, 0x1F64F),     // 4-byte chars - emojis
            ];

            let (start, end) = RANGES[self.next_range(..RANGES.len()-1)];
            let num = self.next_range(start as usize..=end as usize) as u32;

            if let Some(ch) = char::from_u32(num) {
                return ch
            }
        }
    }


    /// Generates a random `String` and appends the result into the given `string`
    pub fn string(&mut self, string: &mut String, len: usize) {

        string.reserve(len.into());

        for _ in 0..len {

            string.push(self.next_char());
        }
    }

    /// Generates a new random `String`
    #[inline(always)]
    pub fn new_string(&mut self, len: usize) -> String {
        let mut string = String::new();
        self.string(&mut string, len);
        string
    }

    /// Returns random element from the given slice
    #[inline]
    pub fn from_slice<'l, T>(&'_ mut self, slice: &'l [T]) -> &'l T {
        &slice[self.next_range(..slice.len())]
    }

    #[inline]
    /// Returns next random number as `isize`
    pub fn next_signed(&mut self) -> isize {
        usize::cast_signed(self.next())
    }

}
