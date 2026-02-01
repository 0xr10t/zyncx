import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram } from '@solana/web3.js';
import * as fs from 'fs';

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

// Simple hash function matching frontend
function simpleHash(data: Uint8Array): Uint8Array {
  const result = new Uint8Array(32);
  for (let i = 0; i < Math.min(data.length, 32); i++) {
    result[i] = data[i];
  }
  for (let i = 0; i < data.length; i++) {
    const idx = i % 32;
    result[idx] = (result[idx] ^ data[i]) & 0xff;
    const next = (idx + 1) % 32;
    result[next] = ((result[next] + result[idx] + i) ^ (data[i] >>> 1)) & 0xff;
  }
  for (let round = 0; round < 64; round++) {
    const temp = result[0];
    for (let j = 0; j < 31; j++) {
      result[j] = ((result[j] * 31 + result[j + 1] * 37 + round) ^ (result[(j + 16) % 32])) & 0xff;
    }
    result[31] = ((result[31] * 31 + temp * 37 + round) ^ result[15]) & 0xff;
  }
  return result;
}

async function main() {
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  // Load wallet
  const keypairPath = process.env.HOME + '/.config/solana/id.json';
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const wallet = Keypair.fromSecretKey(new Uint8Array(secretKey));
  
  console.log("\n🔑 Wallet:", wallet.publicKey.toBase58());
  
  // Derive vault PDA
  const [vault] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), NATIVE_MINT.toBuffer()],
    PROGRAM_ID
  );
  console.log("📦 Vault:", vault.toBase58());
  
  // Get nullifier secret from note
  const nullifierSecret = hexToBytes(note.nullifierSecret);
  console.log("\n🔐 NullifierSecret bytes:", Buffer.from(nullifierSecret).toString('hex'));
  
  // Option 1: Use raw nullifierSecret as nullifier
  const nullifierRaw = nullifierSecret;
  console.log("Option 1 - Raw nullifierSecret:", Buffer.from(nullifierRaw).toString('hex'));
  
  // Option 2: Use hashed nullifierSecret  
  const nullifierHashed = simpleHash(nullifierSecret);
  console.log("Option 2 - Hashed nullifierSecret:", Buffer.from(nullifierHashed).toString('hex'));
  
  // Derive nullifier PDAs for both options
  const [nullifierPDA_raw] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), Buffer.from(nullifierRaw)],
    PROGRAM_ID
  );
  console.log("\n📍 NullifierPDA (raw):", nullifierPDA_raw.toBase58());
  
  const [nullifierPDA_hashed] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), Buffer.from(nullifierHashed)],
    PROGRAM_ID
  );
  console.log("📍 NullifierPDA (hashed):", nullifierPDA_hashed.toBase58());
  
  // The error shows:
  // Left: HpT7x2gQvFn3XeZK9PDF8Ztgtm4EhoMqm4KFgaCD8APh (frontend computed)
  // Right: DQhzJZqxysngNGSqapk7husTbmEs1E1Nnd6UpfiCmShm (program computed)
  
  console.log("\n❌ Error showed:");
  console.log("   Left (frontend):  HpT7x2gQvFn3XeZK9PDF8Ztgtm4EhoMqm4KFgaCD8APh");
  console.log("   Right (program):  DQhzJZqxysngNGSqapk7husTbmEs1E1Nnd6UpfiCmShm");
  
  // Check which one matches
  if (nullifierPDA_raw.toBase58() === "HpT7x2gQvFn3XeZK9PDF8Ztgtm4EhoMqm4KFgaCD8APh") {
    console.log("\n✅ Raw nullifier matches LEFT (frontend)");
  }
  if (nullifierPDA_hashed.toBase58() === "HpT7x2gQvFn3XeZK9PDF8Ztgtm4EhoMqm4KFgaCD8APh") {
    console.log("\n✅ Hashed nullifier matches LEFT (frontend)");
  }
  if (nullifierPDA_raw.toBase58() === "DQhzJZqxysngNGSqapk7husTbmEs1E1Nnd6UpfiCmShm") {
    console.log("\n✅ Raw nullifier matches RIGHT (program expects)");
  }
  if (nullifierPDA_hashed.toBase58() === "DQhzJZqxysngNGSqapk7husTbmEs1E1Nnd6UpfiCmShm") {
    console.log("\n✅ Hashed nullifier matches RIGHT (program expects)");
  }
  
  // The program uses the nullifier argument directly in PDA derivation
  // So we need to pass the SAME bytes we use to compute the PDA
  // Let's test with the RAW nullifierSecret (not hashed)
  console.log("\n🧪 Testing withdraw with RAW nullifierSecret (no hash)...");
  
  const nullifier = nullifierRaw; // Use raw, not hashed
  const [nullifierPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), Buffer.from(nullifier)],
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
  
  console.log("📍 Using NullifierPDA:", nullifierPDA.toBase58());
  console.log("🌳 MerkleTree:", merkleTree.toBase58());
  console.log("💰 VaultTreasury:", vaultTreasury.toBase58());
  
  // Build withdraw instruction
  // Discriminator for withdraw_native from IDL
  const discriminator = Buffer.from([113, 227, 26, 32, 53, 66, 90, 250]);
  
  const amount = BigInt(note.amount);
  const amountBuffer = Buffer.alloc(8);
  amountBuffer.writeBigUInt64LE(amount);
  
  const newCommitment = new Uint8Array(32); // zeros for full withdrawal
  
  // Mock proof
  const proof = new Uint8Array(256);
  for (let i = 0; i < 256; i++) proof[i] = (i * 7 + 13) % 256;
  
  // Proof length as u32 LE
  const proofLenBuffer = Buffer.alloc(4);
  proofLenBuffer.writeUInt32LE(proof.length);
  
  // Instruction data: discriminator(8) + amount(8) + nullifier(32) + new_commitment(32) + proof_len(4) + proof(256)
  const instructionData = Buffer.concat([
    discriminator,
    amountBuffer,
    Buffer.from(nullifier),
    Buffer.from(newCommitment),
    proofLenBuffer,
    Buffer.from(proof),
  ]);
  
  console.log("\n📝 Instruction data length:", instructionData.length);
  console.log("   Discriminator:", discriminator.toString('hex'));
  console.log("   Amount:", amount.toString());
  console.log("   Nullifier (first 8 bytes):", Buffer.from(nullifier).slice(0, 8).toString('hex'));
  
  const withdrawIx = new TransactionInstruction({
    keys: [
      { pubkey: wallet.publicKey, isSigner: false, isWritable: true }, // recipient
      { pubkey: vault, isSigner: false, isWritable: true }, // vault
      { pubkey: merkleTree, isSigner: false, isWritable: true }, // merkle_tree
      { pubkey: vaultTreasury, isSigner: false, isWritable: true }, // vault_treasury
      { pubkey: nullifierPDA, isSigner: false, isWritable: true }, // nullifier_account
      { pubkey: VERIFIER_PROGRAM, isSigner: false, isWritable: false }, // verifier_program
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true }, // payer
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system_program
    ],
    programId: PROGRAM_ID,
    data: instructionData,
  });
  
  const tx = new Transaction().add(withdrawIx);
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  tx.feePayer = wallet.publicKey;
  tx.sign(wallet);
  
  try {
    const signature = await connection.sendRawTransaction(tx.serialize());
    console.log("\n✅ WITHDRAW SUCCESSFUL!");
    console.log("   Signature:", signature);
    console.log("   https://explorer.solana.com/tx/" + signature + "?cluster=devnet");
  } catch (error: any) {
    console.log("\n❌ Transaction failed:");
    console.log(error.message);
    if (error.logs) {
      console.log("\nLogs:");
      error.logs.forEach((log: string) => console.log("  ", log));
    }
  }
}

main().catch(console.error);
