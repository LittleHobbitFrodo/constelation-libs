use alloc::string::String;

use core::ops::{Bound, Range, RangeBounds};
#[derive(Clone, Debug)]
/// Simple pseudo random number generator made for unit testing and (maybe) fuzzing
/// - This is **NOT** cryptographically safe RNG
pub struct TestRng<R: FnMut() -> Result<usize, ()>> {
    state: usize,
    f: R,
}

/// Helper macro that makes sure both `Rng` and `TestRng` works exactly the same
macro_rules! generate_worker_code {
    () => {
        #[inline]
    /// Returns next random number
    pub fn next(&mut self) -> usize {

        let mut z = {
            self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
            self.state
        };

        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
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
    };
}

impl<R: FnMut() -> Result<usize, ()>> TestRng<R> {

    /// Constructs new uninitialized `Rng` with state equal to zero
    pub const fn new_uninit(randomizer: R) -> Self {
        Self { state: 0, f: randomizer }
    }

    /// Constructs new uninitialized `Rng` with randomizer
    pub const fn new_uninit_with(randomizer: R) -> Self {
        Self { state: 0, f: randomizer }
    }

    #[inline]
    /// Creates new `Rng` and initializes it by given randomizer
    pub fn new(mut randomizer: R) -> Result<Self, ()> {
        Ok(Self {
            state: randomizer()?,
            f: randomizer
        })
    }

    #[inline]
    /// Initializes the `Rng` by using the randomizer closure
    pub fn initialize(&mut self) -> Result<(), ()> {
        self.state = (self.f)()?;
        Ok(())
    }

    #[inline]
    /// Initializes the `Rng` by using the given closure
    pub fn initialize_with<X: FnMut() -> Result<usize, ()>>(&mut self, mut f: X) -> Result<(), ()> {
        self.state = f()?;
        Ok(())
    }

    #[inline]
    /// Randomizes the generator by xorring current state with value returned by the randomizer
    pub fn randomize(&mut self) -> Result<(), ()> {
        self.state ^= (self.f)()?;
        Ok(())
    }

    #[inline]
    /// Randomizes the `Rng` by xorring the state with value returned by the given randomizer
    pub fn randomize_with<X: FnMut() -> Result<usize, ()>>(&mut self, mut f: X) -> Result<(), ()> {
        self.state ^= f()?;
        Ok(())
    }

    generate_worker_code!();

}
