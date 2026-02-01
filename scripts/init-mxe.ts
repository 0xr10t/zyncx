import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { AnchorProvider, Program, Wallet } from '@coral-xyz/anchor';
import * as fs from 'fs';

// Your deployed program ID
const PROGRAM_ID = new PublicKey('6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC');
const CLUSTER_OFFSET = 456;

async function main() {
  console.log('🔧 Initializing Arcium MXE for ZYNCX...\n');

  // Setup connection
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  const walletKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(
      fs.readFileSync(process.env.HOME + '/.config/solana/id.json', 'utf-8')
    ))
  );
  const wallet = new Wallet(walletKeypair);
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });

  console.log(`Wallet: ${wallet.publicKey.toString()}`);
  console.log(`Program: ${PROGRAM_ID.toString()}`);
  console.log(`Cluster Offset: ${CLUSTER_OFFSET}\n`);

  // Load program IDL
  const idl = JSON.parse(fs.readFileSync('./target/idl/zyncx.json', 'utf-8'));
  const program = new Program(idl, provider);

  try {
    // Initialize init_vault computation definition
    console.log('1️⃣  Initializing init_vault computation definition...');
    const tx1 = await program.methods
      .initVaultCompDef()
      .accounts({
        payer: wallet.publicKey,
        // Arcium will auto-derive other accounts
      })
      .rpc();
    console.log(`   ✅ Signature: ${tx1}\n`);

    // Initialize process_deposit computation definition
    console.log('2️⃣  Initializing process_deposit computation definition...');
    const tx2 = await program.methods
      .initProcessDepositCompDef()
      .accounts({
        payer: wallet.publicKey,
      })
      .rpc();
    console.log(`   ✅ Signature: ${tx2}\n`);

    // Initialize confidential_swap computation definition
    console.log('3️⃣  Initializing confidential_swap computation definition...');
    const tx3 = await program.methods
      .initConfidentialSwapCompDef()
      .accounts({
        payer: wallet.publicKey,
      })
      .rpc();
    console.log(`   ✅ Signature: ${tx3}\n`);

    console.log('✅ All computation definitions initialized successfully!');
    console.log('\n📝 Next steps:');
    console.log('   1. Initialize a vault: ts-node scripts/init-vault.ts');
    console.log('   2. Test deposits/withdrawals');
    console.log('   3. Test confidential swaps\n');

  } catch (error: any) {
    console.error('❌ Error:', error.message);
    if (error.logs) {
      console.error('Logs:', error.logs);
    }
    process.exit(1);
  }
}

main().catch(console.error);
