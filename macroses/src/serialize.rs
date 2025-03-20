use std::collections::HashMap;
use std::{path::Path, env};
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use itertools::Itertools;
use syn::MetaNameValue;
use syn::{Attribute, Data::Struct, Fields::Named, AttrStyle, Lit, Meta, parse2};
use serde::{Deserialize, Deserializer};
use lazy_static::lazy_static;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FieldInfo {
    nth: i16,
    #[serde(rename = "isVLEncoded")]
    is_vl_encoded: bool,
    #[serde(rename = "isSerialized")]
    is_serialized: bool,
    #[serde(rename = "isSigningField")]
    is_signing_field: bool,
    r#type: String,
}

#[derive(Debug)]
struct Definitions {
    pub types: HashMap<String, i16>,
    #[allow(dead_code)]
    pub ledger_entry_types: HashMap<String, i16>,
    pub fields: HashMap<(String, String), FieldInfo>, // (Name, Type) -> FieldInfo
}

lazy_static! {
    static ref DEFINITIONS: Definitions = {
        let path = Path::new(&env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var missing"))
            .join("definitions.json");
        let file = std::fs::File::open(path).expect("Can't open definitions.json");
        serde_json::from_reader(file).expect("Can't read file")
    };
}

impl<'de> Deserialize<'de> for Definitions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        struct Definitions2 {
            #[serde(rename = "TYPES")]
            pub types: HashMap<String, i16>,
            #[serde(rename = "LEDGER_ENTRY_TYPES")]
            pub ledger_entry_types: HashMap<String, i16>,
            #[serde(rename = "FIELDS")]
            pub fields: Vec<(String, FieldInfo)>,
        }
        let value: Definitions2 = Definitions2::deserialize(deserializer)?;
        Ok(Self {
            types: value.types,
            ledger_entry_types: value.ledger_entry_types,
            fields: value.fields.into_iter().map(|(k, v)| ((k, v.r#type.clone()), v)).into_iter().collect(),
        })
    }
}

pub(crate) fn impl_serialize(ast: &syn::DeriveInput) -> TokenStream {
    let Struct(s) = &ast.data else {
        panic!("derive(BinarySerialize) applied not to a struct.")
    };
    let Named(fields) = &s.fields else {
        panic!("derive(BinarySerialize) works only with named fields.")
    };
    let fields_data = (&fields.named).into_iter().map(|field| -> Option<_> {
        for attr in &field.attrs {
            if let AttrStyle::Outer = attr.style {
                if let Meta::List(list) = &attr.meta {
                    if list.path.is_ident("binary") {
                        let meta: Vec<_> = list.tokens.clone().into_iter().collect();
                        if meta.iter().find(|t| if let TokenTree::Ident(id) = t {
                            id.to_string() == "skip"
                        } else {
                            false
                        }).is_some() {
                            return None;
                        }
                        let kvs: Vec<_> = meta.iter().filter_map(|t| 
                            if let Ok(meta) = parse2::<MetaNameValue>(t.into_token_stream()) { // FIXME: Check for more values in the stream.
                                if meta.path.is_ident("id") {
                                    Some(meta)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        ).collect();
                        if kvs.len() > 1 {
                            panic!("Must be no more than one binary(id)");
                        }
                        let kvs2: Vec<_> = meta.iter().filter_map(|t| 
                            if let Ok(meta) = parse2::<MetaNameValue>(t.into_token_stream()) { // FIXME: Check for more values in the stream.
                                if meta.path.is_ident("rtype") { // FIXME: `type` or `rtype`?
                                    Some(meta)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        ).collect();
                        if kvs2.len() != 1 {
                            panic!("Must be exactly one binary(type)");
                        }
                        let id = if let Some(pair) = kvs.first() {
                            if let syn::Expr::Lit(expr_lit) = &pair.value {
                                if let Lit::Str(lit) = &expr_lit.lit {
                                    lit.value()
                                } else {
                                    panic!("binary(rtype) must be a string.")
                                }
                            } else {
                                panic!("binary(rtype) must be a literal.")
                            }
                        } else {
                            field.ident.as_ref().unwrap().to_string()
                        };
                        let r#type = if let Some(pair) = kvs2.first() {
                            if let syn::Expr::Lit(expr_lit) = &pair.value {
                                if let Lit::Str(lit) = &expr_lit.lit {
                                    lit.value()
                                } else {
                                    panic!("binary(id) must be a string.")
                                }
                            } else {
                                panic!("binary(id) must be a literal.")
                            }
                        } else {
                            field.ident.as_ref().unwrap().to_string()
                        };
                        let field_info = &DEFINITIONS.fields[&(id, r#type.clone())];
                        let type_code = DEFINITIONS.types[&r#type]; // a little inefficient because of string index
                        return Some((type_code, field_info.nth, &field.ident));
                    }
                }
            }
        }
        panic!("No #[binary] attribute for field {:?}", field.ident);
    });
    let fields_data = fields_data.flatten()
        .sorted_by(|a, b| Ord::cmp(&(a.0, a.1), &(b.0, b.1)));
    let body = fields_data.map(|field| {
        let (type_code, nth, field_name) = field;
        quote!(
            crate::serialize::XrplBinaryField {
                xrpl_type: &crate::serialize::XrplType {
                    type_code: #type_code,
                },
                field_code: #nth,
                value: &self.#field_name,
            }.serialize(writer)?;
        )
    });
    let body = proc_macro2::TokenStream::from_iter(body);

    let struct_name = &ast.ident;
    quote!(
        impl Transaction for #struct_name {
            fn set_signature(&mut self, signature: Vec<u8>) {
                self.signature = Some(signature);
            }
            fn set_public_key(&mut self, public_key: AccountPublicKey) {
                self.public_key = Some(public_key);
            }
        }
        impl TransactionSerializer for #struct_name {
            fn serialize(&self, prefix: &[u8; 4], writer: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                use crate::serialize::Serialize; // TODO: needed?
                writer.write_all(prefix)?;
                #body
                Ok(())
            }
        }
    ).into()
}