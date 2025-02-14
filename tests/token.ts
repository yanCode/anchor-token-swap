import * as anchor from "@coral-xyz/anchor";
import { createAccount, createMint, mintTo } from "@solana/spl-token";
import {
  PublicKey,
  LAMPORTS_PER_SOL,
  Keypair,
  Connection,
} from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

// Initial amount in each swap token
let amountOfCurrentSwapTokenA = 1000000n;
let amountOfCurrentSwapTokenB = 1000000n;

/**
 * Airdrop and confirm the transaction completes
 * @param key The public key to airdrop to, underneath is can create an account on the pubkey is doesn't exist
 * @param connection
 * @param amount The amount to airdrop, default is 2 SOL
 */
export async function airdrop_and_confirm(
  key: PublicKey,
  connection: Connection,
  amount: number = 2 * LAMPORTS_PER_SOL
) {
  let signature = await connection.requestAirdrop(key, amount);
  await connection.confirmTransaction({
    signature,
    ...(await connection.getLatestBlockhash()),
  });
}

export class TokenSwapTest {
  tokenSwapAccount: Keypair;
  authority: PublicKey;
  authorityBumpSeed: number;
  provider: anchor.AnchorProvider;
  // owner of the user accounts
  owner: Keypair;
  tokenAccountA: PublicKey;
  tokenAccountB: PublicKey;
  mintA: PublicKey;
  mintB: PublicKey;
  // payer for transactions
  payer: Keypair;
  tokenPool: PublicKey;
  tokenAccountPool: PublicKey;
  feeAccount: PublicKey;
  constructor() {}
  public static async init(connection: Connection, programId: PublicKey) {
    let test = new TokenSwapTest();
    test.owner = Keypair.generate();
    test.payer = Keypair.generate();

    // Airdrop transactions
    await Promise.all([
      airdrop_and_confirm(test.owner.publicKey, connection),
      airdrop_and_confirm(
        test.payer.publicKey,
        connection,
        10 * LAMPORTS_PER_SOL
      ),
    ]);

    test.tokenSwapAccount = Keypair.generate();
    [test.authority, test.authorityBumpSeed] = PublicKey.findProgramAddressSync(
      [test.tokenSwapAccount.publicKey.toBuffer()],
      programId
    );

    // Batch 1: Create all mints
    const [tokenPool, mintA, mintB] = await Promise.all([
      createMint(
        connection,
        test.payer,
        test.authority,
        null,
        2,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      createMint(
        connection,
        test.payer,
        test.owner.publicKey,
        null,
        2,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      createMint(
        connection,
        test.payer,
        test.owner.publicKey,
        null,
        2,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);
    test.tokenPool = tokenPool;
    test.mintA = mintA;
    test.mintB = mintB;

    // Batch 2: Create all accounts
    const [tokenAccountPool, feeAccount, tokenAccountA, tokenAccountB] =
      await Promise.all([
        createAccount(
          connection,
          test.payer,
          test.tokenPool,
          test.owner.publicKey,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
        createAccount(
          connection,
          test.payer,
          test.tokenPool,
          test.owner.publicKey,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
        createAccount(
          connection,
          test.payer,
          test.mintA,
          test.owner.publicKey,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
        createAccount(
          connection,
          test.payer,
          test.mintB,
          test.owner.publicKey,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
      ]);
    test.tokenAccountPool = tokenAccountPool;
    test.feeAccount = feeAccount;
    test.tokenAccountA = tokenAccountA;
    test.tokenAccountB = tokenAccountB;

    // Batch 3: Mint tokens
    await Promise.all([
      mintTo(
        connection,
        test.payer,
        test.mintA,
        test.tokenAccountA,
        test.owner,
        amountOfCurrentSwapTokenA,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      mintTo(
        connection,
        test.payer,
        test.mintB,
        test.tokenAccountB,
        test.owner,
        amountOfCurrentSwapTokenB,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    return test;
  }
}
