use proc_macro::TokenStream;

use quote::quote;
use syn::{Ident, ItemFn, Safety, spanned::Spanned};

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

#[proc_macro_attribute]
pub fn testing(attr: TokenStream, input: TokenStream) -> TokenStream {

    let test_fn = match syn::parse::<ItemFn>(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into()
    };

    if let Err(e) = check_signature(&test_fn) {
        return e
    }

    generate_code(test_fn)
}


#[cfg(feature = "testing")]
fn generate_code(test_fn: ItemFn) -> TokenStream {

    let fn_name = test_fn.sig.ident.to_string();
    let fn_name_ident = test_fn.sig.ident.clone();

    dbg!(&fn_name);
    let mod_name = Ident::new(format!("__test_fn_{fn_name}").as_ref(), test_fn.span());

    quote! {
        mod #mod_name {

            use libtest::exposed::*;

            #test_fn

            static TEST_CASE: TestCase = TestCase::new(#fn_name_ident, #fn_name, Location::new(file!(), line!()));

            inventory::submit!(&TEST_CASE);
        }
    }.into()

}

#[cfg(not(feature = "testing"))]
fn generate_code(test_fn: ItemFn) -> TokenStream {
    quote! {
        #[cfg()]
        #test_fn
    }.into()
}

/// Checks function signature
/// - Aborts if not ok
fn check_signature(test_fn: &ItemFn) -> Result<(), TokenStream> {

    let sig = &test_fn.sig;

    if let Some(_) = sig.abi {
        return Err(error!(&sig.abi, "cannot have explicit ABI"));
    }

    if let Some(_) = sig.asyncness {
        return Err(error!(&sig.asyncness, "cannot be async"));
    }

    if let Some(_) = sig.constness {
        return Err(error!(&sig.constness, "cannot be const"));
    }

    if sig.generics.const_params().count() != 0 {
        return Err(error!(&sig.generics, "cannot have any generic parameters"));
    }

    if !sig.inputs.is_empty() {
        return Err(error!(&sig.inputs, "cannot expect any parameters"));
    }

    match sig.safety {
        Safety::Safe(_) | Safety::Default => { /*OK*/ },
        Safety::Unsafe(_) => return Err(error!(&sig.safety, "cannot be unsafe")),
    }

    if let Some(_) = sig.variadic {
        return Err(error!(&sig.variadic, "cannot be variadic"));
    }

    if let Some(_) = sig.receiver() {
        return Err(error!(sig.receiver(), "cannot have receiver"));
    }

    Ok(())

}
