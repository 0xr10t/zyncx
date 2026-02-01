import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import * as fs from 'fs';

// Load IDL
const idl = JSON.parse(fs.readFileSync('./target/idl/zyncx.json', 'utf-8'));

// Decode the user's note
const noteBase64 = "eyJzZWNyZXQiOiIzNGJhNjcwYjU4NGVjODRhOTgzMTU2MDg5Zjc5NzJmZjZkODYzOTI4MmI4OWYwZTMyM2ZhMmI0YjMzMGFkOTUxIiwibnVsbGlmaWVyU2VjcmV0IjoiNjNmZWQ0NzRmOGI2OTBhZTkxODE3NWY2YzZiNzNiMjY3YjEyODE1ZTU5M2Y5NmQwYzQxNmRhNTM4NGE4YTdlNCIsInByZWNvbW1pdG1lbnQiOiJmMzY5NDc4MzMwYjkyMmNmZjFlZmViMWExZmU0YzMwZmY2NTc3OTVkY2EzZjk0NWFkMzBlYmE3MTI5NjkwZjdkIiwiYW1vdW50IjoiMTAwMDAwMDAiLCJjb21taXRtZW50IjoiY2IzZjI3YWNlNzk4NTY4MzJhMjM0ZGYyZTkxNGVhMWJlY2MwN2MyYWIzNTZmN2NiMzY2NTg5YWQ3ZTZmMmI5YiIsInRpbWVzdGFtcCI6MTc2OTk3MzQwNDY4NCwidHhTaWduYXR1cmUiOiI0S2ppOVpTMnZnTUpnanloaUY5ekdpMXBFaFlQM2I2cDZ4bWNQbm1ORXVickQ5dzl6SGc3a1R6UVdQOG5DRmRjRUxNRkVVdTV3TE1yaUtIcE4zQ1B4dXB0In0=";
const note = JSON.parse(Buffer.from(noteBase64, 'base64').toString());

const PROGRAM_ID = new PublicKey('5TGQEPDL2K6RoxKLbfjD2KMypbvKewDUsfuaNAvCAUMU');
const VERIFIER_PROGRAM = new PublicKey('AWUEQfGnU2nVYAA3dfKpckDhqjoW6HELT5wvkg9Sve1y');
const NATIVE_MINT = new PublicKey(new Uint8Array(32));

function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

async function main() {
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  const keypairPath = process.env.HOME + '/.config/solana/id.json';
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));
  const wallet = new Wallet(keypair);
  
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
  const program = new Program(idl, provider);
  
  const [vault] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), NATIVE_MINT.toBuffer()],
    PROGRAM_ID
  );
  
  const [merkleTree] = PublicKey.findProgramAddressSync(
    [Buffer.from('merkle_tree'), vault.toBuffer()],
    PROGRAM_ID
  );
  
  const [vaultTreasury] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault_treasury'), vault.toBuffer()],
    PROGRAM_ID
  );
  
  const nullifierSecret = hexToBytes(note.nullifierSecret);
  const nullifier = Array.from(nullifierSecret);
  const newCommitment = Array(32).fill(0);
  const proof = Buffer.alloc(256);
  for (let i = 0; i < 256; i++) proof[i] = (i * 7 + 13) % 256;
  const amount = new BN(note.amount);
  
  // Compute nullifier PDA
  const [nullifierAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), Buffer.from(nullifier)],
    PROGRAM_ID
  );
  
  console.log("🔍 Debug: Building transaction with Anchor...\n");
  
  // Build instruction using Anchor (just get the instruction, don't send)
  // @ts-ignore
  const ix = await (program.methods as any)
    .withdrawNative(amount, nullifier, newCommitment, proof)
    .accounts({
      recipient: wallet.publicKey,
      vault: vault,
      merkleTree: merkleTree,
      vaultTreasury: vaultTreasury,
      nullifierAccount: nullifierAccount,
      verifierProgram: VERIFIER_PROGRAM,
      payer: wallet.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .instruction();
  
  console.log("📝 Anchor-generated instruction data:");
  console.log("   Total length:", ix.data.length);
  console.log("   Hex:", Buffer.from(ix.data).toString('hex'));
  console.log("");
  
  // Parse the instruction data
  const data = Buffer.from(ix.data);
  const discriminator = data.slice(0, 8);
  const amountBytes = data.slice(8, 16);
  const nullifierBytes = data.slice(16, 48);
  const newCommitmentBytes = data.slice(48, 80);
  const proofLenBytes = data.slice(80, 84);
  const proofLen = proofLenBytes.readUInt32LE(0);
  
  console.log("📊 Parsed fields:");
  console.log("   Discriminator (0-8):", discriminator.toString('hex'));
  console.log("   Amount (8-16):", amountBytes.toString('hex'), "=", amountBytes.readBigUInt64LE().toString());
  console.log("   Nullifier (16-48):", nullifierBytes.toString('hex'));
  console.log("   NewCommitment (48-80):", newCommitmentBytes.toString('hex'));
  console.log("   ProofLen (80-84):", proofLen);
  console.log("");
  
  // Compare with what we passed
  console.log("🔄 What we passed:");
  console.log("   Nullifier:", Buffer.from(nullifier).toString('hex'));
  console.log("   Match:", Buffer.from(nullifier).toString('hex') === nullifierBytes.toString('hex') ? "✅ YES" : "❌ NO");
  console.log("");
  
  // Now compute PDAs with both values
  const pdaWithOurNullifier = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), Buffer.from(nullifier)],
    PROGRAM_ID
  )[0];
  
  const pdaWithAnchorNullifier = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), nullifierBytes],
    PROGRAM_ID
  )[0];
  
  console.log("📍 PDA comparisons:");
  console.log("   With our nullifier:", pdaWithOurNullifier.toBase58());
  console.log("   With Anchor's nullifier:", pdaWithAnchorNullifier.toBase58());
  console.log("   Match:", pdaWithOurNullifier.equals(pdaWithAnchorNullifier) ? "✅ YES" : "❌ NO");
  console.log("");
  
  // What the program expects (from error logs)
  console.log("❌ From error logs:");
  console.log("   Program expects: 42LhtoEvfwzNPLd1P6L3MYVkxuDHE7yeStCRJtBkZbQE");
  console.log("");
  
  // Try to reverse-engineer what nullifier would produce that PDA
  // We can't easily do this, but let's check if maybe it's using vault.asset_mint instead of vault.key()
  const [pdaWithMint] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), NATIVE_MINT.toBuffer(), Buffer.from(nullifier)],
    PROGRAM_ID
  );
  console.log("🔬 Alternative derivations:");
  console.log("   With NATIVE_MINT instead of vault:", pdaWithMint.toBase58());
}

main().catch(console.error);
