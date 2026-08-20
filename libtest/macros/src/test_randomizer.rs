use syn::{ItemFn, ReturnType, Type};
use proc_macro::TokenStream;



pub fn check_signature(f: &ItemFn) -> Result<(), TokenStream> {
    if f.sig.abi.is_some() {
        return Err(error!(&f.sig.abi, "must not be extern"))
    }

    if f.sig.asyncness.is_some() {
        return Err(error!(&f.sig.asyncness, "must not be async"))
    }

    if f.sig.constness.is_some() {
        return Err(error!(&f.sig.constness, "must not be const"))
    }

    if f.sig.generics.gt_token.is_some() {
        return Err(error!(&f.sig.generics, "must not take any generics"))
    }

    if !f.sig.inputs.is_empty() {
        return Err(error!(&f.sig.inputs, "must not take any inputs"))
    }

    if f.sig.unsafety.is_some() {
        return Err(error!(&f.sig.unsafety, "must not be unsafe"))
    }

    if f.sig.variadic.is_some() {
        return Err(error!(&f.sig.variadic, "must not take variadic parameter"))
    }
    Ok(())
}


pub fn check_output(f: &ItemFn) -> Result<(), TokenStream> {

    macro_rules! fail {
        () => { return Err(error!(&f.sig.output, "must return `Result<usize, ()>`")) };
    }

    let return_type = match &f.sig.output {
        ReturnType::Type(_, ty) => ty,
        _ => fail!(),
    };

    if let Type::Path(type_path) = &**return_type {
        let segments = &type_path.path.segments;

        if segments.len() == 1 && segments[0].ident == "Result" {
            if let syn::PathArguments::AngleBracketed(args) = &segments[0].arguments {

                //  make sure the first arg is usize
                if let Some(arg) = args.args.first() {
                    match arg {
                        syn::GenericArgument::Type(Type::Path(p)) => if !p.path.is_ident("usize") {
                            return Err(error!(arg, "must return Result<usize, ()>"))
                        },
                        _ => return Err(error!(arg, "must return Result<usize, ()>")),
                    }
                } else {
                    return Err(error!(args, "must return Result<usize, ()>"))
                }


                //  make sure the second is ()
                if let Some(arg) = args.args.get(1) {
                    match arg {
                        syn::GenericArgument::Type(Type::Tuple(t)) => if !t.elems.is_empty() {
                            return Err(error!(arg, "must return Result<usize, ()>"))
                        },
                        _ => return Err(error!(arg, "must return Result<usize, ()>")),
                    }
                } else {
                    return Err(error!(args, "must return Result<usize, ()>"))
                }
            } else {
                fail!()
            }
        } else {
            fail!()
        }

    } else {
        fail!()
    }


    Ok(())
}