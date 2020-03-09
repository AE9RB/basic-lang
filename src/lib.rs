//! # BASIC
//!
//! Should not see this. Documentation is in bin crate.
//!

extern crate proc_macro;
use proc_macro::*;

/*
This provides #[derive(EnumIter)] which implements
YourEnum::iter() to return all variants that don't have data.
*/

#[doc(hidden)]
#[proc_macro_derive(EnumFieldLess)]
pub fn derive_enum_iter(item: TokenStream) -> TokenStream {
    let mut scan = item.into_iter();
    let mut enum_name: Option<String> = None;
    loop {
        if let Some(t) = scan.next() {
            if let TokenTree::Ident(t) = t {
                if t.to_string() == "pub" {
                    continue;
                } else if t.to_string() == "enum" {
                    if let Some(t) = scan.next() {
                        if let TokenTree::Ident(t) = t {
                            enum_name = Some(t.to_string());
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let enum_name = match enum_name {
        None => panic!("Not a supported enum"),
        Some(name) => name,
    };
    let mut values: Vec<String> = Vec::new();

    if let Some(t) = scan.next() {
        if let TokenTree::Group(t) = t {
            t.stream().into_iter().for_each(|x| match x {
                TokenTree::Ident(t) => values.push(t.to_string()),
                TokenTree::Group(_) => {
                    // exclude variants with data
                    values.pop();
                }
                _ => {}
            })
        } else {
            panic!("Generic enums are not supported")
        }
    }

    let mut ts = TokenStream::new();
    let mut code = String::new();

    code.push_str(&format!("impl {} {{ ", enum_name));

    code.push_str(&format!(
        "fn field_less() -> ::std::vec::Vec<{}> {{ vec![\n",
        enum_name
    ));
    values
        .iter()
        .for_each(|v| code.push_str(&format!("{}::{},\n", enum_name, v)));
    code.push_str("] } }\n");

    ts.extend(code.parse::<TokenStream>());
    ts
}
