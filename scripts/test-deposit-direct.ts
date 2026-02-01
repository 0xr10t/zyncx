import { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction, sendAndConfirmTransaction } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

const PROGRAM_ID = new PublicKey('5TGQEPDL2K6RoxKLbfjD2KMypbvKewDUsfuaNAvCAUMU');
const NATIVE_MINT = new PublicKey(new Uint8Array(32));

async function main() {
  console.log('🧪 Direct On-Chain Deposit Test\n');

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

  console.log(`Program: ${PROGRAM_ID.toBase58()}`);
  console.log(`Vault: ${vault.toBase58()}`);
  console.log(`Merkle Tree: ${merkleTree.toBase58()}`);
  console.log(`Treasury: ${vaultTreasury.toBase58()}\n`);

  // Generate random precommitment
  const precommitment = crypto.randomBytes(32);
  const amount = 10_000_000; // 0.01 SOL in lamports

  console.log(`Depositing: ${amount / 1e9} SOL`);
  console.log(`Precommitment: ${precommitment.toString('hex')}\n`);

  // Build instruction data manually using EXACT discriminator from IDL
  // IDL shows: [13, 158, 13, 223, 95, 213, 28, 6]
  const discriminator = Buffer.from([13, 158, 13, 223, 95, 213, 28, 6]);
  
  // Serialize amount (u64, little-endian)
  const amountBuffer = Buffer.alloc(8);
  amountBuffer.writeBigUInt64LE(BigInt(amount));
  
  // Instruction data = discriminator + amount + precommitment
  const instructionData = Buffer.concat([
    discriminator,
    amountBuffer,
    precommitment,
  ]);

  console.log(`Instruction data length: ${instructionData.length} bytes`);
  console.log(`Discriminator: ${discriminator.toString('hex')}`);
  console.log(`Amount bytes: ${amountBuffer.toString('hex')}`);

  // Build instruction with exact accounts from IDL
  const depositIx = new TransactionInstruction({
    keys: [
      { pubkey: walletKeypair.publicKey, isSigner: true, isWritable: true }, // depositor
      { pubkey: vault, isSigner: false, isWritable: true }, // vault
      { pubkey: merkleTree, isSigner: false, isWritable: true }, // merkle_tree
      { pubkey: vaultTreasury, isSigner: false, isWritable: true }, // vault_treasury
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system_program
    ],
    programId: PROGRAM_ID,
    data: instructionData,
  });

  // Send transaction
  console.log('\n📤 Sending transaction to devnet...\n');
  const tx = new Transaction().add(depositIx);
  
  try {
    const signature = await sendAndConfirmTransaction(
      connection,
      tx,
      [walletKeypair],
      { commitment: 'confirmed' }
    );
    
    console.log('✅ DEPOSIT SUCCESSFUL!');
    console.log(`Signature: ${signature}`);
    console.log(`🔗 https://explorer.solana.com/tx/${signature}?cluster=devnet`);
  } catch (error: any) {
    console.error('❌ Deposit failed:', error.message);
    
    // Get transaction logs
    if (error.logs) {
      console.error('\n📋 Program logs:');
      error.logs.forEach((log: string) => console.error('  ', log));
    }
    
    // Try to get more details
    if (error.signature) {
      console.log(`\nFailed TX: ${error.signature}`);
      try {
        const txDetails = await connection.getTransaction(error.signature, {
          maxSupportedTransactionVersion: 0,
        });
        if (txDetails?.meta?.logMessages) {
          console.log('\n📋 Full transaction logs:');
          txDetails.meta.logMessages.forEach(log => console.log('  ', log));
        }
      } catch (e) {
        // Ignore
      }
    }
  }
}

main().catch(console.error);
