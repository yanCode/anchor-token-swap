import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { AnchorTokenSwap } from "../target/types/anchor_token_swap";
import { TokenSwapTest } from "./token";
import {
  approve,
  createAccount,
  createApproveInstruction,
  createInitializeAccountInstruction,
  createMintToInstruction,
  getAccount,
  getAccountLenForMint,
  getMinimumBalanceForRentExemptAccount,
  getMint,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { assert } from "chai";

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
const DEFAULT_POOL_TOKEN_AMOUNT = 1000000000n;
// Pool token amount to withdraw / deposit, which is 1% of `DEFAULT_POOL_TOKEN_AMOUNT`
const TEST_POOL_TOKEN_AMOUNT = 10000000n;
const SWAP_AMOUNT_IN = 100000n;
const EXPECTED_SWAP_AMOUNT_OUT = 90661n;

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

  it("It should initialized!", async () => {
    await program.methods
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
        swapTokenA: tokenSwapTest.swapTokenA,
        swapTokenB: tokenSwapTest.swapTokenB,
        poolMint: tokenSwapTest.poolMint,
        userPoolToken: tokenSwapTest.userPoolTokenAccount,
        poolFeeAccount: tokenSwapTest.poolFeeAccount,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([tokenSwapTest.tokenSwapAccount])
      .rpc();

    const userPoolTokenReciever = await tokenSwapTest.getAccount(
      connection,
      tokenSwapTest.userPoolTokenAccount
    );

    assert.equal(
      userPoolTokenReciever.amount,
      BigInt(DEFAULT_POOL_TOKEN_AMOUNT)
    );
    const poolMint = await tokenSwapTest.getPoolMint(connection);
    assert.equal(poolMint.supply, BigInt(DEFAULT_POOL_TOKEN_AMOUNT));
    const swapV1 = await program.account.swapV1.fetch(
      tokenSwapTest.tokenSwapAccount.publicKey
    );
    assert.ok(swapV1.fees.tradeFeeNumerator.eq(new BN(TRADING_FEE_NUMERATOR)));
    assert.ok(swapV1.tokenA.equals(tokenSwapTest.swapTokenA));
  });

  it("It should depositAllTokenTypes", async () => {
    const poolMint = await tokenSwapTest.getPoolMint(connection);
    const supply = poolMint.supply;

    const [swapTokenA, swapTokenB] = await Promise.all([
      getAccount(
        connection,
        tokenSwapTest.swapTokenA,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      getAccount(
        connection,
        tokenSwapTest.swapTokenB,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
    ]);

    const amountOftokenAToDeposit =
      (swapTokenA.amount * BigInt(TEST_POOL_TOKEN_AMOUNT)) / supply;
    const amountOftokenBToDeposit =
      (swapTokenB.amount * BigInt(TEST_POOL_TOKEN_AMOUNT)) / supply;
    const userTransferAuthority = Keypair.generate();
    const [userAccountA, userAccountB] = await tokenSwapTest.createTokenPair(
      connection
    );
    await tokenSwapTest.mintToTokenPair(
      connection,
      userAccountA,
      userAccountB,
      amountOftokenAToDeposit,
      amountOftokenBToDeposit
    );
    await tokenSwapTest.approveForPair(
      connection,
      userAccountA,
      userAccountB,
      userTransferAuthority.publicKey,
      amountOftokenAToDeposit,
      amountOftokenBToDeposit
    );

    const userPoolToken = await createAccount(
      connection,
      tokenSwapTest.payer,
      tokenSwapTest.poolMint,
      tokenSwapTest.owner.publicKey,
      Keypair.generate(),
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await program.methods
      .depositAllTokenTypes(
        new BN(TEST_POOL_TOKEN_AMOUNT.toString()),
        new BN(amountOftokenAToDeposit.toString()),
        new BN(amountOftokenBToDeposit.toString())
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        userTransferAuthority: userTransferAuthority.publicKey,
        sourceA: userAccountA,
        sourceB: userAccountB,
        tokenA: tokenSwapTest.swapTokenA,
        tokenB: tokenSwapTest.swapTokenB,
        tokenAMint: tokenSwapTest.mintA,
        tokenBMint: tokenSwapTest.mintB,
        poolMint: tokenSwapTest.poolMint,
        destination: userPoolToken,
        poolFeeAccount: tokenSwapTest.poolFeeAccount,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
        tokenAProgram: TOKEN_2022_PROGRAM_ID,
        tokenBProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc({ commitment: "confirmed" });
    const userAInfo = await tokenSwapTest.getAccount(connection, userAccountA);
    assert(userAInfo.amount == 0n);
    const userBInfo = await tokenSwapTest.getAccount(connection, userAccountB);
    assert(userBInfo.amount == 0n);
    tokenSwapTest.amountOfCurrentSwapToken.a += amountOftokenAToDeposit;
    tokenSwapTest.amountOfCurrentSwapToken.b += amountOftokenBToDeposit;
    const swapTokenAInfo = await tokenSwapTest.getAccount(
      connection,
      tokenSwapTest.swapTokenA
    );
    assert(swapTokenAInfo.amount == tokenSwapTest.amountOfCurrentSwapToken.a);
    const userPoolTokenInfo = await tokenSwapTest.getAccount(
      connection,
      userPoolToken
    );
    assert(userPoolTokenInfo.amount == TEST_POOL_TOKEN_AMOUNT);
  });

  it("It should withdrawAllTokenTypes", async () => {
    const poolMint = await tokenSwapTest.getPoolMint(connection);
    const supply = poolMint.supply;

    const [swapTokenA, swapTokenB] = await tokenSwapTest.getSwapTokenAccounts(
      connection
    );
    // let feeAmount =
    //   (BigInt(TEST_POOL_TOKEN_AMOUNT) * BigInt(OWNER_WITHDRAW_FEE_NUMERATOR)) /
    //   BigInt(OWNER_WITHDRAW_FEE_DENOMINATOR);//todo test pool fee account

    let feeAmount = 0n;
    const poolTokenAmount = BigInt(TEST_POOL_TOKEN_AMOUNT) - BigInt(feeAmount);
    const expectedWithdrawAmountOftokenA =
      (swapTokenA.amount * BigInt(poolTokenAmount)) / supply;
    const expectedWithdrawAmountOftokenB =
      (swapTokenB.amount * BigInt(poolTokenAmount)) / supply;
    const [userAccountA, userAccountB] = await tokenSwapTest.createTokenPair(
      connection
    );
    const userTransferAuthority = Keypair.generate();

    await approve(
      connection,
      tokenSwapTest.payer,
      tokenSwapTest.userPoolTokenAccount,
      userTransferAuthority.publicKey,
      tokenSwapTest.owner,
      TEST_POOL_TOKEN_AMOUNT,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await program.methods
      .withdrawAllTokenTypes(
        new BN(TEST_POOL_TOKEN_AMOUNT.toString()),
        new BN(expectedWithdrawAmountOftokenA.toString()),
        new BN(expectedWithdrawAmountOftokenB.toString())
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        userTransferAuthority: userTransferAuthority.publicKey,
        destinationA: userAccountA,
        destinationB: userAccountB,
        userPoolTokenSource: tokenSwapTest.userPoolTokenAccount,
        swapTokenA: tokenSwapTest.swapTokenA,
        swapTokenB: tokenSwapTest.swapTokenB,
        tokenAMint: tokenSwapTest.mintA,
        tokenBMint: tokenSwapTest.mintB,
        poolMint: tokenSwapTest.poolMint,
        poolFeeAccount: null,
        // poolFeeAccount: tokenSwapTest.poolFeeAccount,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
        tokenAProgram: TOKEN_2022_PROGRAM_ID,
        tokenBProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc();

    const [swapTokenAInfo, swapTokenBInfo] =
      await tokenSwapTest.getSwapTokenAccounts(connection);
    tokenSwapTest.amountOfCurrentSwapToken.a -= expectedWithdrawAmountOftokenA;
    tokenSwapTest.amountOfCurrentSwapToken.b -= expectedWithdrawAmountOftokenB;
    assert(swapTokenAInfo.amount == tokenSwapTest.amountOfCurrentSwapToken.a);
    assert(swapTokenBInfo.amount == tokenSwapTest.amountOfCurrentSwapToken.b);
    const userAccountAInfo = await tokenSwapTest.getAccount(
      connection,
      userAccountA
    );
    assert(userAccountAInfo.amount == expectedWithdrawAmountOftokenA);
    const userAccountBInfo = await tokenSwapTest.getAccount(
      connection,
      userAccountB
    );
    assert(userAccountBInfo.amount == expectedWithdrawAmountOftokenB);
  });
  it("It should create account & swap in a single tx", async () => {
    const sourceUserAccountA = Keypair.generate();
    const mintAProgramId = (
      await connection.getAccountInfo(tokenSwapTest.mintA, "confirmed")
    ).owner;
    const mintBProgramId = (
      await connection.getAccountInfo(tokenSwapTest.mintB, "confirmed")
    ).owner;
    const mintA = await getMint(
      connection,
      tokenSwapTest.mintA,
      "confirmed",
      mintAProgramId
    );
    const space = getAccountLenForMint(mintA);
    const lamports = await connection.getMinimumBalanceForRentExemption(space);
    const createSystemAccountForUserTokenAInstruction =
      SystemProgram.createAccount({
        fromPubkey: tokenSwapTest.payer.publicKey,
        newAccountPubkey: sourceUserAccountA.publicKey,
        space,
        programId: mintAProgramId,
        lamports: lamports,
      });
    const createInitializeSwapTokenAInstruction =
      createInitializeAccountInstruction(
        sourceUserAccountA.publicKey,
        tokenSwapTest.mintA,
        tokenSwapTest.owner.publicKey,
        mintAProgramId
      );
    const mintToUserTokenInstruction = createMintToInstruction(
      tokenSwapTest.mintA,
      sourceUserAccountA.publicKey,
      tokenSwapTest.owner.publicKey,
      SWAP_AMOUNT_IN,
      [],
      mintAProgramId
    );
    const balanceNeeded = await getMinimumBalanceForRentExemptAccount(
      connection
    );
    const userDestinationTokenB = Keypair.generate();

    const createSystemAccountForUserDestinationAInstruction =
      SystemProgram.createAccount({
        fromPubkey: tokenSwapTest.payer.publicKey,
        newAccountPubkey: userDestinationTokenB.publicKey,
        space,
        programId: mintBProgramId,
        lamports: balanceNeeded,
      });
    const createInitializeDestinationTokenBInstruction =
      createInitializeAccountInstruction(
        userDestinationTokenB.publicKey,
        tokenSwapTest.mintB,
        tokenSwapTest.owner.publicKey,
        mintBProgramId
      );
    const userTransferAuthority = Keypair.generate();
    const approveUserTokenInstruction = createApproveInstruction(
      sourceUserAccountA.publicKey,
      userTransferAuthority.publicKey,
      tokenSwapTest.owner.publicKey,
      SWAP_AMOUNT_IN,
      [],
      mintAProgramId
    );
    let swapInstruction = await program.methods
      .swap(
        new BN(SWAP_AMOUNT_IN.toString()),
        new BN(EXPECTED_SWAP_AMOUNT_OUT.toString())
      )
      .accounts({
        tokenSwap: tokenSwapTest.tokenSwapAccount.publicKey,
        swapSource: tokenSwapTest.swapTokenA,
        userSource: sourceUserAccountA.publicKey,
        swapDestination: tokenSwapTest.swapTokenB,
        userDestination: userDestinationTokenB.publicKey,
        poolMint: tokenSwapTest.poolMint,
        poolFeeAccount: tokenSwapTest.poolFeeAccount,
        userTransferAuthority: userTransferAuthority.publicKey,
        sourceTokenMint: tokenSwapTest.mintA,
        destinationTokenMint: tokenSwapTest.mintB,
        hostFeeAccount: null,
        tokenSourceProgram: TOKEN_2022_PROGRAM_ID,
        tokenDestinationProgram: TOKEN_2022_PROGRAM_ID,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
      })
      .instruction();
    const tx = new Transaction();
    tx.add(createSystemAccountForUserTokenAInstruction);
    tx.add(createInitializeSwapTokenAInstruction);
    tx.add(mintToUserTokenInstruction);
    tx.add(createSystemAccountForUserDestinationAInstruction);
    tx.add(createInitializeDestinationTokenBInstruction);
    tx.add(approveUserTokenInstruction);
    tx.add(swapInstruction);

    await sendAndConfirmTransaction(connection, tx, [
      tokenSwapTest.payer,
      tokenSwapTest.owner,
      sourceUserAccountA,
      userDestinationTokenB,
      userTransferAuthority,
    ]);
    const userDestinationTokenBInfo = await tokenSwapTest.getAccount(
      connection,
      userDestinationTokenB.publicKey
    );
    assert.equal(userDestinationTokenBInfo.amount, EXPECTED_SWAP_AMOUNT_OUT);
    const userSourceTokenAInfo = await tokenSwapTest.getAccount(
      connection,
      sourceUserAccountA.publicKey
    );
    assert.equal(userSourceTokenAInfo.amount, 0n);
    tokenSwapTest.amountOfCurrentSwapToken.a += SWAP_AMOUNT_IN;
    tokenSwapTest.amountOfCurrentSwapToken.b -= EXPECTED_SWAP_AMOUNT_OUT;
    const [swapTokenAInfo, swapTokenBInfo] =
      await tokenSwapTest.getSwapTokenAccounts(connection);
    assert.equal(
      swapTokenAInfo.amount,
      tokenSwapTest.amountOfCurrentSwapToken.a
    );
    assert.equal(
      swapTokenBInfo.amount,
      tokenSwapTest.amountOfCurrentSwapToken.b
    );
  });
  it("It should depositSingleTokenTypeExactAmountIn", async () => {
    // process.exit(0);
    // Pool token amount to deposit on one side
    const depositAmount = 10000n;

    const userTransferAuthority = Keypair.generate();
    const [userAccountA, userAccountB] = await tokenSwapTest.createTokenPair(
      connection
    );
    await tokenSwapTest.mintToTokenPair(
      connection,
      userAccountA,
      userAccountB,
      depositAmount,
      depositAmount
    );
    await tokenSwapTest.approveForPair(
      connection,
      userAccountA,
      userAccountB,
      userTransferAuthority.publicKey,
      depositAmount,
      depositAmount
    );
    // const destinationTokenAccount = await createAccount(
    //   connection,
    //   tokenSwapTest.payer,
    //   tokenSwapTest.poolMint,
    //   tokenSwapTest.owner.publicKey,
    //   Keypair.generate(),
    //   undefined,
    //   TOKEN_2022_PROGRAM_ID
    // );
    await program.methods
      .depositSingleTokenTypeExactAmountIn(
        new BN(depositAmount.toString()),
        new BN(0)
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        userTransferAuthority: userTransferAuthority.publicKey,
        source: userAccountA,
        sourceTokenMint: tokenSwapTest.mintA,
        swapTokenA: tokenSwapTest.swapTokenA,
        swapTokenB: tokenSwapTest.swapTokenB,
        sourceTokenProgram: TOKEN_2022_PROGRAM_ID,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
        poolMint: tokenSwapTest.poolMint,
        poolTokenDestination: tokenSwapTest.userPoolTokenAccount,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc();
    const userAccountAInfo = await tokenSwapTest.getAccount(
      connection,
      userAccountA
    );
    assert.equal(userAccountAInfo.amount, 0n);
    tokenSwapTest.amountOfCurrentSwapToken.a += depositAmount;
    const swapTokenAInfo = await tokenSwapTest.getAccount(
      connection,
      tokenSwapTest.swapTokenA
    );
    assert.equal(
      swapTokenAInfo.amount,
      tokenSwapTest.amountOfCurrentSwapToken.a
    );

    await program.methods
      .depositSingleTokenTypeExactAmountIn(
        new BN(depositAmount.toString()),
        new BN(0)
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        userTransferAuthority: userTransferAuthority.publicKey,
        source: userAccountB,
        sourceTokenMint: tokenSwapTest.mintB,
        swapTokenA: tokenSwapTest.swapTokenA,
        swapTokenB: tokenSwapTest.swapTokenB,
        sourceTokenProgram: TOKEN_2022_PROGRAM_ID,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
        poolMint: tokenSwapTest.poolMint,
        poolTokenDestination: tokenSwapTest.userPoolTokenAccount,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc();

    const userAccountBInfo = await tokenSwapTest.getAccount(
      connection,
      userAccountB
    );
    assert.equal(userAccountBInfo.amount, 0n);
    tokenSwapTest.amountOfCurrentSwapToken.b += depositAmount;
    const swapTokenBInfo = await tokenSwapTest.getAccount(
      connection,
      tokenSwapTest.swapTokenB
    );
    assert.equal(
      swapTokenBInfo.amount,
      tokenSwapTest.amountOfCurrentSwapToken.b
    );
  });

  it("It should withdrawSingleTokenTypeExactAmountIn", async () => {
    // Pool token amount to withdraw on one side
    const withdrawAmount = 50000n;
    const adjustedPoolTokenA = 1_000_000_000_000n;
    const adjustedPoolTokenB = 1_000_000_000_000n;
    const userTransferAuthority = Keypair.generate();
    const [_, userAccountB] = await tokenSwapTest.createTokenPair(connection);
    await approve(
      connection,
      tokenSwapTest.payer,
      tokenSwapTest.userPoolTokenAccount,
      userTransferAuthority.publicKey,
      tokenSwapTest.owner,
      adjustedPoolTokenA + adjustedPoolTokenB,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    await program.methods
      .withdrawSingleTokenTypeExactAmountOut(
        new BN(withdrawAmount.toString()),
        new BN(adjustedPoolTokenB.toString())
      )
      .accounts({
        payer: tokenSwapTest.payer.publicKey,
        swapV1: tokenSwapTest.tokenSwapAccount.publicKey,
        poolTokenSource: tokenSwapTest.userPoolTokenAccount,
        userTransferAuthority: userTransferAuthority.publicKey,
        userTokenDestination: userAccountB,
        swapTokenA: tokenSwapTest.swapTokenA,
        swapTokenB: tokenSwapTest.swapTokenB,
        poolMint: tokenSwapTest.poolMint,
        tokenAMint: tokenSwapTest.mintA,
        tokenBMint: tokenSwapTest.mintB,
        tokenPoolProgram: TOKEN_2022_PROGRAM_ID,
        destinationTokenMint: tokenSwapTest.mintB,
        destinationTokenProgram: TOKEN_2022_PROGRAM_ID,
        poolFeeAccount: null,
      })
      .signers([tokenSwapTest.payer, userTransferAuthority])
      .rpc();
    const userAccountBInfo = await tokenSwapTest.getAccount(
      connection,
      userAccountB
    );
    assert.equal(userAccountBInfo.amount, withdrawAmount);
  });
});
