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
  const store = await getStore(connection, 'account-name-service.json')
  const storeProxy = await getStore(connection, 'proxy-pointer.json')
  const counterAccount = 'ENdBRErsGnqKp3gRBMj9rU1wQXXFnhda7WNvhJAeEER2'
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
    if (key.toString() !== nullAccount) {
      // trim empty
      console.log(data)
      // console.log(msg.replace(/\0/g, ''))
      console.log(`Somebody registered token -> ${key} under name -> ${msg}`)
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
  const tokenId = await createToken(
    connection,
    newAccount,
    9,
    undefined,
    newAccount.publicKey.toString()
  )
  const name = Buffer.alloc(32)
  name.write('test name user')
  const instruction_data = Buffer.concat([newAccount.publicKey.toBuffer(), name])
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
      { pubkey: new PublicKey(counterAccount), isSigner: false, isWritable: true },
      { pubkey: storageAccount, isSigner: false, isWritable: true }
    ],
    programId: store.programId,
    data: instruction_data
  })
  await sendAndConfirmTransaction(
    'register token',
    connection,
    new Transaction().add(instruction),
    newAccount
  )
}
main()
