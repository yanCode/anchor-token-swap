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
const TRADING_FEE_NUMERATOR = new BN(25);
const TRADING_FEE_DENOMINATOR = new BN(10000);
const OWNER_TRADING_FEE_NUMERATOR = new BN(5);
const OWNER_TRADING_FEE_DENOMINATOR = new BN(10000);
const OWNER_WITHDRAW_FEE_NUMERATOR = new BN(1);
const OWNER_WITHDRAW_FEE_DENOMINATOR = new BN(6);
const HOST_FEE_NUMERATOR = new BN(20);
const HOST_FEE_DENOMINATOR = new BN(100);

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
          tradeFeeNumerator: TRADING_FEE_NUMERATOR,
          tradeFeeDenominator: TRADING_FEE_DENOMINATOR,
          ownerTradeFeeNumerator: OWNER_TRADING_FEE_NUMERATOR,
          ownerTradeFeeDenominator: OWNER_TRADING_FEE_DENOMINATOR,
          ownerWithdrawFeeNumerator: OWNER_WITHDRAW_FEE_NUMERATOR,
          ownerWithdrawFeeDenominator: OWNER_WITHDRAW_FEE_DENOMINATOR,
          hostFeeNumerator: HOST_FEE_NUMERATOR,
          hostFeeDenominator: HOST_FEE_DENOMINATOR,
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

  it("It should depoist all token types!", async () => {
    const poolMint = await getMint(
      connection,
      tokenSwapTest.tokenPool,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    const supply = poolMint.supply;
    const swapTokenA = await getAccount(
      connection,
      tokenSwapTest.tokenAccountA,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    const amountOftokenA =
      (swapTokenA.amount * BigInt(POOL_TOKEN_AMOUNT)) / supply;
    const swapTokenB = await getAccount(
      connection,
      tokenSwapTest.tokenAccountB,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
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
});
