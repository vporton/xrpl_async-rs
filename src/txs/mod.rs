use crate::address::AccountPublicKey;

pub mod payment;

pub trait Transaction: TransactionSerializer {
    fn set_signature(&mut self, signature: Vec<u8>);
    fn set_public_key(&mut self, public_key: AccountPublicKey);
}

pub trait TransactionSerializer {
    fn serialize(&self, prefix: &[u8; 4], writer: &mut dyn ::std::io::Write) -> ::std::io::Result<()>;
}