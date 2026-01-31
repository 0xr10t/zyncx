/**
 * Deposit Hook for ZYNCX Privacy Protocol
 */

import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useState } from 'react';
import { getProgram, getVaultPDA, getMerkleTreePDA, getVaultTreasuryPDA } from '../program';
import { generateDepositSecrets, computePrecommitment, createDepositNote, DepositNote } from '../crypto';

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
      const program = getProgram(connection, wallet);
      const amountLamports = BigInt(Math.floor(amountSol * 1e9)); // SOL to lamports

      // Generate secrets (CRITICAL: User must save these!)
      const { secret, nullifierSecret } = generateDepositSecrets();
      const precommitment = computePrecommitment(secret, nullifierSecret);

      // Derive PDAs
      const NATIVE_MINT = PublicKey.default; // Zero pubkey for SOL
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      const [vaultTreasury] = getVaultTreasuryPDA(vault);

      // Build transaction
      const tx = await program.methods
        .depositNative(
          new BN(amountLamports.toString()),
          Array.from(precommitment)
        )
        .accounts({
          depositor: wallet.publicKey,
          vault,
          merkleTree,
          vaultTreasury,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      // Send transaction
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
