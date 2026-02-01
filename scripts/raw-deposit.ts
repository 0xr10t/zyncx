import { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';
import { BN } from 'bn.js';

const PROGRAM_ID = new PublicKey('6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC');
const NATIVE_MINT = new PublicKey(new Uint8Array(32));

async function main() {
  console.log('🧪 Testing Raw Deposit (No Anchor Client)\n');

  // Load wallet
  const walletPath = path.join(process.env.HOME || '~', '.config/solana/id.json');
  const walletKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, 'utf-8')))
  );

  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  console.log(`Wallet: ${walletKeypair.publicKey.toBase58()}`);
  const balance = await connection.getBalance(walletKeypair.publicKey);
  console.log(`Balance: ${balance / 1e9} SOL\n`);

  // Derive PDAs
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

  console.log(`Vault: ${vault.toBase58()}`);
  console.log(`Merkle Tree: ${merkleTree.toBase58()}`);
  console.log(`Treasury: ${vaultTreasury.toBase58()}\n`);

  // Generate random precommitment
  const precommitment = crypto.randomBytes(32);
  const amount = new BN(10_000_000); // 0.01 SOL

  console.log(`Depositing: ${amount.toNumber() / 1e9} SOL`);
  console.log(`Precommitment: ${precommitment.toString('hex')}\n`);

  // Build instruction data manually
  // Discriminator for deposit_native: sha256("global:deposit_native")[0..8]
  const discriminator = Buffer.from([13, 158, 5, 185, 190, 249, 182, 175]);
  
  // Serialize amount (u64, little-endian)
  const amountBuffer = Buffer.alloc(8);
  amountBuffer.writeBigUInt64LE(BigInt(amount.toString()));
  
  // Instruction data = discriminator + amount + precommitment
  const instructionData = Buffer.concat([
    discriminator,
    amountBuffer,
    precommitment,
  ]);

  console.log(`Instruction data length: ${instructionData.length} bytes`);
  console.log(`Discriminator: ${discriminator.toString('hex')}`);

  // Build instruction
  const depositIx = new TransactionInstruction({
    keys: [
      { pubkey: walletKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: merkleTree, isSigner: false, isWritable: true },
      { pubkey: vaultTreasury, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: instructionData,
  });

  // Send transaction
  console.log('📤 Sending raw transaction...\n');
  const tx = new Transaction().add(depositIx);
  tx.feePayer = walletKeypair.publicKey;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  tx.sign(walletKeypair);

  try {
    const signature = await connection.sendRawTransaction(tx.serialize(), {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
    });
    
    console.log(`📡 Transaction sent: ${signature}`);
    console.log('⏳ Waiting for confirmation...\n');
    
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log('✅ Deposit successful!');
    console.log(`🔗 https://explorer.solana.com/tx/${signature}?cluster=devnet`);
  } catch (error: any) {
    console.error('❌ Deposit failed:', error.message);
    if (error.logs) {
      console.error('\nProgram logs:');
      error.logs.forEach((log: string) => console.error(log));
    }
  }
}

main().catch(console.error);
