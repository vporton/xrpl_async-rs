use crate::objects::amount::Amount;
use xrpl_async_macroses::BinarySerialize;
use crate::address::{AccountPublicKey, Address};
use crate::txs::{Transaction, TransactionSerializer};
use crate::types::Hash;

#[derive(BinarySerialize)]
struct PaymentTransaction {
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