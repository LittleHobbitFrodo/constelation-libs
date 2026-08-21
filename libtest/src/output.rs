

unsafe extern "Rust" {
    /// Gives the output to sextant harness
    pub(crate) fn __sextant_output_receiver(s: &'_ str);
}


#[macro_export]
macro_rules! print {
    (($arg:tt)*) => {

    };
}
