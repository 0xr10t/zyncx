import { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';

// Program ID (deployed on devnet)
const PROGRAM_ID = new PublicKey('4C1cTQ89vywkBtaPuSXu5FZCuf89eqpXPDbsGMKUhgGT');
const NATIVE_MINT = new PublicKey(new Uint8Array(32)); // Zero pubkey for SOL

async function main() {
  console.log('🏦 Initializing ZYNCX Vault on Devnet');
  console.log('=====================================\n');

  // Load wallet
  const walletPath = path.join(process.env.HOME || '~', '.config/solana/id.json');
  const walletKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, 'utf-8')))
  );
  console.log(` Wallet: ${walletKeypair.publicKey.toBase58()}`);

  // Connect to devnet
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  const balance = await connection.getBalance(walletKeypair.publicKey);
  console.log(`💰 Balance: ${balance / 1e9} SOL\n`);

  // Derive PDAs
  const [vaultPDA, vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), NATIVE_MINT.toBuffer()],
    PROGRAM_ID
  );
  const [merkleTreePDA, merkleTreeBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('merkle_tree'), vaultPDA.toBuffer()],
    PROGRAM_ID
  );

  console.log(`📦 Vault PDA: ${vaultPDA.toBase58()}`);
  console.log(`🌳 Merkle Tree PDA: ${merkleTreePDA.toBase58()}\n`);

  // Check if vault already exists
  const vaultAccount = await connection.getAccountInfo(vaultPDA);
  if (vaultAccount) {
    console.log('✅ Vault already initialized!');
    return;
  }

  // Build initialize_vault instruction
  // Discriminator for initialize_vault (first 8 bytes of sha256("global:initialize_vault"))
  const discriminator = Buffer.from([48, 191, 163, 44, 71, 129, 63, 164]);
  
  const instructionData = Buffer.concat([
    discriminator,
    NATIVE_MINT.toBuffer(), // asset_mint
  ]);

  const initVaultIx = new TransactionInstruction({
    keys: [
      { pubkey: walletKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: vaultPDA, isSigner: false, isWritable: true },
      { pubkey: merkleTreePDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: instructionData,
  });

  // Send transaction
  console.log('📤 Sending initialize_vault transaction...');
  const tx = new Transaction().add(initVaultIx);
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  tx.sign(walletKeypair);

  try {
    const signature = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction(signature, 'confirmed');
    console.log(`✅ Vault initialized! Tx: ${signature}`);
    console.log(`🔗 https://explorer.solana.com/tx/${signature}?cluster=devnet`);
  } catch (error: any) {
    console.error('❌ Error:', error.message);
    if (error.logs) {
      console.error('Logs:', error.logs);
    }
  }
}

main().catch(console.error);
