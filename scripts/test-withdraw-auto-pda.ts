import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import * as fs from 'fs';

const idl = JSON.parse(fs.readFileSync('./target/idl/zyncx.json', 'utf-8'));
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
  
  console.log("🧪 Testing with AUTO PDA derivation (not passing nullifierAccount)...\n");
  console.log("📦 Vault:", vault.toBase58());
  console.log("🔐 Nullifier (first 8):", Buffer.from(nullifier.slice(0, 8)).toString('hex'));
  
  try {
    // Let Anchor derive nullifierAccount automatically
    // @ts-ignore
    const tx = await (program.methods as any)
      .withdrawNative(amount, nullifier, newCommitment, proof)
      .accounts({
        recipient: wallet.publicKey,
        vault: vault,
        merkleTree: merkleTree,
        vaultTreasury: vaultTreasury,
        // NOT passing nullifierAccount - let Anchor derive it
        verifierProgram: VERIFIER_PROGRAM,
        payer: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("\n✅ WITHDRAW SUCCESSFUL!");
    console.log("   Signature:", tx);
  } catch (error: any) {
    console.log("\n❌ Transaction failed:");
    console.log(error.message);
    
    if (error.logs) {
      console.log("\nLogs:");
      error.logs.forEach((log: string) => console.log("  ", log));
    }
    
    // Extract what PDA Anchor auto-derived
    console.log("\n🔍 Checking what PDA Anchor would auto-derive...");
    
    // Get the instruction to see what accounts Anchor uses
    // @ts-ignore
    const ix = await (program.methods as any)
      .withdrawNative(amount, nullifier, newCommitment, proof)
      .accounts({
        recipient: wallet.publicKey,
        vault: vault,
        merkleTree: merkleTree,
        vaultTreasury: vaultTreasury,
        verifierProgram: VERIFIER_PROGRAM,
        payer: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .instruction();
    
    console.log("\n📝 Accounts in instruction:");
    ix.keys.forEach((key: any, i: number) => {
      console.log(`   ${i}: ${key.pubkey.toBase58()} (signer: ${key.isSigner}, writable: ${key.isWritable})`);
    });
  }
}

main().catch(console.error);
