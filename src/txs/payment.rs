use crate::objects::amount::Amount;
use xrpl_async_macroses::BinarySerialize;
use crate::address::{AccountPublicKey, Address};
use crate::txs::{Transaction, TransactionSerializer};
use crate::types::Hash;

// FIXME: We have not all required fields from https://xrpl.org/transaction-common-fields.html
#[derive(BinarySerialize)]
pub struct PaymentTransaction {
    #[binary(id = "TransactionType")]
    pub transaction_type: i16,
    #[binary(id = "Account")]
    pub account: Address,
    #[binary(id = "Amount")]
    pub amount: Amount,
    #[binary(id = "Destination")]
    pub destination: Address,
    #[binary(id = "DestinationTag")]
    pub destination_tag: Option<u32>,
    #[binary(id = "InvoiceID")]
    pub invoice_id: Option<Hash<32>>,
    // TODO: Add `Paths`
    // #[binary(id = "Paths")]
    // pub paths: Option<Hash<32>>,
    #[binary(id = "SendMax")]
    pub send_max: Option<Amount>,
    #[binary(id = "DeliverMin")]
    pub deliver_min: Option<Amount>,
    #[binary(id = "TxnSignature")]
    pub signature: Option<Vec<u8>>,
    #[binary(id = "SigningPubKey")]
    pub public_key: Option<AccountPublicKey>,
}

#[cfg(test)]
mod tests {
    use xrpl::core::keypairs::derive_keypair;
    use crate::address::{Address, Encoding};
    use crate::objects::amount::Amount;
    use crate::txs::payment::PaymentTransaction;
    use crate::txs::sign_transaction;

    fn test_serialize() {
        let our_address = Address::decode("rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh").unwrap();
        let our_seed = "sEdTWjtgXkxfh2p4KrTyDzmKu8aYNnK";
        let (public_key, private_key) = derive_keypair(our_seed, false).unwrap(); // TODO: ineffective!
        let (public_key, private_key) =
            (hex::decode(public_key).unwrap(), hex::decode(private_key).unwrap());
        let private_key = &private_key[1..33];
        let tx = PaymentTransaction {
            amount: Amount {
                value: 10.0,
                currency: "USD".to_string(),
                issuer: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
            },
            destination: our_address,
            destination_tag: None,
            invoice_id: None,
            send_max: None,
            deliver_min: None,
            signature: None,
            public_key: None,
        };
        let tx = sign_transaction(tx, Encoding(public_key.as_slice().try_into().unwrap()), private_key);
        // assert_eq!(tx.signature, ""); // FIXME
    }
}