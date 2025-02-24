import * as anchor from "@coral-xyz/anchor";
import {
  approve,
  createAccount,
  createMint,
  getAccount,
  getMint,
  Mint,
  mintTo,
  Account as TokenAccount,
} from "@solana/spl-token";
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
  swapTokenA: PublicKey;
  swapTokenB: PublicKey;
  mintA: PublicKey;
  mintB: PublicKey;
  // payer for transactions
  payer: Keypair;
  poolMint: PublicKey;
  //only use for receiver of the pool token during init
  userPoolTokenAccount: PublicKey;
  poolFeeAccount: PublicKey;
  amountOfCurrentSwapToken: { a: bigint; b: bigint };
  constructor() {}
  public static async init(connection: Connection, programId: PublicKey) {
    let test = new TokenSwapTest();
    test.owner = Keypair.generate();
    test.payer = Keypair.generate();
    test.amountOfCurrentSwapToken = {
      a: amountOfCurrentSwapTokenA,
      b: amountOfCurrentSwapTokenB,
    };
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
    const [poolMint, mintA, mintB] = await Promise.all([
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
    test.poolMint = poolMint;
    test.mintA = mintA;
    test.mintB = mintB;

    // Batch 2: Create all accounts
    const [userPoolTokenReciever, poolFeeAccount, swapTokenA, swapTokenB] =
      await Promise.all([
        createAccount(
          connection,
          test.payer,
          test.poolMint,
          test.owner.publicKey,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
        createAccount(
          connection,
          test.payer,
          test.poolMint,
          test.owner.publicKey,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
        createAccount(
          connection,
          test.payer,
          test.mintA,
          test.authority,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
        createAccount(
          connection,
          test.payer,
          test.mintB,
          test.authority,
          Keypair.generate(),
          undefined,
          TOKEN_2022_PROGRAM_ID
        ),
      ]);
    test.userPoolTokenAccount = userPoolTokenReciever;
    test.poolFeeAccount = poolFeeAccount;
    test.swapTokenA = swapTokenA;
    test.swapTokenB = swapTokenB;

    // Batch 3: Mint tokens
    await Promise.all([
      mintTo(
        connection,
        test.payer,
        test.mintA,
        test.swapTokenA,
        test.owner,
        test.amountOfCurrentSwapToken.a,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      mintTo(
        connection,
        test.payer,
        test.mintB,
        test.swapTokenB,
        test.owner,
        test.amountOfCurrentSwapToken.b,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    return test;
  }
  public async getAccount(
    connection: Connection,
    key: PublicKey
  ): Promise<TokenAccount> {
    return getAccount(connection, key, undefined, TOKEN_2022_PROGRAM_ID);
  }
  private async createToken(
    connection: Connection,
    mint: PublicKey,
    key: Keypair = Keypair.generate()
  ): Promise<PublicKey> {
    return createAccount(
      connection,
      this.payer,
      mint,
      this.owner.publicKey,
      key,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
  }

  /**
   * Get the token accounts from the swap tokens
   * @param connection
   * @returns [tokenA, tokenB]
   */
  public async getSwapTokenAccounts(
    connection: Connection
  ): Promise<[TokenAccount, TokenAccount]> {
    return Promise.all([
      this.getAccount(connection, this.swapTokenA),
      this.getAccount(connection, this.swapTokenB),
    ]);
  }
  /**
   * Create a pair of token accounts of mintA and mintB
   * @param connection
   * @returns [tokenA, tokenB]
   */
  public async createTokenPair(
    connection: Connection
  ): Promise<[PublicKey, PublicKey]> {
    const result = await Promise.all([
      this.createToken(connection, this.mintA),
      this.createToken(connection, this.mintB),
    ]);
    return result;
  }
  /**
   * Mint tokens to the token accounts
   * @param connection
   * @param tokenAccountA
   * @param tokenAccountB
   * @param amounts
   */
  public async mintToTokenPair(
    connection: Connection,
    tokenAccountA: PublicKey,
    tokenAccountB: PublicKey,
    amountA: bigint,
    amountB: bigint
  ): Promise<void> {
    await Promise.all([
      mintTo(
        connection,
        this.payer,
        this.mintA,
        tokenAccountA,
        this.owner,
        amountA,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      mintTo(
        connection,
        this.payer,
        this.mintB,
        tokenAccountB,
        this.owner,
        amountB,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);
  }
  public async approveForPair(
    connection: Connection,
    tokenAccountA: PublicKey,
    tokenAccountB: PublicKey,
    delegate_authority: PublicKey,
    amountA: bigint,
    amountB: bigint
  ): Promise<void> {
    await Promise.all([
      approve(
        connection,
        this.payer,
        tokenAccountA,
        delegate_authority,
        this.owner,
        amountA,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      approve(
        connection,
        this.payer,
        tokenAccountB,
        delegate_authority,
        this.owner,
        amountB,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);
  }
  public async getPoolMint(connection: Connection): Promise<Mint> {
    return getMint(connection, this.poolMint, undefined, TOKEN_2022_PROGRAM_ID);
  }
}
