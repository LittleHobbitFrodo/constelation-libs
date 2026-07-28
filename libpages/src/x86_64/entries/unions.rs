use crate::{Either, PageEntry, PageEntryUnion, PdRegularEntry, PdSizedEntry, PdptRegularEntry, PdptSizedEntry};





//  TODO: const KERNEL: bool generics


macro_rules! generate_union_entry {
    ($entry_name:ident, $union_name:ident, $reg_variant:ident, $sized_variant:ident) => {


        /// Pd table entry - an union distingushing between regular and page-sized entry
        #[repr(transparent)]
        pub struct $entry_name(pub(crate) $union_name);

        pub(crate) union $union_name {
            pub(crate) raw: u64,
            pub(crate) reg: $reg_variant,
            pub(crate) sized: $sized_variant,
        }

        impl Clone for $union_name {
            fn clone(&self) -> Self {
                Self {
                    raw: unsafe { self.raw }
                }
            }
        }

        impl PageEntry for $entry_name {}
        impl PageEntryUnion for $entry_name {}


        impl $entry_name {


            pub fn new_regular(entry: $reg_variant) -> Option<Self> {
                if entry.is_executable() && entry.is_writeable() {
                    core::hint::cold_path();
                    None
                } else {
                    Some(Self($union_name { reg: entry }))
                }
            }

            pub fn new_sized(entry: $sized_variant) -> Option<Self> {
                if entry.is_executable() && entry.is_writeable() {
                    core::hint::cold_path();
                    None
                } else {
                    Some(Self($union_name { sized: entry }))
                }
            }

            /// Indicates whether the entry is page-sized
            #[inline]
            pub fn is_page_sized(&self) -> bool {
                unsafe { self.0.raw & $sized_variant::BITS_PS != 0 }
            }

            /// Indicates whether the entry is present in memory
            #[inline]
            pub fn is_present(&self) -> bool {
                unsafe { self.0.raw & $reg_variant::BITS_PRESENT != 0 }
            }


            /// Checks the page size bit and reinterprets itself as regular xor sized entry
            pub fn entry(&self) -> Either<&$reg_variant, &$sized_variant> {
                if self.is_page_sized() {
                    Either::Regular(unsafe { &self.0.reg })
                } else {
                    Either::Sized(unsafe { &self.0.sized })
                }
            }

            /// Checks the page size bit and loads corresponding entry
            pub fn entry_copied(&self) -> Either<$reg_variant, $sized_variant> {
                let bits = unsafe { self.0.raw };

                if bits & $sized_variant::BITS_PS == 0 {
                    Either::Regular($reg_variant(bits))
                } else {
                    Either::Sized($sized_variant(bits))
                }
            }

            /// Reinterprets itself as regular entry
            pub fn as_regular_entry(&self) -> Option<&$reg_variant> {
                if self.is_page_sized() {
                    None
                } else {
                    Some(unsafe { &self.0.reg })
                }
            }

            /// Loads and reinterprets itself as regular entry
            pub fn regular_entry_copied(&self) -> Option<$reg_variant> {
                let bits = unsafe { self.0.raw };

                if bits & $sized_variant::BITS_PS == 0 {
                    Some($reg_variant(bits))
                } else {
                    None
                }
            }

            /// Reinterprets itself as page-sized entry
            pub fn as_sized_entry(&self) -> Option<&$sized_variant> {
                if self.is_page_sized() {
                    Some(unsafe { &self.0.sized })
                } else {
                    None
                }
            }

            /// Loads and reinterprets itself as page-sized entry
            pub fn sized_entry_copied(&self) -> Option<$sized_variant> {
                let bits = unsafe { self.0.raw };

                if bits & $sized_variant::BITS_PS != 0 {
                    Some($sized_variant(bits))
                } else {
                    None
                }
            }

            /// Creates new entry directly from given bits
            ///
            /// # Safety
            /// This function is safe on its own, but libpages has its own way
            /// of doing things and it does not expect any different approach
            /// - Calling this function may result in undefined behaviour
            pub const unsafe fn new_raw(bits: u64) -> Self {
                Self($union_name { raw: bits })
            }

        }


    };
}


generate_union_entry!(PdEntry, PdEntryUnion, PdRegularEntry, PdSizedEntry);
generate_union_entry!(PdptEntry, PdptEntryUnion, PdptRegularEntry, PdptSizedEntry);
