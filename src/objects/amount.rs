use crate::hashes::Address;

#[derive(Clone)]
pub struct Amount {
    pub value: f64,
    pub currency: String,
    pub issuer: Address,
}