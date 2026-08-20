//#![no_std]

//  TODO: REDO oom


/// Returns a proper error
macro_rules! error {
    ($tokens:expr, $msg:expr) => {{
        syn::Error::new_spanned(
            $tokens,
            $msg,
        ).to_compile_error()
        .into()
    }};
}



use std::collections::btree_map::ValuesMut;
use std::str::FromStr;

use proc_macro::*;

use proc_macro2::{Delimiter, Span};
use quote::quote;
use syn::token::{Extern, Token};
use syn::{
    Abi, Expr, ExprReference, ExprReturn, FnArg, Item, ItemFn, LitStr, Meta, Pat, Path, ReturnType, Type, parse_macro_input, parse_quote, parse2};

/*use syn::{parse_quote_spanned, Attribute, Error, Expr, ExprLit, ExprPath, ItemStatic, ItemStruct, Lit, Visibility, ExprMacro};*/

use syn::{PatType, TypeReference, TypePath, Signature, GenericArgument};
use syn::{visit_mut::VisitMut, Stmt};

//use crate::testing::{HarnessUsage, get_group_content};


mod testing;
mod stdout_handler;
mod test_randomizer;
mod test_only;


/// `#[testing]` marks functions dedicated for unit testing
///
/// Tests can be collected by using the `ministd::tests::collect()` function
///
/// ## General usage
/// ```rust
/// #[testing]
/// fn some_test() {
///     //  test code ...
///     return  //  on success
/// }
/// ```
///
/// ## Test categories
/// You can categorize tests by simply adding the category name into the attributes:
/// ```rust
/// #[testing("category-1")]
/// fn categorized_test() {
///     //  test code ...
/// }
/// ```
///
/// All tests will then be grouped by category and sorted by file and line where they were introduced
///
/// ## Harness
/// Using the `harness(...)` attribute makes possible to run the test on your local machine
/// - This requires additional actions, for example your project must be able to be built as library and then linked to the mnistd harness
///   - **TODO**: Documentation
/// - Tests running within harness must be suitable for running under an operating system
///
/// ```rust
/// #[testing(harness(always))]
/// fn only_in_emulator() {
///     //  test code ...
/// }
/// ```
/// Valid `harness` attribute options:
/// 1. `never`: forbids running in the harness
/// 2. `any`: runs in the harness and/or in emulator
/// 3. `always`: runs only in the harness
///
/// **The `never` option is used by default**
///
/// ## Fuzzing
/// The `fuzz(N)` attribute takes care of the boilerplate code for your simple fuzzer!
///
///  When in use, the fuzzer generates code that:
/// 1. Initializes `ministd::TestRng`
/// 2. Creates a `for` loop to properly fuzz the test
/// 3. Adds your test code into the loop
///
/// The code is generated as follows:
/// ```rust
/// #[testing(fuzz)]
/// fn some_test() {
///     let mut rand = TestRng::new()
///         .expect("failed to initialize TestRng");
///
///     for i in 0..1000 {
///         //  Your code...
///     }
/// }
///
/// #[testing(fuzz(556))]
/// fn some_other_test() {
///     let mut rand = TestRng::new()
///         .expect("failed to initialize TestRng");
///
///     for i in 0..556 {
///         //  Your code
///     }
/// }
/// ```
///
/// ### Parameter
/// `fuzz(N)` takes one parameter: an unsigned integer. If specified, the fuzzer loop will iterate exactly `N` times
/// - If `N` is not specified, it will default to **1000**
///
/// ## Lets look under the hood
/// All tests marked as `#[testing]` can be executed only if the `ministd` is built with the `testing` feature enabled
/// - Enable the `harness_tests` to build for tests within the harness
///   - This may also disable certain functions of the `ministd`
#[proc_macro_attribute]
pub fn testing(attr: TokenStream, input: TokenStream) -> TokenStream {
    testing::testing(attr, input)
}

#[proc_macro_attribute]
pub fn stdout_handler(attr: TokenStream, input: TokenStream) -> TokenStream {

    #[cfg(feature = "disable_stdout_pipe")] {
        return TokenStream::new()
    }

    #[cfg(not(feature = "disable_stdout_pipe"))] {

        use stdout_handler::*;

        let func = parse_macro_input!(input as ItemFn);

        let sig = &func.sig;

        if let Err(e) = check_signature(&func.sig) {
            return e
        }


        if !matches!(sig.output, ReturnType::Default) {
            return error!(&sig.output, "stdout handler never returns an value");
        }

        let attr: proc_macro2::TokenStream = attr.into();

        quote! {
            #attr
            #[unsafe(export_name="__ministd_stdout_pipe_handler")]
            extern "Rust" #func
        }.into()
    }

}


#[proc_macro_attribute]
pub fn test_randomizer(_: TokenStream, input: TokenStream) -> TokenStream {

    use test_randomizer::*;

    let f = parse_macro_input!(input as ItemFn);

    if let Err(e) = check_signature(&f) {
        return e
    }

    if let Err(e) = check_output(&f) {
        return e
    }


    //  transform the function
    quote! {
        #[unsafe(no_mangle)]
        #[unsafe(export_name = "__ministd_test_randomizer")]
        extern "Rust" #f
    }.into()
}
