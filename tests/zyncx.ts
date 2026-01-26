import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zyncx } from "../target/types/zyncx";
import { expect } from "chai";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";

describe("zyncx", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Zyncx as Program<Zyncx>;
  
  // Native asset mint (represents SOL)
  const NATIVE_MINT = new PublicKey("11111111111111111111111111111111");
  
  let vaultPda: PublicKey;
  let vaultBump: number;
  let merkleTreePda: PublicKey;
  let merkleTreeBump: number;
  let vaultTreasuryPda: PublicKey;
  let vaultTreasuryBump: number;

  before(async () => {
    // Derive PDAs
    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), NATIVE_MINT.toBuffer()],
      program.programId
    );

    [merkleTreePda, merkleTreeBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("merkle_tree"), vaultPda.toBuffer()],
      program.programId
    );

    [vaultTreasuryPda, vaultTreasuryBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_treasury"), vaultPda.toBuffer()],
      program.programId
    );
  });

  it("Initializes a native vault", async () => {
    const tx = await program.methods
      .initializeVault(NATIVE_MINT)
      .accounts({
        authority: provider.wallet.publicKey,
        vault: vaultPda,
        merkleTree: merkleTreePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize vault tx:", tx);

    // Fetch and verify vault state
    const vaultAccount = await program.account.vaultState.fetch(vaultPda);
    expect(vaultAccount.assetMint.toString()).to.equal(NATIVE_MINT.toString());
    expect(vaultAccount.nonce.toNumber()).to.equal(0);
    expect(vaultAccount.totalDeposited.toNumber()).to.equal(0);
  });

  it("Deposits native SOL", async () => {
    const depositAmount = new anchor.BN(1_000_000_000); // 1 SOL
    const precommitment = Buffer.alloc(32);
    precommitment.fill(1); // Simple precommitment for testing

    const tx = await program.methods
      .depositNative(depositAmount, Array.from(precommitment))
      .accounts({
        depositor: provider.wallet.publicKey,
        vault: vaultPda,
        merkleTree: merkleTreePda,
        vaultTreasury: vaultTreasuryPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Deposit tx:", tx);

    // Verify vault state updated
    const vaultAccount = await program.account.vaultState.fetch(vaultPda);
    expect(vaultAccount.nonce.toNumber()).to.equal(1);
    expect(vaultAccount.totalDeposited.toNumber()).to.equal(1_000_000_000);

    // Verify merkle tree updated
    const merkleTreeAccount = await program.account.merkleTreeState.fetch(merkleTreePda);
    expect(merkleTreeAccount.size.toNumber()).to.equal(1);
  });

  it("Verifies merkle root exists", async () => {
    const merkleTreeAccount = await program.account.merkleTreeState.fetch(merkleTreePda);
    const currentRoot = merkleTreeAccount.root;

    const result = await program.methods
      .checkRoot(Array.from(currentRoot))
      .accounts({
        merkleTree: merkleTreePda,
        vault: vaultPda,
      })
      .view();

    expect(result).to.be.true;
  });

  // Note: Withdrawal and swap tests require valid ZK proofs
  // These would be generated off-chain using snarkjs or similar
  it.skip("Withdraws native SOL with ZK proof", async () => {
    // This test requires a valid ZK proof
    // In production, the proof would be generated off-chain
  });

  it.skip("Swaps native SOL via DEX with ZK proof", async () => {
    // This test requires a valid ZK proof and DEX integration
  });
});
