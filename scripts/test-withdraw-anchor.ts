import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import * as fs from 'fs';

// Load IDL
const idl = JSON.parse(fs.readFileSync('./target/idl/zyncx.json', 'utf-8'));

// Decode the user's note
const noteBase64 = "eyJzZWNyZXQiOiIzNGJhNjcwYjU4NGVjODRhOTgzMTU2MDg5Zjc5NzJmZjZkODYzOTI4MmI4OWYwZTMyM2ZhMmI0YjMzMGFkOTUxIiwibnVsbGlmaWVyU2VjcmV0IjoiNjNmZWQ0NzRmOGI2OTBhZTkxODE3NWY2YzZiNzNiMjY3YjEyODE1ZTU5M2Y5NmQwYzQxNmRhNTM4NGE4YTdlNCIsInByZWNvbW1pdG1lbnQiOiJmMzY5NDc4MzMwYjkyMmNmZjFlZmViMWExZmU0YzMwZmY2NTc3OTVkY2EzZjk0NWFkMzBlYmE3MTI5NjkwZjdkIiwiYW1vdW50IjoiMTAwMDAwMDAiLCJjb21taXRtZW50IjoiY2IzZjI3YWNlNzk4NTY4MzJhMjM0ZGYyZTkxNGVhMWJlY2MwN2MyYWIzNTZmN2NiMzY2NTg5YWQ3ZTZmMmI5YiIsInRpbWVzdGFtcCI6MTc2OTk3MzQwNDY4NCwidHhTaWduYXR1cmUiOiI0S2ppOVpTMnZnTUpnanloaUY5ekdpMXBFaFlQM2I2cDZ4bWNQbm1ORXVickQ5dzl6SGc3a1R6UVdQOG5DRmRjRUxNRkVVdTV3TE1yaUtIcE4zQ1B4dXB0In0=";
const note = JSON.parse(Buffer.from(noteBase64, 'base64').toString());

console.log("📋 Decoded Note:");
console.log("  nullifierSecret:", note.nullifierSecret);
console.log("  amount:", note.amount, "lamports");

// Program ID
const PROGRAM_ID = new PublicKey('5TGQEPDL2K6RoxKLbfjD2KMypbvKewDUsfuaNAvCAUMU');
const VERIFIER_PROGRAM = new PublicKey('AWUEQfGnU2nVYAA3dfKpckDhqjoW6HELT5wvkg9Sve1y');

// Native mint (zero pubkey)
const NATIVE_MINT = new PublicKey(new Uint8Array(32));

// Helper to convert hex to bytes
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
  
  // Load wallet
  const keypairPath = process.env.HOME + '/.config/solana/id.json';
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));
  const wallet = new Wallet(keypair);
  
  console.log("\n🔑 Wallet:", wallet.publicKey.toBase58());
  
  // Create provider and program
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
  const program = new Program(idl, provider);
  
  // Derive vault PDA
  const [vault] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), NATIVE_MINT.toBuffer()],
    PROGRAM_ID
  );
  console.log("📦 Vault:", vault.toBase58());
  
  // Get nullifier secret from note - try using it RAW (as the secret itself, not hashed)
  const nullifierSecret = hexToBytes(note.nullifierSecret);
  console.log("\n🔐 NullifierSecret:", Buffer.from(nullifierSecret).toString('hex'));
  
  // The nullifier we pass to the program - try different options
  console.log("\n🧪 Testing with Anchor client...\n");
  
  // Option A: Use nullifierSecret directly as the nullifier
  const nullifier = Array.from(nullifierSecret);
  
  // Derive PDAs that Anchor will use
  const [merkleTree] = PublicKey.findProgramAddressSync(
    [Buffer.from('merkle_tree'), vault.toBuffer()],
    PROGRAM_ID
  );
  
  const [vaultTreasury] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault_treasury'), vault.toBuffer()],
    PROGRAM_ID
  );
  
  // Nullifier PDA - using the same nullifier bytes
  const [nullifierAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), Buffer.from(nullifier)],
    PROGRAM_ID
  );
  
  console.log("📍 NullifierPDA (computed):", nullifierAccount.toBase58());
  console.log("🌳 MerkleTree:", merkleTree.toBase58());
  console.log("💰 VaultTreasury:", vaultTreasury.toBase58());
  
  // New commitment (zeros for full withdrawal)
  const newCommitment = Array(32).fill(0);
  
  // Mock proof
  const proof = Buffer.alloc(256);
  for (let i = 0; i < 256; i++) proof[i] = (i * 7 + 13) % 256;
  
  const amount = new BN(note.amount);
  
  console.log("\n📝 Calling withdrawNative via Anchor...");
  console.log("   Amount:", amount.toString());
  console.log("   Nullifier (first 8):", Buffer.from(nullifier.slice(0, 8)).toString('hex'));
  
  try {
    // Use Anchor's program.methods to build the transaction
    // @ts-ignore - Anchor type complexity
    const tx = await (program.methods as any)
      .withdrawNative(
        amount,
        nullifier,
        newCommitment,
        proof
      )
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
      .rpc();
    
    console.log("\n✅ WITHDRAW SUCCESSFUL!");
    console.log("   Signature:", tx);
    console.log("   https://explorer.solana.com/tx/" + tx + "?cluster=devnet");
  } catch (error: any) {
    console.log("\n❌ Transaction failed:");
    console.log(error.message);
    
    // Try to get more details
    if (error.logs) {
      console.log("\nProgram Logs:");
      error.logs.forEach((log: string) => console.log("  ", log));
    }
    
    // Let's also check what PDA Anchor would derive
    console.log("\n🔍 Debugging PDA derivation...");
    console.log("   Seeds: [b'nullifier', vault, nullifier_bytes]");
    console.log("   Vault:", vault.toBase58());
    console.log("   Nullifier bytes (hex):", Buffer.from(nullifier).toString('hex'));
    
    // Let's manually check the PDA
    const [manualPDA, bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('nullifier'),
        vault.toBuffer(),
        Buffer.from(nullifier),
      ],
      PROGRAM_ID
    );
    console.log("   Manual PDA:", manualPDA.toBase58());
    console.log("   Bump:", bump);
  }
}

main().catch(console.error);
