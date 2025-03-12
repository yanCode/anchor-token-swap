import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorTokenSwap } from "../../target/types/anchor_token_swap";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

describe("anchor-token-swap", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  anchor.setProvider(
    new anchor.AnchorProvider(connection, anchor.AnchorProvider.env().wallet, {
      commitment: "confirmed",
    })
  );

  anchor.setProvider(provider);
  const program = anchor.workspace.AnchorTokenSwap as Program<AnchorTokenSwap>;
  it("It should initialized!", async () => {
    const programDataAddress = PublicKey.findProgramAddressSync(
      [program.programId.toBuffer()],
      new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")
    )[0];
    await program.methods
      .upgradeVerifier()
      .accounts({
        programData: programDataAddress,
      })
      .rpc();
  });
});
