import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair, Connection } from "@solana/web3.js";

/**
 * Airdrop and confirm the transaction completes
 * @param key The public key to airdrop to, underneath is can create an account on the pubkey is doesn't exist
 * @param connection 
 * @param amount The amount to airdrop, default is 2 SOL
 */
export async function airdrop_and_confirm(key: PublicKey, connection: Connection, amount: number = 2 * LAMPORTS_PER_SOL) {
  let signature = await connection.requestAirdrop(key, amount);
  await connection.confirmTransaction({
    signature,
    ...(await connection.getLatestBlockhash()),
  });
}
