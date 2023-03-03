use crate::objects::amount::Amount;
use xrpl_async_macroses::BinarySerialize;
// use crate::address::{AccountPublicKey, Address};
use crate::txs::{Transaction, TransactionSerializer};
use crate::types::Hash;
use crate::address::{AccountPublicKey, Address};

pub const TRANSACTION_TYPE_PAYMENT: i16 = 0;

// FIXME: We have not all required fields from https://xrpl.org/transaction-common-fields.html
#[derive(BinarySerialize, Clone)]
pub struct PaymentTransaction {
    #[binary(id = "TransactionType", rtype = "UInt16")]
    pub transaction_type: i16,
    #[binary(id = "Account", rtype = "AccountID")]
    pub account: Address,
    #[binary(id = "Amount", rtype = "Amount")]
    pub amount: Amount,
    #[binary(id = "Destination", rtype = "AccountID")]
    pub destination: Address,
    #[binary(id = "DestinationTag", rtype = "UInt32")]
    pub destination_tag: Option<u32>,
    #[binary(id = "InvoiceID", rtype = "Hash256")]
    pub invoice_id: Option<Hash<32>>,
    // TODO: Add `Paths`
    // #[binary(id = "Paths")]
    // pub paths: Option<Hash<32>>,
    #[binary(id = "SendMax", rtype = "Amount")]
    pub send_max: Option<Amount>,
    #[binary(id = "DeliverMin", rtype = "Amount")]
    pub deliver_min: Option<Amount>,
    #[binary(id = "TxnSignature", rtype = "Blob")]
    pub signature: Option<Vec<u8>>,
    #[binary(id = "SigningPubKey", rtype = "Blob")]
    pub public_key: Option<AccountPublicKey>,
}

#[cfg(test)]
mod tests {
    use xrpl::core::keypairs::derive_keypair;
    use xrpl_binary_codec::serializer::{HASH_PREFIX_TRANSACTION, HASH_PREFIX_UNSIGNED_TRANSACTION_SINGLE};
    use crate::address::{Address, Encoding};
    use crate::objects::amount::Amount;
    use crate::txs::payment::{PaymentTransaction, TRANSACTION_TYPE_PAYMENT};
    use crate::txs::sign_transaction;
    use crate::txs::TransactionSerializer;

    #[test]
    fn test_serialize() {
        let our_address = Address::decode("rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh").unwrap();
        let our_seed = "sEdTWjtgXkxfh2p4KrTyDzmKu8aYNnK";
        let (public_key, private_key) = derive_keypair(our_seed, false).unwrap(); // TODO: ineffective!
        let (public_key, private_key) =
            (hex::decode(public_key).unwrap(), hex::decode(private_key).unwrap());
        assert_eq!(public_key, hex::decode( "EDC5248F3F06990D2E694C83AF55C45206ACD4AABC1151020600ECD6B75A5FF628").unwrap());
        assert_eq!(private_key, hex::decode("EDE8249E957A8A50AF727C78B214F7192FD69F72E5EEC105FB1D838D46D26F06B5").unwrap());
        assert_eq!(our_address, Address::decode("rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh").unwrap());
        let private_key = &private_key[1..];
        let tx = PaymentTransaction {
            transaction_type: TRANSACTION_TYPE_PAYMENT,
            account: our_address.clone(),
            amount: Amount {
                value: 10.0,
                currency: "USD".to_string(),
                issuer: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
            },
            destination: our_address,
            destination_tag: None,
            invoice_id: None,
            send_max: Some(Amount {
                value: 10.0,
                currency: "USD".to_string(),
                issuer: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
            }),
            deliver_min: None,
            signature: None,
            public_key: None,
        };
        // let mut tx = tx;
        // tx.set_public_key(Encoding(*(<&[u8; 33]>::try_from(public_key.as_slice()).unwrap())));
        let mut ser = Vec::new();
        PaymentTransaction::serialize(&tx, &HASH_PREFIX_UNSIGNED_TRANSACTION_SINGLE, &mut ser).unwrap(); // TODO: `unwrap`
        assert_eq!(ser.as_slice(), hex::decode("5354580012000061D4C38D7EA4C6800000000000000000000000000055534400000000004B4E9C06F24296074F7BC48F92A97916C6DC5EA969D4C38D7EA4C6800000000000000000000000000055534400000000004B4E9C06F24296074F7BC48F92A97916C6DC5EA981147CCFE86388B264396710C29F69025DB1DFA4AE4C83147CCFE86388B264396710C29F69025DB1DFA4AE4C").unwrap());
        let mut tx = sign_transaction(tx, Encoding(public_key.as_slice().try_into().unwrap()), private_key);
        tx.signature = None;
        let mut ser = Vec::new();
        PaymentTransaction::serialize(&tx, &HASH_PREFIX_TRANSACTION, &mut ser).unwrap(); // TODO: `unwrap`
        // Apparently, in JavaScript's serialization SigningPubKey is missing
        // assert_eq!(ser, hex::decode("5354580012000061D4C38D7EA4C6800000000000000000000000000055534400000000004B4E9C06F24296074F7BC48F92A97916C6DC5EA969D4C38D7EA4C6800000000000000000000000000055534400000000004B4E9C06F24296074F7BC48F92A97916C6DC5EA97321EDC5248F3F06990D2E694C83AF55C45206ACD4AABC1151020600ECD6B75A5FF62881147CCFE86388B264396710C29F69025DB1DFA4AE4C83147CCFE86388B264396710C29F69025DB1DFA4AE4C").unwrap());
    }
}