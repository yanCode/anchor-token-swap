import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorTokenSwap } from "../target/types/anchor_token_swap";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair } from "@solana/web3.js";
import { assert } from "chai";

describe("anchor-token-swap", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  anchor.setProvider(new anchor.AnchorProvider(connection, anchor.AnchorProvider.env().wallet, {
    commitment: "confirmed",
  }));
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorTokenSwap as Program<AnchorTokenSwap>;

  it("Is initialized!", async () => {
    const [swap_pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("swap_v1")],
      program.programId
    );
    // Add your test here.
    const tx = await program.methods.initialize().rpc({
      commitment: "confirmed",
    });
    // const peekTx = await program.methods.peekCurve().rpc({
    //   commitment: "confirmed",
    // });

    // console.log("Your transaction signature", tx);
    // const swap = await program.account.swapV1.fetch(swap_pda);
    // console.log("Swap", swap);

    // const txReceipt = await program.provider.connection.getTransaction(peekTx, {
    //   maxSupportedTransactionVersion: 0,
    //   commitment: "confirmed",
    // });
    // console.log("Tx receipt", txReceipt.meta?.logMessages);
    // assert.equal(swap.calculator.name, "Constant Product hello world");
  });
});
