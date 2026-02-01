import { PublicKey } from '@solana/web3.js';

const PROGRAM_ID = new PublicKey('5TGQEPDL2K6RoxKLbfjD2KMypbvKewDUsfuaNAvCAUMU');
const vault = new PublicKey('BovZNYudQdDRxFT8464iWnifqdJA3vNgPJ2jm6zWfmnz');
const TARGET_PDA = '42LhtoEvfwzNPLd1P6L3MYVkxuDHE7yeStCRJtBkZbQE';

// Full instruction data from Anchor (hex)
const ixDataHex = '71e31a2035425afa809698000000000063fed474f8b690ae918175f6c6b73b267b12815e593f96d0c416da5384a8a7e400000000000000000000000000000000000000000000000000000000000000000001000';
const ixData = Buffer.from(ixDataHex, 'hex');

console.log("🔍 Searching for correct nullifier offset...\n");
console.log("Target PDA:", TARGET_PDA);
console.log("Instruction data length:", ixData.length);
console.log("");

// Try reading 32 bytes from different offsets
for (let offset = 0; offset <= ixData.length - 32; offset++) {
  const nullifierCandidate = ixData.slice(offset, offset + 32);
  
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), nullifierCandidate],
    PROGRAM_ID
  );
  
  if (pda.toBase58() === TARGET_PDA) {
    console.log(`✅ FOUND! Offset ${offset}:`);
    console.log(`   Bytes: ${nullifierCandidate.toString('hex')}`);
    console.log(`   PDA: ${pda.toBase58()}`);
    break;
  }
}

// Also try with amount bytes included (maybe it's reading amount + part of nullifier?)
console.log("\n🔬 Checking specific offsets:");

// Expected: offset 16 (after discriminator + amount)
const expectedNullifier = ixData.slice(16, 48);
const [expectedPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from('nullifier'), vault.toBuffer(), expectedNullifier],
  PROGRAM_ID
);
console.log(`Offset 16 (expected): ${expectedPDA.toBase58()}`);
console.log(`  Bytes: ${expectedNullifier.toString('hex')}`);

// What if it's reading from offset 8 (amount + nullifier)?
const offset8 = ixData.slice(8, 40);
const [pda8] = PublicKey.findProgramAddressSync(
  [Buffer.from('nullifier'), vault.toBuffer(), offset8],
  PROGRAM_ID
);
console.log(`Offset 8: ${pda8.toBase58()}`);
console.log(`  Bytes: ${offset8.toString('hex')}`);

// What if there's no discriminator?
const offset0 = ixData.slice(0, 32);
const [pda0] = PublicKey.findProgramAddressSync(
  [Buffer.from('nullifier'), vault.toBuffer(), offset0],
  PROGRAM_ID
);
console.log(`Offset 0: ${pda0.toBase58()}`);
console.log(`  Bytes: ${offset0.toString('hex')}`);

// Try with NATIVE_MINT instead of vault
const NATIVE_MINT = new PublicKey(new Uint8Array(32));
const [pdaWithMint] = PublicKey.findProgramAddressSync(
  [Buffer.from('nullifier'), NATIVE_MINT.toBuffer(), expectedNullifier],
  PROGRAM_ID
);
console.log(`\nWith NATIVE_MINT: ${pdaWithMint.toBase58()}`);
