import {
  Account,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction
} from '@solana/web3.js'

import { getOurAccount } from './ourAccount'
import { getNodeConnection } from './nodeConnection'
import { getStore, setStore } from './storeConfig'

import { estCostLoadProgram, loadProgram, estCostMakeAccount, makeAccount } from './deploy'

import * as fs from 'fs'

const pathToProgram = 'dist/program/proxy-pointer.so'

async function main() {
  console.log('Deploying...')

  try {
    if (fs.existsSync(pathToProgram)) {
      //file exists
    }
  } catch (err) {
    console.error('No file ' + pathToProgram + ' -- build rust program first')
    process.exit(1)
  }

  const ourAccount = await getOurAccount()

  const connection = await getNodeConnection()

  // NB: the use of this store is just a convenience, nothing fundamental is going on here

  const s = await getStore(connection, 'proxy-pointer.json')

  if (s.inStore === true) {
    console.log(
      'Program has already been deployed, pubkey:',
      s.programId.toString(),
      ' with data account:',
      s.accountId.toString()
    )
    process.exit(0)
  }

  console.log('-----')

  const estimatedCostOfLoad = await estCostLoadProgram(connection, pathToProgram)

  console.log(
    'Estimated cost to program load:',
    estimatedCostOfLoad,
    ' lamports (',
    estimatedCostOfLoad / LAMPORTS_PER_SOL,
    ') Sol'
  )

  const startingBalance = await connection.getBalance(ourAccount.publicKey)

  const programId = await loadProgram(connection, ourAccount, pathToProgram)

  const afterLoadBalance = await connection.getBalance(ourAccount.publicKey)

  const costLoad = startingBalance - afterLoadBalance

  console.log(
    'Program loaded to:',
    programId.toBase58(),
    ' cost was:',
    costLoad,
    ' lamports (',
    costLoad / LAMPORTS_PER_SOL,
    ') Sol'
  )

  const proxyAccount = await makeAccount(connection, ourAccount, 33, programId)
  console.log(`Created pointer: ${proxyAccount.toString()}`)
  await setStore('proxy-pointer.json', programId, proxyAccount)
  console.log('-----')
}

main()
  .catch(err => {
    console.error(err)
  })
  .then(() => process.exit())
