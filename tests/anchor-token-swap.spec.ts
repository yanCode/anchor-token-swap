import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { AnchorTokenSwap } from "../target/types/anchor_token_swap";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair, Connection } from "@solana/web3.js";
import { assert } from "chai";
import { airdrop_and_confirm } from "./token";
import { createAccount, createMint, mintTo, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

// Initial amount in each swap token
let amountOfCurrentSwapTokenA = 1000000n;
let amountOfCurrentSwapTokenB = 1000000n;
let currentFeeAmount = 0n;

// Pool fees
const TRADING_FEE_NUMERATOR = new BN(25);
const TRADING_FEE_DENOMINATOR = new BN(10000);
const OWNER_TRADING_FEE_NUMERATOR = new BN(5);
const OWNER_TRADING_FEE_DENOMINATOR = new BN(10000);
const OWNER_WITHDRAW_FEE_NUMERATOR = new BN(1);
const OWNER_WITHDRAW_FEE_DENOMINATOR = new BN(6);
const HOST_FEE_NUMERATOR = new BN(20);
const HOST_FEE_DENOMINATOR = new BN(100);

class TokenSwapTest {
  tokenSwapAccount: Keypair
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
  constructor() {

  }
  public static async initialize(connection: Connection, programId: PublicKey) {

    let test = new TokenSwapTest();
    test.owner = Keypair.generate();
    test.payer = Keypair.generate();
    await airdrop_and_confirm(test.owner.publicKey, connection);
    await airdrop_and_confirm(test.payer.publicKey, connection, 10 * LAMPORTS_PER_SOL);
    test.tokenSwapAccount = Keypair.generate();
    [test.authority, test.authorityBumpSeed] = PublicKey.findProgramAddressSync(
      [test.tokenSwapAccount.publicKey.toBuffer()],
      programId
    );
    test.tokenPool = await createMint(
      connection,
      test.payer,
      test.authority,
      null,
      2,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID);
    console.log('creating pool account');
    test.tokenAccountPool = await createAccount(
      connection,
      test.payer,
      test.tokenPool,
      test.owner.publicKey,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    test.feeAccount = await createAccount(
      connection,
      test.payer,
      test.tokenPool,
      test.owner.publicKey,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID);
    test.mintA = await createMint(
      connection,
      test.payer,
      test.owner.publicKey,
      null,
      2,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    test.mintB = await createMint(
      connection,
      test.payer,
      test.owner.publicKey,
      null,
      2,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    test.tokenAccountA = await createAccount(
      connection,
      test.payer,
      test.mintA,
      test.owner.publicKey,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    test.tokenAccountB = await createAccount(
      connection,
      test.payer,
      test.mintB,
      test.owner.publicKey,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    await mintTo(
      connection,
      test.payer,
      test.mintA,
      test.tokenAccountA,
      test.owner,
      amountOfCurrentSwapTokenA,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    await mintTo(
      connection,
      test.payer,
      test.mintB,
      test.tokenAccountB,
      test.owner,
      amountOfCurrentSwapTokenB,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    return test;
  }
}


describe("anchor-token-swap", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  anchor.setProvider(new anchor.AnchorProvider(connection, anchor.AnchorProvider.env().wallet, {
    commitment: "confirmed",
  }));
  let tokenSwapTest: TokenSwapTest;
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const program = anchor.workspace.AnchorTokenSwap as Program<AnchorTokenSwap>;

  beforeEach(async () => {
    tokenSwapTest = await TokenSwapTest.initialize(connection, program.programId);
  });




  it("Is initialized!", async () => {
    // const tx = program.methods.initialize({
    //   constantProduct: {}
    // },
    //   {
    //     tradeFeeNumerator: TRADING_FEE_NUMERATOR,
    //     tradeFeeDenominator: TRADING_FEE_DENOMINATOR,
    //     ownerTradeFeeNumerator: OWNER_TRADING_FEE_NUMERATOR,
    //     ownerTradeFeeDenominator: OWNER_TRADING_FEE_DENOMINATOR,
    //     ownerWithdrawFeeNumerator: OWNER_WITHDRAW_FEE_NUMERATOR,
    //     ownerWithdrawFeeDenominator: OWNER_WITHDRAW_FEE_DENOMINATOR,
    //     hostFeeNumerator: HOST_FEE_NUMERATOR,
    //     hostFeeDenominator: HOST_FEE_DENOMINATOR
    //   })
    //   .accounts({
    //     swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
    //     tokenA: tokenSwapTest.mintA,
    //     tokenB: tokenSwapTest.mintB,
    //     poolMint: tokenSwapTest.tokenPool,
    //     destination: tokenSwapTest.tokenAccountPool,
    //     feeAccount: tokenSwapTest.feeAccount,
    //   })
    //   .signers([tokenSwapTest.tokenSwapAccount])
    //   .rpc();
    // console.log('tx', tx);
    // const tx = await program.methods.initialize({
    //   curveType: CurveType.ConstantProduct,
    //   fees: {
    //     tokenBPrice: 1000000000000000000n,
    //   }
    // }).accountsPartial({
    //   tokenSwapAccount: tokenSwapTest.tokenSwapAccount.publicKey,
    //   tokenPool: tokenSwapTest.tokenPool,
    //   tokenAccountPool: tokenSwapTest.tokenAccountPool,
    //   feeAccount: tokenSwapTest.feeAccount,
    // }).rpc();
  });
});
