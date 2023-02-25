use serde::{de, Deserialize, Deserializer};

pub struct SignerListFlags(u32);
// TODO: Values of flags: https://xrpl.org/signerlist.html#signerlist-flags

impl<'de> Deserialize<'de> for SignerListFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(SignerListFlags(u32::deserialize(deserializer)?.try_into().map_err(de::Error::custom)?))
    }
}

#[derive(Debug, Deserialize)]
pub struct SignerList {
    // FIXME: more fields
}