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
import { getStore, setStore } from './storeConfig'
import { sendAndConfirmTransaction } from './util/send-and-confirm-transaction'
import { createToken } from './createToken'
import { airDrop } from './util/air-drop'

const main = async () => {
  const ourAccount = await getOurAccount()
  console.log('####')
  const connection = await getNodeConnection()
  const store = await getStore(connection, 'account-name-service.json')
  const storeProxy = await getStore(connection, 'proxy-pointer.json')

  // Listen for new registration
  const nullAccount = '11111111111111111111111111111111'
  connection.onProgramAccountChange(storeProxy.programId, (accountInfo, { slot }) => {
    const data = accountInfo.accountInfo.data
    const key = new PublicKey(data.slice(0, 32))
    if (key.toString() !== nullAccount) {
      // trim empty
      // console.log(msg.replace(/\0/g, ''))
      console.log(`Created Pointer to -> ${key}`)
      
      process.exit()
    }
  })
  const counterAccount = await makeAccount(connection, ourAccount, 8, store.programId)
  await setStore('counter.json', counterAccount, counterAccount)
  console.log(`Created counter on address ${counterAccount.toString()}`)
  const instruction = new TransactionInstruction({
    keys: [{ pubkey: storeProxy.accountId, isSigner: false, isWritable: true }],
    programId: storeProxy.programId,
    data: counterAccount.toBuffer()
  })
  await sendAndConfirmTransaction(
    'Create pointer',
    connection,
    new Transaction().add(instruction),
    ourAccount
  )
}
main()
