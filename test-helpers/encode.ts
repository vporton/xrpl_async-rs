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
}
console.log(encodeForSigning(tx))