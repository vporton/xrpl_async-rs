import {encodeForSigning, Payment} from 'xrpl'
import { Amount } from 'xrpl/dist/npm/models/common'

const amount: Amount = {
    currency: 'USD',
    value: '10',
    issuer: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
}

const tx: Payment = {
    TransactionType: 'Payment',
    Account: "rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh",
    Amount: amount,
    Destination: "rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh",
    SendMax: amount,
    SigningPubKey: "EDC5248F3F06990D2E694C83AF55C45206ACD4AABC1151020600ECD6B75A5FF628",
}
console.log(encodeForSigning(tx))
const tx2: Payment = { TxnSignature: encodeForSigning(tx), ...tx }
console.log(encodeForSigning(tx2))
console.log(tx2)