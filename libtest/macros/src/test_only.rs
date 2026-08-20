

use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenTree};

use crate::testing::HarnessUsage;


pub fn get_harness_usage(attr: TokenStream) -> Result<HarnessUsage, TokenStream> {
    let attr: proc_macro2::TokenStream = attr.into();

    let mut iter = attr.into_iter();

    if let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) if ident == "harness" => {
                if let Some(TokenTree::Group(group)) = iter.next() {
                    if group.delimiter() != Delimiter::Parenthesis {
                        return Err(error!(&group, "harness(..) attribute expects its value to be enclosed in parenthesis: ()"));
                    }

                    let val = match group.stream().into_iter().next() {
                        Some(val) => val,
                        None => return Err(error!(group, "harness(...) attribute expects value/ident")),
                    };

                    if let TokenTree::Literal(lit) = val {
                        HarnessUsage::from_str(&lit.to_string())
                        .map_err(|_| error!(&lit, format!("harness(...) expects one of following values: `never`, `always`, `any`")) )
                    } else {
                        return Err(error!(&val, format!("harness(...) expects one of following values: `never`, `always`, `any`")))
                    }

                } else {
                    return Err(error!(&ident, "harness(...) expects either one value or nothing"));
                }
            },
            unknown => return Err(error!(&unknown, format!("expected `harness(...)` attribute or nothing, got {unknown}"))),
        }
    } else {
        Ok(HarnessUsage::Emul)
    }
}