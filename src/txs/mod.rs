use xrpl_binary_codec::sign::sign;
use xrpl_binary_codec::serializer::HASH_PREFIX_UNSIGNED_TRANSACTION_SINGLE;
use crate::address::AccountPublicKey;


pub mod payment;

pub trait Transaction: TransactionSerializer {
    fn set_signature(&mut self, signature: Vec<u8>);
    fn set_public_key(&mut self, public_key: AccountPublicKey);
}

pub trait TransactionSerializer {
    fn serialize(&self, prefix: &[u8; 4], writer: &mut dyn ::std::io::Write) -> ::std::io::Result<()>;
}

// FIXME: special type for secret key
pub fn sign_transaction<T: Transaction>(tx: T, public_key: AccountPublicKey, secret_key: &[u8]) -> T {
    let mut tx = tx;
    tx.set_public_key(public_key);
    let mut ser = Vec::new();
    T::serialize(&tx, &HASH_PREFIX_UNSIGNED_TRANSACTION_SINGLE, &mut ser).unwrap(); // TODO: `unwrap`
    let signature = sign(ser.as_slice(), secret_key);
    tx.set_signature(signature);
    tx
}