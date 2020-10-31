import { Connection, PublicKey, Account } from '@solana/web3.js'
import { Token } from '@solana/spl-token'

export async function createToken(
  connection: Connection,
  ourAccount: Account,
  decimals: number = 9,
  freezeAuthority?: string,
  mintAuthority?: string
): Promise<string> {
  console.log('create token')
  const token = await Token.createMint(
    connection,
    ourAccount,
    mintAuthority ? new PublicKey(mintAuthority) : ourAccount.publicKey,
    freezeAuthority ? new PublicKey(freezeAuthority) : null,
    decimals,
    new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
  )
  // @ts-expect-error
  console.log(`created token ${token.publicKey.toString()}`)
  // @ts-expect-error
  return token.publicKey.toString()
}
