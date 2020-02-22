extern crate proc_macro;
use proc_macro::*;

/*
This provides #[derive(EnumIter)] which implements
YourEnum::iter() to return all variants that don't have data.
*/

#[proc_macro_derive(EnumIter)]
pub fn derive_enum_iter(item: TokenStream) -> TokenStream {
    let mut scan = item.clone().into_iter();
    let mut enum_name: Option<String> = None;
    let mut is_pub = false;
    loop {
        if let Some(t) = scan.next() {
            if let TokenTree::Ident(t) = t {
                if t.to_string() == "pub" {
                    is_pub = true;
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
    if enum_name == None {
        panic!("Not a supported enum");
    }

    let enum_name = enum_name.unwrap();
    let mut vals: Vec<String> = Vec::new();

    if let Some(t) = scan.next() {
        if let TokenTree::Group(t) = t {
            t.stream().clone().into_iter().for_each(|x| match x {
                TokenTree::Ident(t) => vals.push(t.to_string()),
                TokenTree::Group(_) => {
                    // exclude variants with data
                    vals.pop();
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

    if is_pub {
        code.push_str("pub ");
    }

    code.push_str(&format!(
        "fn iter() -> ::std::slice::Iter<'static, {}> {{ [\n",
        enum_name
    ));
    vals.iter()
        .for_each(|v| code.push_str(&format!("{}::{},\n", enum_name, v)));
    code.push_str("].iter() } }\n");

    ts.extend(code.parse::<TokenStream>());
    ts
}
