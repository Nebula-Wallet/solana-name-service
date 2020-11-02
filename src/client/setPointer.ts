import {
  Connection,
  PublicKey,
  Account,
  Transaction,
  TransactionInstruction
} from '@solana/web3.js'
import { sendAndConfirmTransaction } from './util/send-and-confirm-transaction'

export async function setPointer(
  connection: Connection,
  ourAccount: Account,
  pointerProgramAddress: PublicKey,
  pointerAddress: PublicKey,
  value: PublicKey
) {
  const instruction = new TransactionInstruction({
    keys: [{ pubkey: pointerAddress, isSigner: false, isWritable: true }],
    programId: pointerProgramAddress,
    data: value.toBuffer()
  })
  await sendAndConfirmTransaction(
    'Create pointer',
    connection,
    new Transaction().add(instruction),
    ourAccount
  )
}
