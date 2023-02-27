mod serialize;

extern crate proc_macro;
// #[macro_use]
// extern crate quote;
use syn::{Data::{Struct}, DeriveInput, Fields::{self, Named}, AttrStyle, Meta, MetaList, parse_macro_input};
use proc_macro::TokenStream;
use quote::quote;
// use crate::serialize::impl_serialize;

/// ```
/// #[derive(BinarySerialize)]
/// struct Transaction {
///     #[field(nth = 1)]
///     account: Address,
///     // ...
/// }
/// ```
#[proc_macro_derive(BinarySerialize)]
pub fn serialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    if let Struct(s) = ast.data {
        if let Named(fields) = s.fields {
            for field in fields.named {
                for attr in field.attrs {
                    if let AttrStyle::Outer = attr.style {
                        if let Ok(Meta::List(MetaList { path, paren_token, nested })) = attr.parse_meta() {
                            if path.is_ident("field") {
                                // FIXME
                            }
                        }
                    }
                }
            }
        } else {
            panic!("derive(Serialize) works only with named fields.")
        }
    } else {
        panic!("derive(Serialize) applied not to a struct.")
    }

    quote!(ast).into()
}