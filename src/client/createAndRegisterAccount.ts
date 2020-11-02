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
import { airDrop } from './util/air-drop'

const main = async () => {
  const ourAccount = await getOurAccount()

  const connection = await getNodeConnection()
  const store = await getStore(connection, 'account-name-service.json')
  const storeProxy = await getStore(connection, 'proxy-pointer.json')
  const counterStore = await getStore(connection, 'counter.json')

  console.log(counterStore.accountId.toString())
  const counterAccount = counterStore.accountId
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
    const msg = data.slice(32, 64).toString('utf8')
    const is_initialized = data.slice(64, 65)
    const counter = data.slice(65, 73)
    if (key.toString() !== nullAccount) {
      // trim empty
      console.log(is_initialized)
      console.log(counter)
      console.log(counter.readUInt32LE(0) + counter.readUInt32LE(4) * 2 ** 32)
      // console.log(msg.replace(/\0/g, ''))
      console.log(`Somebody registered address -> ${key} under name -> ${msg}`)
      process.exit()
    }
  })
  const storageAccount = await makeAccount(connection, newAccount, 73, store.programId)
  await sendAndConfirmTransaction(
    'transfer 1 SOL',
    connection,
    new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: newAccount.publicKey,
        toPubkey: storageAccount,
        lamports: 1 * 1e9
      })
    ),
    newAccount
  )

  const name = Buffer.alloc(32)
  name.write('test name user')
  const instruction_data = Buffer.concat([newAccount.publicKey.toBuffer(), name]) // 64 bytes
  console.log(instruction_data.length)
  const instruction = new TransactionInstruction({
    keys: [
      // This account must match one in smartcontract
      { pubkey: ourAccount.publicKey, isSigner: false, isWritable: true },
      // This account must match one in smartcontract
      {
        pubkey: storeProxy.accountId,
        isSigner: false,
        isWritable: true
      },
      // This account must match one in smartcontract
      { pubkey: counterAccount, isSigner: false, isWritable: true },
      { pubkey: storageAccount, isSigner: false, isWritable: true }
    ],
    programId: store.programId,
    data: instruction_data
  })
  console.log('a')
  await sendAndConfirmTransaction(
    'register user',
    connection,
    new Transaction().add(instruction),
    newAccount
  )
}
main()
