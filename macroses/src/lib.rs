mod serialize;

extern crate proc_macro;
extern crate core;

use syn::{DeriveInput, parse_macro_input};
use proc_macro::TokenStream;
use crate::serialize::impl_serialize;
// use crate::serialize::impl_serialize;

/// ```
/// #[derive(BinarySerialize)]
/// struct Transaction {
///     #[binary(id = "Account")]
///     account: Address,
///     #[binary(skip)]
///     x: Address,
///     // ...
/// }
/// ```
/// WARNING: It serializes as unsigned transaction.
#[proc_macro_derive(BinarySerialize, attributes(binary))]
pub fn binary_serialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_serialize(&ast)
}