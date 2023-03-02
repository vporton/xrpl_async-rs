use xrpl::core::types::Amount;
use xrpl_async_macroses::BinarySerialize;

#[derive(BinarySerialize)]
struct PaymentTransaction {
    #[binary(id = "Amount")]
    pub amount: Amount,
}