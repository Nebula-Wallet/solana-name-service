import {
  Transaction,
  SystemProgram,
  TransactionInstruction,
  PublicKey,
  encodeData,
  Account
} from '@solana/web3.js'
import { getOurAccount } from './ourAccount'
import { getNodeConnection } from './nodeConnection'
import { makeAccount } from './deploy'
import { getStore } from './storeConfig'
import { sendAndConfirmTransaction } from './util/send-and-confirm-transaction'
import { createToken } from './createToken'
import { airDrop } from './util/air-drop'

const main = async () => {
  const ourAccount = await getOurAccount()
  const connection = await getNodeConnection()
  const store = await getStore(connection, 'proxy-pointer.json')
  console.log('Create account and airdrop 10 sol')
  const newAccount = new Account()
  await airDrop(newAccount, connection)
  console.log(
    `Created account: ${newAccount.publicKey.toString()} balance: ${await connection.getBalance(
      newAccount.publicKey
    )}`
  )

  // Listen for new registration
  const nullAccount = '11111111111111111111111111111111'
  connection.onProgramAccountChange(store.programId, (accountInfo, { slot }) => {
    const data = accountInfo.accountInfo.data
    const key = new PublicKey(data.slice(0, 32))
    if (key.toString() !== nullAccount) {
      // trim empty
      // console.log(msg.replace(/\0/g, ''))
      console.log(`Created Pointer to -> ${key}`)
      process.exit()
    }
  })
  const storageAccount = await makeAccount(connection, newAccount, 33, store.programId)

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: storageAccount, isSigner: false, isWritable: true }
    ],
    programId: store.programId,
    data: newAccount.publicKey.toBuffer()
  })
  await sendAndConfirmTransaction(
    'Create pointer',
    connection,
    new Transaction().add(instruction),
    newAccount
  )
}
main()
