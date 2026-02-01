/**
 * Deposit Hook for ZYNCX Privacy Protocol
 * Uses raw transaction instructions for compatibility with Arcium macro
 */

import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import { useState } from 'react';
import { PROGRAM_ID, getVaultPDA, getMerkleTreePDA, getVaultTreasuryPDA } from '../program';
import { generateDepositSecrets, computePrecommitment, createDepositNote, DepositNote } from '../crypto';

// Exact discriminator from IDL for deposit_native instruction
const DEPOSIT_NATIVE_DISCRIMINATOR = Buffer.from([13, 158, 13, 223, 95, 213, 28, 6]);

export function useDeposit() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [isDepositing, setIsDepositing] = useState(false);

  async function depositNative(amountSol: number): Promise<DepositNote> {
    if (!wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    setIsDepositing(true);

    try {
      const amountLamports = BigInt(Math.floor(amountSol * 1e9)); // SOL to lamports

      // Generate secrets (CRITICAL: User must save these!)
      const { secret, nullifierSecret } = generateDepositSecrets();
      const precommitment = computePrecommitment(secret, nullifierSecret);

      // Derive PDAs
      const NATIVE_MINT = new PublicKey(new Uint8Array(32)); // Zero pubkey for SOL
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      const [vaultTreasury] = getVaultTreasuryPDA(vault);

      // Build instruction data manually (bypasses Anchor client issues with Arcium macro)
      // Format: discriminator (8 bytes) + amount (8 bytes u64 LE) + precommitment (32 bytes)
      const amountBuffer = Buffer.alloc(8);
      amountBuffer.writeBigUInt64LE(amountLamports);
      
      const instructionData = Buffer.concat([
        DEPOSIT_NATIVE_DISCRIMINATOR,
        amountBuffer,
        Buffer.from(precommitment),
      ]);

      // Create raw instruction
      const depositIx = new TransactionInstruction({
        keys: [
          { pubkey: wallet.publicKey, isSigner: true, isWritable: true }, // depositor
          { pubkey: vault, isSigner: false, isWritable: true }, // vault
          { pubkey: merkleTree, isSigner: false, isWritable: true }, // merkle_tree
          { pubkey: vaultTreasury, isSigner: false, isWritable: true }, // vault_treasury
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system_program
        ],
        programId: PROGRAM_ID,
        data: instructionData,
      });

      // Build and send transaction
      const tx = new Transaction().add(depositIx);
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      tx.feePayer = wallet.publicKey;
      
      const signed = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      // Create deposit note
      const note = createDepositNote(secret, nullifierSecret, amountLamports);
      note.txSignature = signature;

      return note;
    } catch (error) {
      console.error('Deposit failed:', error);
      throw error;
    } finally {
      setIsDepositing(false);
    }
  }

  return { depositNative, isDepositing };
}
