use proc_macro::TokenStream;
use std::collections::HashMap;
use quote::quote;
use itertools::Itertools;
use syn::{Data::Struct, Fields::Named, AttrStyle, Lit, Meta, MetaList, NestedMeta};
use serde::{Deserialize, Deserializer};
use lazy_static::lazy_static;
use syn::__private::TokenStream2;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FieldInfo {
    nth: i16,
    #[serde(rename="isVLEncoded")]
    is_vl_encoded: bool,
    #[serde(rename="isSerialized")]
    is_serialized: bool,
    #[serde(rename="isSigningField")]
    is_signing_field: bool,
    r#type: String,
}

#[derive(Debug)]
struct Definitions {
    pub types: HashMap<String, i16>,
    #[allow(dead_code)]
    pub ledger_entry_types: HashMap<String, i16>,
    pub fields: HashMap<String, FieldInfo>,
}

lazy_static!{
    static ref DEFINITIONS: Definitions = {
        let file = std::fs::File::open("definitions.json").expect("Can't open definitions.json");
        serde_json::from_reader(file).expect("Can't read file")
    };
}

impl<'de> Deserialize<'de> for Definitions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        struct Definitions2 {
            #[serde(rename="TYPES")]
            pub types: HashMap<String, i16>,
            #[serde(rename="LEDGER_ENTRY_TYPES")]
            pub ledger_entry_types: HashMap<String, i16>,
            #[serde(rename="FIELDS")]
            pub fields: Vec<(String, FieldInfo)>,
        }
        let value: Definitions2 = Definitions2::deserialize(deserializer)?;
        Ok(Self {
            types: value.types,
            ledger_entry_types: value.ledger_entry_types,
            fields: value.fields.into_iter().collect(),
        })
    }
}

pub(crate) fn impl_serialize(ast: &syn::DeriveInput) -> TokenStream {
    let Struct(s) = &ast.data else {
        panic!("derive(Serialize) applied not to a struct.")
    };
    let Named(fields) = &s.fields else {
        panic!("derive(Serialize) works only with named fields.")
    };
    // TODO: better error checking
    let fields_data = (&fields.named).into_iter().map(|field| -> Option<_> {
        for attr in &field.attrs {
            if let AttrStyle::Outer = attr.style {
                if let Ok(Meta::List(MetaList { path, paren_token: _, nested })) = attr.parse_meta() {
                    if path.is_ident("binary") {
                        for kv in nested.iter() {
                            if let NestedMeta::Meta(Meta::NameValue(kv)) = kv {
                                if kv.path.is_ident("skip") {
                                    return None;
                                } else if kv.path.is_ident("id") {
                                    let Lit::Str(lit) = &kv.lit else {
                                        panic!("binary(id) must be a string.")
                                    };
                                    let id = lit.value();
                                    let field_info = &DEFINITIONS.fields[&id];
                                    let type_code = DEFINITIONS.types[&field_info.r#type]; // a little inefficient because of string index
                                    return Some((type_code, field_info.nth, id/*, field.ty*/));
                                }
                            }
                        }
                        panic!("No binary(nth)");
                    }
                }
            }
        }
        None // TODO: or `panic!`?
    });
    let fields_data = fields_data.flatten()
        .sorted_by(|a, b| Ord::cmp(&(a.0, a.1), &(b.0, b.1)));
    let body = fields_data.map(|field| {
        let type_code = field.0;
        let nth = field.1;
        quote!(
            BinaryFormat(XrplField {
                xrpl_type: XrplType {
                    type_code: #type_code,
                },
                field_code: #nth,
                value: self,
            })
                .serialize(writer);
        )
    });
    let body = TokenStream2::from_iter(body);

    let struct_name = &ast.ident;
    quote!(
        impl Serialize for BinaryFormat<'a, #struct_name> {
            fn serialize(&self, writer: &mut dyn Write) -> io::Result<()> {
                #body
            }
        }
    ).into()
}