use proc_macro::TokenStream;
use syn::{FnArg, Pat, Signature, Type, TypePath, TypeReference};



pub fn check_signature(sig: &Signature) -> Result<(), TokenStream> {

    if let Some(_) = sig.abi {
        return Err(error!(&sig.abi, "cannot have any explicit ABI"));
    }

    if let Some(_) = sig.asyncness {
        return Err(error!(&sig.asyncness, "cannot be async"));
    }

    if let Some(_) = sig.constness {
        return Err(error!(&sig.constness, "cannot be const"));
    }

    if !sig.generics.params.is_empty() {    //  TODO: check
        return Err(error!(&sig.generics.params, "cannot take any generic parameters"));
    }

    if sig.inputs.len() != 1 {
        return Err(error!(&sig.inputs, "always takes one parameter: `&'_ str`"));
    }

    //  check inputs
    match sig.inputs.first().unwrap() {
        FnArg::Typed(pat_type) => {
            match &*pat_type.ty {
                // Must be a reference: &...
                Type::Reference(TypeReference {
                    elem,
                    ..
                }) => {
                    match &**elem {
                        // Inner type must be `str`
                        Type::Path(TypePath { path, .. }) => {
                            if ! path.is_ident("str") {
                                //input_panic!("must be reference to `str`")
                                return Err(error!(&path, "must be reference to `str`"))
                            }
                        }
                        _ => return Err(error!(&elem, "must be reference to `str`"))
                        //input_panic!("must be reference to `str`"),
                    }
                }
                _ => return Err(error!(&pat_type, "must be a reference"))
                //input_panic!("must be a reference"),
            }
        },
        t => return Err(error!(&t, "must be a reference"))
        //input_panic!(),
    }

    Ok(())

}
