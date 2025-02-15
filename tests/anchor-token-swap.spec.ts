import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { AnchorTokenSwap } from "../target/types/anchor_token_swap";
import { TokenSwapTest } from "./token";
import {
  approve,
  createAccount,
  getAccount,
  getMint,
  mintTo,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";

let currentFeeAmount = 0n;

// Pool fees
const TRADING_FEE_NUMERATOR = 25;
const TRADING_FEE_DENOMINATOR = 10000;
const OWNER_TRADING_FEE_NUMERATOR = 5;
const OWNER_TRADING_FEE_DENOMINATOR = 10000;
const OWNER_WITHDRAW_FEE_NUMERATOR = 1;
const OWNER_WITHDRAW_FEE_DENOMINATOR = 6;
const HOST_FEE_NUMERATOR = 20;
const HOST_FEE_DENOMINATOR = 100;

// Pool token amount minted on init
// const DEFAULT_POOL_TOKEN_AMOUNT = 1000000000n;
// Pool token amount to withdraw / deposit
const POOL_TOKEN_AMOUNT = 10000000;

describe("anchor-token-swap", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  anchor.setProvider(
    new anchor.AnchorProvider(connection, anchor.AnchorProvider.env().wallet, {
      commitment: "confirmed",
    })
  );
  let tokenSwapTest: TokenSwapTest;
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const program = anchor.workspace.AnchorTokenSwap as Program<AnchorTokenSwap>;

  before(async () => {
    tokenSwapTest = await TokenSwapTest.init(connection, program.programId);
  });

  it("Is initialized!", async () => {
    let tx = await program.methods
      .initialize(
        {
          constantProduct: {},
        },
        {
          tradeFeeNumerator: new BN(TRADING_FEE_NUMERATOR),
          tradeFeeDenominator: new BN(TRADING_FEE_DENOMINATOR),
          ownerTradeFeeNumerator: new BN(OWNER_TRADING_FEE_NUMERATOR),
          ownerTradeFeeDenominator: new BN(OWNER_TRADING_FEE_DENOMINATOR),
          ownerWithdrawFeeNumerator: new BN(OWNER_WITHDRAW_FEE_NUMERATOR),
          ownerWithdrawFeeDenominator: new BN(OWNER_WITHDRAW_FEE_DENOMINATOR),
          hostFeeNumerator: new BN(HOST_FEE_NUMERATOR),
          hostFeeDenominator: new BN(HOST_FEE_DENOMINATOR),
        }
      )
      .accounts({
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        tokenA: tokenSwapTest.tokenAccountA,
        tokenB: tokenSwapTest.tokenAccountB,
        poolMint: tokenSwapTest.tokenPool,
        destination: tokenSwapTest.tokenAccountPool,
        feeAccount: tokenSwapTest.feeAccount,
      })
      .signers([tokenSwapTest.tokenSwapAccount])
      .rpc({ commitment: "confirmed" });
  });

  it("test depositAllTokenTypes", async () => {
    const poolMint = await getMint(
      connection,
      tokenSwapTest.tokenPool,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    const supply = poolMint.supply;

    const [swapTokenA, swapTokenB] = await Promise.all([
      getAccount(
        connection,
        tokenSwapTest.tokenAccountA,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      getAccount(
        connection,
        tokenSwapTest.tokenAccountB,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    const amountOftokenA =
      (swapTokenA.amount * BigInt(POOL_TOKEN_AMOUNT)) / supply;
    const amountOftokenB =
      (swapTokenB.amount * BigInt(POOL_TOKEN_AMOUNT)) / supply;
    const userTransferAuthority = Keypair.generate();

    const [userAccountA, userAccountB] = await Promise.all([
      createAccount(
        connection,
        tokenSwapTest.payer,
        tokenSwapTest.mintA,
        tokenSwapTest.owner.publicKey,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      createAccount(
        connection,
        tokenSwapTest.payer,
        tokenSwapTest.mintB,
        tokenSwapTest.owner.publicKey,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    await Promise.all([
      mintTo(
        connection,
        tokenSwapTest.payer,
        tokenSwapTest.mintA,
        userAccountA,
        tokenSwapTest.owner,
        amountOftokenA,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      mintTo(
        connection,
        tokenSwapTest.payer,
        tokenSwapTest.mintB,
        userAccountB,
        tokenSwapTest.owner,
        amountOftokenB,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    await Promise.all([
      approve(
        connection,
        tokenSwapTest.payer,
        userAccountA,
        userTransferAuthority.publicKey,
        tokenSwapTest.owner,
        amountOftokenA,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      approve(
        connection,
        tokenSwapTest.payer,
        userAccountB,
        userTransferAuthority.publicKey,
        tokenSwapTest.owner,
        amountOftokenB,
        [],
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    const newAccountPoolToken = await createAccount(
      connection,
      tokenSwapTest.payer,
      tokenSwapTest.tokenPool,
      tokenSwapTest.owner.publicKey,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await program.methods
      .depositAllTokenTypes(
        new BN(POOL_TOKEN_AMOUNT.toString()),
        new BN(amountOftokenA.toString()),
        new BN(amountOftokenB.toString())
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        userTransferAuthority: userTransferAuthority.publicKey,
        sourceA: userAccountA,
        sourceB: userAccountB,
        tokenA: tokenSwapTest.tokenAccountA,
        tokenB: tokenSwapTest.tokenAccountB,
        tokenAMint: tokenSwapTest.mintA,
        tokenBMint: tokenSwapTest.mintB,
        poolMint: tokenSwapTest.tokenPool,
        destination: newAccountPoolToken,
        poolFeeAccount: tokenSwapTest.feeAccount,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc({ commitment: "confirmed" });
  });

  it("test withdrawAllTokenTypes", async () => {
    const poolMint = await getMint(
      connection,
      tokenSwapTest.tokenPool,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    const supply = poolMint.supply;

    const [swapTokenA, swapTokenB] = await Promise.all([
      getAccount(
        connection,
        tokenSwapTest.tokenAccountA,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      getAccount(
        connection,
        tokenSwapTest.tokenAccountB,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);
    let feeAmount =
      (BigInt(POOL_TOKEN_AMOUNT) * BigInt(OWNER_WITHDRAW_FEE_NUMERATOR)) /
      BigInt(OWNER_WITHDRAW_FEE_DENOMINATOR);

    const poolTokenAmount = BigInt(POOL_TOKEN_AMOUNT) - BigInt(feeAmount);
    const amountOftokenA =
      (swapTokenA.amount * BigInt(poolTokenAmount)) / supply;
    const amountOftokenB =
      (swapTokenB.amount * BigInt(poolTokenAmount)) / supply;

    const [userAccountA, userAccountB] = await Promise.all([
      createAccount(
        connection,
        tokenSwapTest.payer,
        tokenSwapTest.mintA,
        tokenSwapTest.owner.publicKey,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      createAccount(
        connection,
        tokenSwapTest.payer,
        tokenSwapTest.mintB,
        tokenSwapTest.owner.publicKey,
        Keypair.generate(),
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);
    const userTransferAuthority = Keypair.generate();

    await approve(
      connection,
      tokenSwapTest.payer,
      tokenSwapTest.tokenAccountPool,
      userTransferAuthority.publicKey,
      tokenSwapTest.owner,
      POOL_TOKEN_AMOUNT,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await program.methods
      .withdrawAllTokenTypes(
        new BN(poolTokenAmount.toString()),
        new BN(amountOftokenA.toString()),
        new BN(amountOftokenB.toString())
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        userTransferAuthority: userTransferAuthority.publicKey,
        destinationA: userAccountA,
        destinationB: userAccountB,
        source: tokenSwapTest.tokenAccountPool,
        tokenA: tokenSwapTest.tokenAccountA,
        tokenB: tokenSwapTest.tokenAccountB,
        tokenAMint: tokenSwapTest.mintA,
        tokenBMint: tokenSwapTest.mintB,
        poolMint: tokenSwapTest.tokenPool,
        poolFeeAccount: tokenSwapTest.feeAccount,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc({ commitment: "confirmed" });
  });
});
