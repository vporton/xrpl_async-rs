use serde_json::Value;
use crate::connection::WrongFieldsError;
use crate::types::Hash;

pub(crate) trait ValueExt {
    fn as_bool_valid(&self) -> Result<bool, WrongFieldsError>;
    fn as_str_valid(&self) -> Result<&str, WrongFieldsError>;
    fn as_u64_valid(&self) -> Result<u64, WrongFieldsError>;
    fn as_u32_valid(&self) -> Result<u32, WrongFieldsError>;
    fn as_hash_valid(&self) -> Result<Hash, WrongFieldsError>;
}

impl ValueExt for Value {
    fn as_bool_valid(&self) -> Result<bool, WrongFieldsError> {
        Ok(self.as_bool().ok_or(WrongFieldsError::new())?)
    }
    fn as_str_valid(&self) -> Result<&str, WrongFieldsError> {
        Ok(self.as_str().ok_or(WrongFieldsError::new())?)
    }
    fn as_u64_valid(&self) -> Result<u64, WrongFieldsError> {
        Ok(self.as_u64().ok_or(WrongFieldsError::new())?)
    }
    fn as_u32_valid(&self) -> Result<u32, WrongFieldsError> {
        Ok(self.as_u64_valid()?.try_into().map_err(|_| WrongFieldsError::new())?)
    }
    fn as_hash_valid(&self) -> Result<Hash, WrongFieldsError> {
        Ok(Hash::from_hex(self.as_str_valid()?).map_err(|_| WrongFieldsError::new())?)
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