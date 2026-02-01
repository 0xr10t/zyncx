import { Connection, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import * as fs from 'fs';
import { Keypair } from '@solana/web3.js';

const idl = JSON.parse(fs.readFileSync('./target/idl/zyncx.json', 'utf-8'));
const PROGRAM_ID = new PublicKey('5TGQEPDL2K6RoxKLbfjD2KMypbvKewDUsfuaNAvCAUMU');
const NATIVE_MINT = new PublicKey(new Uint8Array(32));

async function main() {
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  const keypairPath = process.env.HOME + '/.config/solana/id.json';
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));
  const wallet = new Wallet(keypair);
  
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
  const program = new Program(idl, provider);
  
  // Derive expected vault PDA
  const [expectedVault, expectedBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), NATIVE_MINT.toBuffer()],
    PROGRAM_ID
  );
  
  console.log("📦 Expected Vault PDA:", expectedVault.toBase58());
  console.log("   Expected Bump:", expectedBump);
  console.log("   NATIVE_MINT:", NATIVE_MINT.toBase58());
  console.log("");
  
  // Fetch vault account
  try {
    // @ts-ignore
    const vaultAccount = await program.account.vaultState.fetch(expectedVault);
    
    console.log("✅ Vault account found!");
    console.log("   Bump:", vaultAccount.bump);
    console.log("   Asset Mint:", vaultAccount.assetMint.toBase58());
    console.log("   Merkle Tree:", vaultAccount.merkleTree.toBase58());
    console.log("   Nonce:", vaultAccount.nonce.toString());
    console.log("   Authority:", vaultAccount.authority.toBase58());
    console.log("   Total Deposited:", vaultAccount.totalDeposited.toString());
    console.log("   Vault Type:", JSON.stringify(vaultAccount.vaultType));
    console.log("");
    
    // Verify vault address matches what we expect
    const [derivedFromStoredMint] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), vaultAccount.assetMint.toBuffer()],
      PROGRAM_ID
    );
    console.log("🔍 Verification:");
    console.log("   Vault derived from stored asset_mint:", derivedFromStoredMint.toBase58());
    console.log("   Matches expected?", derivedFromStoredMint.equals(expectedVault) ? "✅ YES" : "❌ NO");
    console.log("");
    
    // Check merkle tree
    const [expectedMerkle] = PublicKey.findProgramAddressSync(
      [Buffer.from('merkle_tree'), expectedVault.toBuffer()],
      PROGRAM_ID
    );
    console.log("🌳 Merkle Tree:");
    console.log("   Stored:", vaultAccount.merkleTree.toBase58());
    console.log("   Expected:", expectedMerkle.toBase58());
    console.log("   Match?", vaultAccount.merkleTree.equals(expectedMerkle) ? "✅ YES" : "❌ NO");
    
  } catch (e: any) {
    console.log("❌ Could not fetch vault:", e.message);
  }
  
  // Also fetch the raw account data to see what's there
  console.log("\n📊 Raw account data:");
  const accountInfo = await connection.getAccountInfo(expectedVault);
  if (accountInfo) {
    console.log("   Owner:", accountInfo.owner.toBase58());
    console.log("   Data length:", accountInfo.data.length);
    console.log("   Lamports:", accountInfo.lamports);
    
    // Parse discriminator
    const discriminator = accountInfo.data.slice(0, 8);
    console.log("   Discriminator:", Buffer.from(discriminator).toString('hex'));
    
    // Parse bump (byte 8)
    const bump = accountInfo.data[8];
    console.log("   Bump (byte 8):", bump);
    
    // The asset_mint should be at some offset - let's check
    // VaultState has: bump(1), vault_type(1+enum), asset_mint(32), merkle_tree(32), nonce(8), authority(32), total_deposited(8)
    // But Anchor also has 8-byte discriminator
    // So: discriminator(8) + bump(1) + vault_type(1 or more) + asset_mint(32)...
    
  } else {
    console.log("   Account not found!");
  }
}

main().catch(console.error);
