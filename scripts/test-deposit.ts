import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { AnchorProvider, Program, Wallet } from '@coral-xyz/anchor';
import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

const PROGRAM_ID = new PublicKey('6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC');
const NATIVE_MINT = new PublicKey(new Uint8Array(32));

async function main() {
  console.log('🧪 Testing Direct Deposit\n');

  // Load wallet
  const walletPath = path.join(process.env.HOME || '~', '.config/solana/id.json');
  const walletKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, 'utf-8')))
  );

  // Connect
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  const wallet = new Wallet(walletKeypair);
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });

  console.log(`Wallet: ${wallet.publicKey.toBase58()}`);
  const balance = await connection.getBalance(wallet.publicKey);
  console.log(`Balance: ${balance / 1e9} SOL\n`);

  // Load IDL
  const idl = JSON.parse(fs.readFileSync('./target/idl/zyncx.json', 'utf-8'));
  const program = new Program(idl, provider);

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
  const amount = 10_000_000; // 0.01 SOL

  console.log(`Depositing: ${amount / 1e9} SOL`);
  console.log(`Precommitment: ${precommitment.toString('hex')}\n`);

  try {
    const tx = await program.methods
      .depositNative(
        amount,
        Array.from(precommitment)
      )
      .accounts({
        depositor: wallet.publicKey,
        vault,
        merkleTree,
        vaultTreasury,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log('✅ Deposit successful!');
    console.log(`Signature: ${tx}`);
    console.log(`🔗 https://explorer.solana.com/tx/${tx}?cluster=devnet`);
  } catch (error: any) {
    console.error('❌ Deposit failed:', error.message);
    if (error.logs) {
      console.error('\nProgram logs:');
      error.logs.forEach((log: string) => console.error(log));
    }
  }
}

main().catch(console.error);
