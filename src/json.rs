use serde_json::Value;
use crate::address::Address;
use crate::response::WrongFieldsError;
use crate::types::{decode_xrp_amount, Hash};

pub(crate) trait ValueExt {
    fn get_valid(&self, key: &str) -> Result<&Value, WrongFieldsError>;
    fn as_bool_valid(&self) -> Result<bool, WrongFieldsError>;
    fn as_str_valid(&self) -> Result<&str, WrongFieldsError>;
    fn as_f64_valid(&self) -> Result<f64, WrongFieldsError>;
    fn as_u64_valid(&self) -> Result<u64, WrongFieldsError>;
    fn as_u32_valid(&self) -> Result<u32, WrongFieldsError>;
    fn as_array_valid(&self) -> Result<&Vec<Value>, WrongFieldsError>;
    fn as_hash_valid(&self) -> Result<Hash, WrongFieldsError>;
    fn as_address_valid(&self) -> Result<Address, WrongFieldsError>;
    fn as_xrp_valid(&self) -> Result<u64, WrongFieldsError>;
}

impl ValueExt for Value {
    fn get_valid(&self, key: &str) -> Result<&Value, WrongFieldsError> {
        self.get(key).ok_or(WrongFieldsError::new())
    }
    fn as_bool_valid(&self) -> Result<bool, WrongFieldsError> {
        self.as_bool().ok_or(WrongFieldsError::new())
    }
    fn as_str_valid(&self) -> Result<&str, WrongFieldsError> {
        self.as_str().ok_or(WrongFieldsError::new())
    }
    fn as_f64_valid(&self) -> Result<f64, WrongFieldsError> {
        self.as_f64().ok_or(WrongFieldsError::new())
    }
    fn as_u64_valid(&self) -> Result<u64, WrongFieldsError> {
        self.as_u64().ok_or(WrongFieldsError::new())
    }
    fn as_u32_valid(&self) -> Result<u32, WrongFieldsError> {
        self.as_u64_valid()?.try_into().map_err(|_| WrongFieldsError::new())
    }
    fn as_array_valid(&self) -> Result<&Vec<Value>, WrongFieldsError> {
        self.as_array().ok_or(WrongFieldsError::new())
    }
    fn as_hash_valid(&self) -> Result<Hash, WrongFieldsError> {
        Hash::from_hex(self.as_str_valid()?).map_err(|_| WrongFieldsError::new())
    }
    fn as_address_valid(&self) -> Result<Address, WrongFieldsError> {
        Address::decode(self.as_str_valid()?).map_err(|_| WrongFieldsError::new())
    }
    fn as_xrp_valid(&self) -> Result<u64, WrongFieldsError> {
        decode_xrp_amount(self.as_str_valid()?).map_err(|_| WrongFieldsError::new())
    }
}

// impl ValueExt for Option<&Value> {
//     fn as_bool_valid(&self) -> Result<bool, WrongFieldsError> {
//         self.ok_or(WrongFieldsError::new())?.as_bool_valid()
//     }
//     fn as_str_valid(&self) -> Result<&str, WrongFieldsError> {
//         self.ok_or(WrongFieldsError::new())?.as_str_valid()
//     }
// }