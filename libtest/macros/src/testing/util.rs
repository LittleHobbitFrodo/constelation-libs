

use core::{arch, error};
use std::{any::type_name, str::FromStr};

pub const FUZZ_ITER_DEFAULT: usize = 1000;

//use syn::{Meta, NestedMeta, Lit};
use proc_macro2::{TokenStream, TokenTree, Literal};
use syn::{Block, Expr, LitStr, Stmt};

use proc_macro2::Span;
use syn::{GenericArgument, Ident, ItemFn, Path, PathArguments, PathSegment, ReturnType, Signature, Type, TypePath, Visibility, parse_quote, spanned::Spanned};
use quote::{ToTokens, TokenStreamExt, quote};


pub fn return_type() -> proc_macro2::TokenStream {
    quote! {core::result::Result<(), core::option::Option<&'static core::primitive::str>>}
}
pub const RETURN_TYPE_STR: &str = "Result<(), TestError>";


macro_rules! generic_err {
    ($token:expr, $msg:literal) => {
        //panic!("must return {} ({}) |line {}|", RETURN_TYPE_STR, $msg, line!())
        return Err(error!($token, $msg))
    };
    () => { panic!("must return {} |line {}|", RETURN_TYPE_STR, line!()) };
}


pub struct Attributes {
    /// Contains category name if specified
    pub category: Option<String>,
    /// Indicates whether a harness can be used
    pub harness: HarnessUsage,
    /// Indicates if the test is set as fuzzer
    /// - The number inside of `Some` indicates iteration count
    pub fuzzing: Option<usize>,
}

/// Parses proc_macro attributes, returns `(category_name, harness_build)`
pub fn parse_attributes(attr: TokenStream) -> Result<Attributes, TokenStream> {


    let mut category = None;
    let mut harness = HarnessUsage::Emul;
    let mut fuzzing = None;
    let mut found_harness_group = false;

    let mut iter = attr.into_iter();
    'attr: loop {  //  for token in attr.into_iter();

        let token = match iter.next() {
            Some(tok) => tok,
            None => break,
        };

        match &token {
            //  resolve fuzzer
            TokenTree::Ident(ident) if ident == "fuzz" => {
                if let Some(_) = fuzzing {  //  found the fuzz statement
                    return Err(error!(ident, "conflicting `fuzz()` directives"))
                }

                if let Some(TokenTree::Group(group)) = iter.next() {
                    if group.delimiter() == proc_macro2::Delimiter::Parenthesis {

                        let lit = match group.stream().into_iter().next() {
                            Some(lit) => lit,
                            None => return Err(error!(ident, "fuzz(...) attribute expects an unsigned integer (iteration count)")),
                        };

                        //  enforce uint
                        if let TokenTree::Literal(lit) = lit {
                            let as_string = lit.to_string();
                            fuzzing = Some(match usize::from_str(&as_string) {
                                Ok(n) => n,
                                Err(_) => return Err(error!(lit, "fuzz(...) attribute expects an unsigned integer (iteration count)")),
                            });
                        } else {
                            return Err(error!(lit, "fuzz(...) attribute expects an unsigned integer (iteration count)"));
                        }

                    } else {
                        return Err(error!(group, "fuzz(...) attribute expects an unsigned integer (iteration count)"));
                    }
                } else {    //  found only `fuzz` instead of `fuzz(n)`
                    fuzzing = Some(FUZZ_ITER_DEFAULT);
                    continue 'attr
                }
            },

            //  resolve harness
            TokenTree::Ident(ident) if ident == "harness" => {
                if found_harness_group {
                    return Err(error!(token, "conflicting `harness(...)` directives"))
                }
                found_harness_group = true;
                if let Some(TokenTree::Group(group)) = iter.next() {
                    if group.delimiter() == proc_macro2::Delimiter::Parenthesis {

                        let lit = match group.stream().into_iter().next() {
                            Some(lit) => lit,
                            None => return Err(error!(ident, "harness(...) attribute expects one of these options: never, always or any")),
                        };

                        //  enforce Ident
                        if let TokenTree::Ident(lit) = lit {
                            harness = match HarnessUsage::from_str(&lit.to_string()) {
                                Ok(usage) => usage,
                                Err(_) => return Err(error!(lit, "harness(...) attribute expects one of these options: never, any or always")),
                            };
                        } else {
                            return Err(error!(lit, "harness(...) attribute expects options in form of idents (simply text, not string)"));
                        }

                    } else {
                        return Err(error!(group, "harness(...) attribute expects options in form of idents (simply text, not string)"));
                    }
                } else {
                    return Err(error!(ident, "harness(...) attribute expects one of these options: never, always or any"))
                }
            },

            //  resolve category
            TokenTree::Literal(lit) => {

                if let Some(_) = category {
                    return Err(error!(lit, "found conflicting category names"))
                }

                let name = match syn::parse_str::<LitStr>(&lit.to_string()) {
                    Ok(name) => name.value(),
                    Err(_) => return Err(error!(lit, "macro internal error: failed to parse LitStr")),
                };

                let mut result: Result<(), TokenStream> = Ok(());
                name.chars().for_each(|c| if !(c.is_alphanumeric() || c == ' ' || c == '-') {
                    result = Err(error!(lit, "category names must be alphanumerical"));
                } );

                result?;

                category = Some(name)
            }
            TokenTree::Punct(_) => continue,
            token => {
                return Err(error!(token, "expected either the `harness(...)` directive or category name"))
            },
        }
    }

    Ok(Attributes { category, harness, fuzzing })

}

/// Checks group content
/// - returns `None` if empty
/// - panics if does not contain exactly one string
pub fn get_group_content(name: &'_ str, attr: TokenStream) -> Result<Option<String>, TokenStream> {


    let mut iter = attr.into_iter();
    loop {  //  for token in attr.into_iter();

        let token = match iter.next() {
            Some(tok) => tok,
            None => break,
        };

        match token {
            TokenTree::Ident(ident) if ident == name => {
                if let Some(TokenTree::Group(group)) = iter.next() {
                    if group.delimiter() == proc_macro2::Delimiter::Parenthesis {

                        let lit = match group.stream().into_iter().next() {
                            Some(lit) => lit,
                            None => return Err(error!(group, format!("{name}(...) attribute expects either one value, or nothing")))
                        };

                        if let TokenTree::Literal(lit) = lit {
                            let value: LitStr = match syn::parse_str(&lit.to_string()) {
                                Ok(val) => val,
                                Err(_) => return Err(error!(lit, format!("failed to parse {name}(...) attribute: expected value/ident"))),
                            };
                            return Ok(Some(value.value()))
                        } else {
                            return Err(error!(lit, format!("{name}(...) attribute expects either one value, or nothing")));
                        }

                    } else {
                        return Err(error!(group, format!("{name}(...) attribute expects either one value, or nothing")));
                    }
                } else {
                    return Err(error!(ident, format!("{name}(...) attribute expects either one value, or nothing")))
                }
            },
            token => {
                return Err(error!(&token, format!("expected either the `{name}(...)` attribute or category name, got \"{token}\"")))
            },
        }
    }

    Ok(None)
}

#[derive(Debug, Copy, Clone)]
pub enum HarnessUsage {
    /// Corresponds to `always`
    Harness,
    /// Corresponds to `never`
    Emul,
    /// Corresponds to `any`s
    Any,
}

impl FromStr for HarnessUsage {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(HarnessUsage::Emul),
            "any" => Ok(HarnessUsage::Any),
            "always" => Ok(HarnessUsage::Harness),
            _ => Err(())
        }
    }
}


/// Checks and modifies the return type
/// - All test functions have zero parameters
pub fn check_function_declaration(test_fn: &mut ItemFn) -> Result<(), TokenStream> {

    if !test_fn.attrs.is_empty() {
        return Err(error!(&test_fn.sig, "no additional attributes may be used on testing function"));
    }

    test_fn.vis = Visibility::Inherited;

    check_signature(&mut test_fn.sig)?;

    Ok(())

}

fn check_signature(sig: &Signature) -> Result<(), TokenStream> {

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

    if let Some(_) = sig.unsafety {
        return Err(error!(&sig.unsafety, "cannot be unsafe"));
    }

    if let Some(_) = sig.variadic {
        return Err(error!(&sig.variadic, "cannot be variadic"));
    }

    if let Some(_) = sig.receiver() {
        return Err(error!(sig.receiver(), "cannot have receiver"));
    }

    Ok(())

}

/// Checks if the return type is default (no returned value) and forces it to `Result<(), TestError>`
pub fn check_and_force_return_type(rt: &mut ReturnType) -> Result<(), TokenStream> {

    if matches!(rt, ReturnType::Default) {
        *rt = syn::parse_quote! { -> Result<(), ministd::tests::TestError> };
    } else {
        return Err(error!(rt, "test functions cannot return a value"))
    }

    Ok(())
}

#[deprecated = "older versions require the user to declare `fn test() -> Result<(), TestError>`"]
/// checks return type
pub fn check_return_type<const MINISTD: bool>(rt: &mut ReturnType) -> Result<(), TokenStream> {

    let g = get_generics(rt)?;

    let PathArguments::AngleBracketed(generics) = g else {
        generic_err!()
    };


    if generics.args.len() != 2 { generic_err!() }

    let GenericArgument::Type(tp) = generics.args.first().unwrap_or_else(|| generic_err!() ) else {
        generic_err!()
    };

    //  check Ok(())
    if let Type::Tuple(inner) = tp {
        if !inner.elems.is_empty() {
            generic_err!(&inner.elems, "empty tuple is expected for the Ok variant")
        }
    } else {
        generic_err!(&tp, "empty tuple is expected for the Ok variant")
    }

    //  check Err(TestError)
    let GenericArgument::Type(tp) = generics.args.get_mut(1).unwrap_or_else(|| generic_err!() ) else {
        generic_err!()
    };

    if let Type::Path(path) = tp {

        if path.path.segments.len() != 1 {
            if path.path.segments.len() > 1 {
                generic_err!(&path.path, "do not try to resolve path to the TestError type for the Err variant")
            } else {
                generic_err!(&path.path, "expected the TestError type for the Err variant")
            }
        }

        if path.path.segments.first().unwrap().ident != "TestError" {
            generic_err!(&path.path, "do not try to resolve path to the TestError type for the Err variant")
        }


        //  add `ministd::test::` before the TestResult
        let prefix_segs: Path = if MINISTD {
            syn::parse_quote!(crate::tests)
        } else {
            syn::parse_quote!(ministd::tests)
        };

        for seg in prefix_segs.segments.into_iter().rev() {
            //dbg!(&seg.ident);
            path.path.segments.insert(0, seg);
        }

    } else {
        generic_err!()
    }


    return Ok(());



    fn get_generics(rt: &mut ReturnType) -> Result<&mut PathArguments, TokenStream> {

        let tp = if let ReturnType::Type(_, t) = rt {
            match &mut **t {
                Type::Path(path) => path,
                t => generic_err!(t, "expected TypePath"),
            }
        } else {
            generic_err!(rt, "got ()");
        };


        //  check for the Result enum
        if tp.path.segments.len() != 1 { generic_err!() }
        match tp.path.segments.last() {
            Some(last) => {
                if last.ident != "Result" {
                    generic_err!()
                }
            },
            None => generic_err!()
        }

        //  add `core::result::` before the Result
        let prefix_segs: Path = syn::parse_quote!(core::result);

        for seg in prefix_segs.segments.into_iter().rev() {
            //dbg!(&seg.ident);
            tp.path.segments.insert(0, seg);
        }


        if let Some(_) = tp.qself { generic_err!(&tp, "remove QSelf") }

        if let Some(_) = tp.path.leading_colon { generic_err!(tp.path.leading_colon, "cannot start with path separator") }

        if let None = tp.path.segments.last_mut() {
            generic_err!(&tp, "the result must have two generic arguments")
        }

        Ok(&mut tp.path.segments.last_mut().unwrap().arguments)

        /*if let Some(arg) = tp.path.segments.last_mut() {
            Ok(&mut arg.arguments)
        } else {
            generic_err!(&, "the result must have two generic arguments")
        }*/

    }

}

/// Adds use `crate`/`ministd` `::{error, fail};`
/// - for `ministd_testing`
pub fn inject_test_tools_import(test_fn: &mut ItemFn) {
    test_fn.block.stmts.insert(0, parse_quote!(use ministd::{error, fail};));
}

/// 2 = `"tests_enabled"` and `"harness_build"`
/// 1 = `"tests_enabled`
/// 0 = Do not generate
pub const fn generate_when(harness: HarnessUsage) -> u8 {

    #[cfg(feature = "tests_enabled")] {
        #[cfg(feature = "harness_build")] {
            2
        }
        #[cfg(not(feature = "harness_build"))] {
            2
        }
    }
    #[cfg(not(feature = "tests_enabled"))]
    0
}

/// Indicates whether the `testing` proc_macro should generate the code or not
pub const fn shall_generate(harness_build: HarnessUsage) -> bool {

    if !cfg!(feature = "tests_enabled") { return false }

    match harness_build {
        HarnessUsage::Any => true,
        HarnessUsage::Emul => cfg!(not(feature = "harness_build")),
        HarnessUsage::Harness => cfg!(feature = "harness_build")
    }

    /*#[cfg(feature = "tests_enabled")] {
        #[cfg(feature = "harness_build")] {
            match harness_build {
                HarnessUsage::Any | HarnessUsage::Harness => !MINISTD || (MINISTD && cfg!(feature = "test_ministd")),
                _ => false,
            }
        }

        #[cfg(not(feature = "harness_build"))] {
            match harness_build {
                HarnessUsage::Any | HarnessUsage::Emul => !MINISTD || (MINISTD && cfg!(feature = "test_ministd")),
                _ => false,
            }
        }
    }
    #[cfg(not(feature = "tests_enabled"))] {
        false
    }*/
}

/// Adds the `ministd::TestRng` instance initialization and the for loop to fuzz the test
pub fn modify_fn_body(mut test_fn: ItemFn, fuzzing: Option<usize>) -> Result<ItemFn, TokenStream> {

    //  detect "plain" return statements and convert them into return Ok(())
    for stmt in &mut test_fn.block.stmts {
        if let Stmt::Expr(Expr::Return(ret), _) = stmt {
            if ret.expr.is_none() {
                *stmt = syn::parse_quote! {
                    return Ok(());
                };
            }
        }
    }

     //  detect `return Ok(())` or `Ok(())`
    if contains_return_stmt_at_the_end(&test_fn.block) {
        test_fn.block.stmts.pop();
    }

    if let Some(n) = fuzzing {
        let original_block = test_fn.block;

        let new_block = quote! {
            {
                let mut rand = TestRng::new().expect("failed to initialize TestRng");
                for i in 0..#n {
                    #original_block
                }
            }
        };

        match syn::parse2(new_block) {
            Ok(block) => test_fn.block = block,
            Err(e) => {
                return Err(TokenStream::from(e.to_compile_error()))
            },
        }
    }

    if !contains_return_stmt_at_the_end(&test_fn.block) {
        //  add return statement if not specified

        //  add semicolon if the last statement does not include it
        if let Some(last) = test_fn.block.stmts.last_mut() {
            *last = match last {
                Stmt::Expr(expr, None) => {
                    syn::parse2(quote! { #expr; }).expect("failed to parse the expression")
                }
                _ => last.clone(),
            }
        }

        test_fn.block.stmts.push(syn::parse2(quote! { return Ok(()); } )
            .expect("failed to parse return statement"));
    }

    Ok(test_fn)

}



pub fn contains_return_stmt_at_the_end(block: &Box<Block>) -> bool {

    return if let Some(stmt) = block.stmts.last() {
        is_ok_unit_stmt(stmt)
    } else {
        false
    };


    fn is_ok_unit_stmt(stmt: &syn::Stmt) -> bool {
        match stmt {
            syn::Stmt::Expr(expr, _) => is_ok_unit_expr(expr),
            _ => false,
        }
    }

    fn is_ok_unit_expr(expr: &syn::Expr) -> bool {
        match expr {
            // return Ok(())
            syn::Expr::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    is_ok_call(expr)
                } else {
                    false
                }
            }

            // plain Ok(())
            _ => is_ok_call(expr),
        }
    }

    fn is_ok_call(expr: &syn::Expr) -> bool {
    if let syn::Expr::Call(call) = expr {
        // Ok(...)
        if let syn::Expr::Path(path) = &*call.func {
            if path.path.is_ident("Ok") {
                // must be exactly Ok(())
                if call.args.len() == 1 {
                    if let Some(arg) = call.args.first() {
                        return matches!(arg, syn::Expr::Tuple(t) if t.elems.is_empty());
                    }
                }
            }
        }
    }
    false
}
}
