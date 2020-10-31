import {
  Transaction,
  SystemProgram,
  TransactionInstruction,
  PublicKey,
  encodeData,
  sendAndConfirmTransaction
} from '@solana/web3.js'
import { getOurAccount } from './ourAccount'
import { getNodeConnection } from './nodeConnection'
import { makeAccount } from './deploy'
import * as bs58 from 'bs58'
// const url = 'http://devnet.solana.com'
// const connection = new solanaWeb3.Connection(url)
import { getStore } from './storeConfig'

const main = async () => {
  const ourAccount = await getOurAccount()

  const connection = await getNodeConnection()
  // connection.getBalance(ourAccount.publicKey).then(balance => {
  //   console.log(`${ourAccount.publicKey} has a balance of ${balance}`)
  // })
  const s = await getStore(connection, 'token-name-service.json')
  // connection.onProgramAccountChange(s.programId, (accountInfo, { slot }) => {
  //   console.log(accountInfo)
  //   console.log('######')
  // })
  // connection.onAccountChange(s.programId, (accountInfo, { slot }) => {
  //   console.log(accountInfo)
  //   console.log('######')
  // })
  // connection.onProgramAccountChange(s.accountId, (accountInfo, { slot }) => {
  //   console.log(accountInfo)
  //   console.log('######')
  // })
  // connection.onAccountChange(s.accountId, (accountInfo, { slot }) => {
  //   console.log(accountInfo)
  //   console.log('######')
  // })
  // await makeAccount(connection, ourAccount, 10, s.programId)

  // const a2 = await connection.getAccountInfo(
  //   new PublicKey('4DKqqaa6fKuC6rA6vG3BL7mydgxpHTUCN5M3wV61JcMT')
  // )
  // console.log(Buffer.from('4NGtJoZ8wy7mwtzWi8JByPMWbTAQHicHKAfcCbsx1yra'))
  // console.log(Buffer.from('4NGtJoZ8wy7mwtzWi8JByPMWbTAQHicHKAfcCbsx1yra').toString())
  // console.log(a.value.data)

  // console.log(bs58.decode('4NGtJoZ8wy7mwtzWi8JByPMWbTAQHicHKAfcCbsx1yra'))
  // console.log(a2.data.length)

  // for (let i = 0; i < a2.data.slice(50, 82).length; i++) {
  //   const element = a2.data.slice(50, 82)[i]
  //   console.log(element.toString(16))
  // }
  // console.log(dataBufferToHexBytes(a2.data))
  // for (const xd of a2.data.slice(50, 82)) {
  //   console.log(xd.toString(16))
  // }
  // console.log(a2.data[0].toString(16))
  // console.log(new PublicKey(a2.data.slice(50, 82)).toString())

  // await makeAccount(
  //   connection,
  //   ourAccount,
  //   10,
  //   new PublicKey('2nwRV6XFViD21U9ufJN2wqx44Qsh6cXHyonMbi9AYa7u')
  // )
  // console.log(a[1])
  // connection.onAccountChange(s.accountId, (accountInfo, { slot }) => {
  //   console.log(accountInfo)
  //   console.log('######3')
  // })
  connection.onAccountChange(s.programId, (accountInfo, { slot }) => {
    console.log(accountInfo)
    console.log('######2')
  })
  connection.onProgramAccountChange(s.programId, (accountInfo, { slot }) => {
    console.log(accountInfo.accountInfo.data)
    const data = accountInfo.accountInfo.data
    const key = new PublicKey(data.slice(0, 32))
    const msg = data.slice(32, 64).toString()
    console.log('######1')
    console.log(key.toString())
    console.log(msg)
  })
  const instruction_data = Buffer.alloc(32)

  instruction_data.write('1234567890123456789012345678901234')
  console.log(instruction_data)
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: ourAccount.publicKey, isSigner: false, isWritable: true },
      {
        pubkey: new PublicKey('7JRWSgMszap7MUUcZMaMXpUHEAyQ1k9y5cTBvRxQMCU6'),
        isSigner: false,
        isWritable: true
      },
      { pubkey: ourAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: s.accountId, isSigner: false, isWritable: true }
    ],
    programId: s.programId,
    data: instruction_data
  })
  // console.log(new PublicKey('7JRWSgMszap7MUUcZMaMXpUHEAyQ1k9y5cTBvRxQMCU6'))
  // // // // await sendAndConfirmTransaction(
  // // // //   'vote',
  // // // //   connection,
  // // // //   new Transaction().add(instruction),
  // // // //   ourAccount
  // // // // )
  // await connection.requestAirdrop(s.accountId, 1 * 1e9)
  console.log(await connection.getBalance(s.accountId))
  const signature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [ourAccount],
    {
      commitment: 'recent'
    }
  )

  console.log(signature)
}
main()
