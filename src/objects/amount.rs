use crate::address::Address;

pub struct Amount {
    pub value: f64,
    pub currency: String,
    pub issuer: Address,
}