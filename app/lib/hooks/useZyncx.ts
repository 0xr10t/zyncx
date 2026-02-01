'use client';

import { useCallback, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { 
  PublicKey, 
  Transaction, 
  SystemProgram, 
  LAMPORTS_PER_SOL,
  TransactionInstruction,
} from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { 
  PROGRAM_ID, 
  NATIVE_MINT, 
  getVaultPDA, 
  getMerkleTreePDA, 
  getVaultTreasuryPDA,
  getNullifierPDA,
} from '../program';
import {
  generateDepositSecrets,
  computePrecommitment,
  computeNullifierHash,
  createDepositNote,
  parseDepositNote,
  generateMockProof,
  hexToBytes,
  DepositNote,
} from '../crypto';

export interface ZyncxState {
  isLoading: boolean;
  error: string | null;
  lastTx: string | null;
}

export function useZyncx() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [state, setState] = useState<ZyncxState>({
    isLoading: false,
    error: null,
    lastTx: null,
  });

  // Deposit SOL into the privacy vault
  const depositSol = useCallback(async (amountSol: number): Promise<DepositNote | null> => {
    if (!wallet.publicKey || !wallet.signTransaction) {
      setState(s => ({ ...s, error: 'Wallet not connected' }));
      return null;
    }

    setState({ isLoading: true, error: null, lastTx: null });

    try {
      const amountLamports = BigInt(Math.floor(amountSol * LAMPORTS_PER_SOL));
      
      // Generate secrets
      const { secret, nullifierSecret } = generateDepositSecrets();
      const precommitment = computePrecommitment(secret, nullifierSecret);
      
      // Create deposit note (user must save this!)
      const note = createDepositNote(secret, nullifierSecret, amountLamports);

      // Derive PDAs
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      const [vaultTreasury] = getVaultTreasuryPDA(vault);

      // Build deposit instruction
      // Instruction discriminator for deposit_native (from IDL)
      const discriminator = new Uint8Array([13, 158, 13, 223, 95, 213, 28, 6]);
      
      // Write amount as little-endian u64
      const amountBuffer = new Uint8Array(8);
      const amountView = new DataView(amountBuffer.buffer);
      amountView.setBigUint64(0, amountLamports, true); // little-endian
      
      // Concatenate all parts
      const instructionDataArray = new Uint8Array(8 + 8 + 32);
      instructionDataArray.set(discriminator, 0);
      instructionDataArray.set(amountBuffer, 8);
      instructionDataArray.set(precommitment, 16);

      const depositIx = new TransactionInstruction({
        keys: [
          { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
          { pubkey: vault, isSigner: false, isWritable: true },
          { pubkey: merkleTree, isSigner: false, isWritable: true },
          { pubkey: vaultTreasury, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: PROGRAM_ID,
        data: Buffer.from(instructionDataArray),
      });

      // Create and send transaction
      const tx = new Transaction().add(depositIx);
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      const signedTx = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signedTx.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      note.txSignature = signature;
      setState({ isLoading: false, error: null, lastTx: signature });
      
      return note;
    } catch (error: any) {
      console.error('Deposit error:', error);
      setState({ isLoading: false, error: error.message || 'Deposit failed', lastTx: null });
      return null;
    }
  }, [connection, wallet]);

  // Withdraw SOL from the privacy vault using a deposit note
  const withdrawSol = useCallback(async (
    note: DepositNote,
    withdrawAmount?: number, // Optional: partial withdrawal amount in SOL
    recipientAddress?: string
  ): Promise<string | null> => {
    if (!wallet.publicKey || !wallet.signTransaction) {
      setState(s => ({ ...s, error: 'Wallet not connected' }));
      return null;
    }

    setState({ isLoading: true, error: null, lastTx: null });

    try {
      const { nullifierSecret, amount: noteAmount } = parseDepositNote(note);
      const nullifierHash = computeNullifierHash(nullifierSecret);
      const recipient = recipientAddress 
        ? new PublicKey(recipientAddress) 
        : wallet.publicKey;

      // Calculate withdrawal amount (default to full note amount)
      const amount = withdrawAmount 
        ? BigInt(Math.floor(withdrawAmount * LAMPORTS_PER_SOL))
        : noteAmount;

      // Derive PDAs
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      const [vaultTreasury] = getVaultTreasuryPDA(vault);
      const [nullifierPDA] = getNullifierPDA(vault, nullifierHash);

      // Verifier program ID from IDL
      const VERIFIER_PROGRAM = new PublicKey('AWUEQfGnU2nVYAA3dfKpckDhqjoW6HELT5wvkg9Sve1y');

      // Generate mock proof for demo
      const proof = generateMockProof();
      
      // New commitment (empty for full withdrawal, would contain new commitment for partial)
      const newCommitment = new Uint8Array(32);

      // Build withdraw instruction
      // Instruction discriminator for withdraw_native (from IDL)
      const discriminator = new Uint8Array([113, 227, 26, 32, 53, 66, 90, 250]);
      
      // Write amount as little-endian u64
      const amountBuffer = new Uint8Array(8);
      const amountView = new DataView(amountBuffer.buffer);
      amountView.setBigUint64(0, amount, true); // little-endian

      // Encode proof length as Borsh Vec<u8> (4 bytes little-endian)
      const proofLenBuffer = new Uint8Array(4);
      const proofLenView = new DataView(proofLenBuffer.buffer);
      proofLenView.setUint32(0, proof.length, true);

      // Concatenate all parts: discriminator(8) + amount(8) + nullifier(32) + commitment(32) + proofLen(4) + proof(256)
      const totalLen = 8 + 8 + 32 + 32 + 4 + proof.length;
      const instructionDataArray = new Uint8Array(totalLen);
      let offset = 0;
      instructionDataArray.set(discriminator, offset); offset += 8;
      instructionDataArray.set(amountBuffer, offset); offset += 8;
      instructionDataArray.set(nullifierHash, offset); offset += 32;
      instructionDataArray.set(newCommitment, offset); offset += 32;
      instructionDataArray.set(proofLenBuffer, offset); offset += 4;
      instructionDataArray.set(proof, offset);

      // Accounts order from IDL:
      // 1. recipient, 2. vault, 3. merkle_tree, 4. vault_treasury,
      // 5. nullifier_account, 6. verifier_program, 7. payer, 8. system_program
      const withdrawIx = new TransactionInstruction({
        keys: [
          { pubkey: recipient, isSigner: false, isWritable: true }, // recipient
          { pubkey: vault, isSigner: false, isWritable: true }, // vault
          { pubkey: merkleTree, isSigner: false, isWritable: true }, // merkle_tree
          { pubkey: vaultTreasury, isSigner: false, isWritable: true }, // vault_treasury
          { pubkey: nullifierPDA, isSigner: false, isWritable: true }, // nullifier_account
          { pubkey: VERIFIER_PROGRAM, isSigner: false, isWritable: false }, // verifier_program
          { pubkey: wallet.publicKey, isSigner: true, isWritable: true }, // payer
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system_program
        ],
        programId: PROGRAM_ID,
        data: Buffer.from(instructionDataArray),
      });

      // Create and send transaction
      const tx = new Transaction().add(withdrawIx);
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

      const signedTx = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signedTx.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      setState({ isLoading: false, error: null, lastTx: signature });
      return signature;
    } catch (error: any) {
      console.error('Withdraw error:', error);
      setState({ isLoading: false, error: error.message || 'Withdrawal failed', lastTx: null });
      return null;
    }
  }, [connection, wallet]);

  // Fetch vault state
  const getVaultInfo = useCallback(async () => {
    try {
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [vaultTreasury] = getVaultTreasuryPDA(vault);
      
      const treasuryBalance = await connection.getBalance(vaultTreasury);
      const vaultAccount = await connection.getAccountInfo(vault);
      
      return {
        vaultAddress: vault.toBase58(),
        treasuryAddress: vaultTreasury.toBase58(),
        treasuryBalance: treasuryBalance / LAMPORTS_PER_SOL,
        isInitialized: vaultAccount !== null,
      };
    } catch (error) {
      console.error('Error fetching vault info:', error);
      return null;
    }
  }, [connection]);

  return {
    ...state,
    depositSol,
    withdrawSol,
    getVaultInfo,
  };
}
