mod serialize;

extern crate proc_macro;
extern crate core;

// #[macro_use]
// extern crate quote;
use syn::{Data::{Struct}, DeriveInput, Fields::Named, AttrStyle, Lit, Meta, MetaList, NestedMeta,
          parse_macro_input};
use proc_macro::TokenStream;
use quote::quote;
// use crate::serialize::impl_serialize;

/// ```
/// #[derive(BinarySerialize)]
/// struct Transaction {
///     #[binary(nth = 1)]
///     account: Address,
///     // ...
/// }
/// ```
#[proc_macro_derive(BinarySerialize)]
pub fn serialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let Struct(s) = ast.data else {
        panic!("derive(Serialize) applied not to a struct.")
    };
    let Named(fields) = s.fields else {
        panic!("derive(Serialize) works only with named fields.")
    };
    // TODO: better error checking
    let fields_data = fields.named.into_iter().map(|field| -> syn::parse::Result<_> { // TODO: Need `Result`?
        for attr in field.attrs {
            if let AttrStyle::Outer = attr.style {
                if let Ok(Meta::List(MetaList { path, paren_token: _, nested })) = attr.parse_meta() {
                    if path.is_ident("binary") {
                        for kv in nested.iter() {
                            if let NestedMeta::Meta(Meta::NameValue(kv)) = kv {
                                if kv.path.is_ident("skip") {
                                    return Ok(None);
                                } else if kv.path.is_ident("nth") {
                                        let Lit::Int(lit) = &kv.lit else {
                                        panic!("binary(nth) must be an integer.")
                                    };
                                    let nth = lit.base10_parse::<u16>()?;
                                    return Ok(Some((field.ident, field.ty, nth)));
                                }
                            }
                        }
                        panic!("No binary(nth)");
                    }
                }
            }
        }
        Ok(None) // TODO: or `panic!`?
    });
    let fields_data = fields_data.collect::<Result<Vec<_>, _>>().expect("TODO error")
        .into_iter().flatten();

    let struct_name = ast.ident;
    quote!(
        impl Serialize for BinaryFormat<'a, #struct_name> {
            fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {

            }
        }
    ).into()
}