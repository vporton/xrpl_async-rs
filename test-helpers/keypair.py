from xrpl.wallet import Wallet
from xrpl.constants import CryptoAlgorithm
import xrpl.transaction
import xrpl.core.binarycodec
from xrpl.models.transactions import Payment
from xrpl.models.amounts import IssuedCurrencyAmount

wallet = Wallet("sEdTWjtgXkxfh2p4KrTyDzmKu8aYNnK", 0, algorithm=CryptoAlgorithm.ED25519)
print(wallet.public_key)
print(wallet.private_key)
print(wallet.classic_address)

amount = IssuedCurrencyAmount(currency='USD', value=10, issuer="rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn")

tx = Payment(
    account="rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh",
    amount=amount,
    destination="rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh",
    send_max=amount,
)

# d = xrpl.transaction.transaction_json_to_binary_codec_form(tx)
print(xrpl.core.binarycodec.encode(tx))
# print(xrpl.core.binarycodec.encode(d))

print(xrpl.transaction.safe_sign_transaction(tx, wallet).txn_signature)