use core::{fmt::Debug, mem::MaybeUninit};

#[cfg(any(feature = "testing", feature = "harness"))]
pub use inventory;

mod test_case;
pub use test_case::*;

#[cfg(any(feature = "testing", feature = "harness"))]
/// Returns the iterator over all testcases
pub fn test_iterator() -> inventory:: iter<&'static TestCase> {
    inventory::iter::<&'static TestCase>::iter
}


/// Collects the output from all tests
/// and hands it over to the harness
pub struct OutputCollector;


impl core::fmt::Write for OutputCollector {

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let mut slice = [0u8; 4];
        let s = c.encode_utf8(&mut slice);

        unsafe {
            crate::output::__sextant_output_receiver(s);
        }
        Ok(())
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            crate::output::__sextant_output_receiver(s);
        }

        Ok(())
    }

}
