use proc_macro::TokenStream;
use syn::ItemFn;

use quote::quote;

pub mod util;
pub use util::{get_group_content, HarnessUsage};
use util::*;


pub const TEST_LINKER_SECTION: &str = "__MINISTD_TEST_LINKER_SECTION_";


pub fn testing(attr: TokenStream, input: TokenStream) -> TokenStream {

    //  Parse attributes (#[testing(<attributes>)])
    let Attributes { category, harness, fuzzing } = match parse_attributes(attr.into()) {
        Ok(attr) => attr,
        Err(e) => return e.into(),
    };

    //  Parse the input function
    let mut test_fn = match syn::parse::<ItemFn>(input) {
        Ok(f) => f,
        Err(e) => return e.into_compile_error().into(),
    };

    //  Check function declaration
    if let Err(e) = check_function_declaration(&mut test_fn) {
        return e.into()
    }

    if let Err(e) = check_and_force_return_type(&mut test_fn.sig.output) {
        return e.into()
    }

    inject_test_tools_import(&mut test_fn);


    generate_unit_test(test_fn, category, harness, fuzzing).into()
}


pub fn generate_unit_test(mut test_fn: ItemFn, category: Option<String>, harness: HarnessUsage, fuzzing: Option<usize>) -> TokenStream {

    //todo!("#[testing] requires REDO");

    //  gather/convert data for the test generation
    let fn_name = test_fn.sig.ident.to_string();
    let fn_export_name = format!("__test_fn_{}", fn_name.as_str());
    let fn_ex_ident = syn::Ident::new(&fn_export_name, test_fn.sig.ident.span());
    let mod_name = syn::Ident::new(&format!("_ministd_test_mod_{}", fn_name.as_str()), test_fn.sig.ident.span());

    //  rename the function
    test_fn.sig.ident = syn::Ident::new(fn_export_name.as_str(), test_fn.sig.ident.span());
    let fn_name_ident = test_fn.sig.ident.clone();

    let category_ident = match category {
        Some(name) => quote! { Some(#name) },
        None => quote! { None }
    };

    //  generate code for the simple fuzzer
    let test_fn = match modify_fn_body(test_fn, fuzzing) {
        Ok(test_fn) => test_fn,
        Err(e) => return e.into()
    };

    //if shall_generate(harness) {
        println!("GENERATING");

        if cfg!(feature = "harness_build") {
            //  Generate tests to work with std
            // - Uses the inventory crate
            quote! {    //  generate code and register it in the inventory
                mod #mod_name {

                    use super::*;
                    use ministd::{tassert, tassert_eq, tassert_ne, fail, tests::TestError};

                    //  The actual Test instance
                    static TEST_MUTEX_INSTANCE: ministd::sync::Mutex<ministd::tests::Test>
                        = ministd::tests::Test::new_reference(
                            #fn_name,
                            #category_ident,
                            #fn_ex_ident,
                            ministd::tests::Location::new(file!(), line!(), column!())
                        );

                    //  register the test to the DB
                    ministd::tests::exposed::inventory::submit!(&TEST_MUTEX_INSTANCE);

                    #test_fn
                }

            }.into()
            /*} else {

            //  Generate tests to work without std
            //  - Create the Test instance and register the TestReference within the linker section
            quote! {
                mod #mod_name {

                    use super::*;
                    use ministd::{tassert, tassert_eq, tassert_ne, fail, tests::TestError};

                    //  The actual Test instance
                    static TEST_MUTEX_INSTANCE: ministd::sync::Mutex<ministd::tests::Test>
                        = ministd::tests::Test::new_reference(
                            #fn_name,
                            #category_ident,
                            #fn_ex_ident,
                            ministd::tests::Location::new(file!(), line!(), column!())
                        );

                    //  register the test to the DB
                    #[unsafe(link_section = #TEST_LINKER_SECTION)]
                    static TEST_REFERENCE: ministd::tests::TestReference = ministd::tests::TestReference::new(&TEST_MUTEX_INSTANCE);

                    #test_fn
                }
            }.into()
        }*/

    } else {
        quote! {    //  generate the function, but do not register it in inventory
            mod #mod_name {
                use super::*;
                use ministd::{tassert, tassert_eq, tassert_ne, fail, tests::TestError};

                struct TestDiscard();
                impl TestDiscard {
                    const fn new(_: fn()-> Result<(), TestError>) -> Self {
                        Self ()
                    }
                }

                //  Use the function, but do not register it
                #[allow(dead_code)]
                static discard: TestDiscard = TestDiscard::new(#fn_name_ident);
                #test_fn
            }

        }.into()
    }

}
