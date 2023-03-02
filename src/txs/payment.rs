use crate::objects::amount::Amount;
use xrpl_async_macroses::BinarySerialize;
use crate::address::Address;

#[derive(BinarySerialize)]
struct PaymentTransaction {
    #[binary(id = "Amount")]
    pub amount: Amount,
    #[binary(id = "Destination")]
    pub destination: Address,
    #[binary(id = "DestinationTag")]
    pub destination_tag: u32,
    // FIXME: more fields
    // #[binary(id = "InvoiceID")]
    // pub invoice_id: Hash<32>,
}