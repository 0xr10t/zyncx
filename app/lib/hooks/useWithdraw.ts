/**
 * Withdrawal Hook for ZYNCX Privacy Protocol
 * Supports both full and partial withdrawals with ZK proofs
 */

import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useState, useCallback } from 'react';
import { 
  getProgram, 
  getVaultPDA, 
  getMerkleTreePDA, 
  getVaultTreasuryPDA, 
  getNullifierPDA 
} from '../program';
import { 
  DepositNote, 
  generateDepositSecrets, 
  computePrecommitment, 
  computeCommitment,
  computeNullifierHash,
  hexToBytes,
  bytesToHex,
  createDepositNote
} from '../crypto';
import { generateWithdrawProof, initProver } from '../prover';
import { fetchMerklePath } from '../merkle';

export interface WithdrawParams {
  note: DepositNote;
  withdrawAmount: bigint;
  recipient: PublicKey;
}

export interface WithdrawResult {
  signature: string;
  changeNote?: DepositNote;
}

export function useWithdraw() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [isWithdrawing, setIsWithdrawing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const withdraw = useCallback(async (params: WithdrawParams): Promise<WithdrawResult> => {
    if (!wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    setIsWithdrawing(true);
    setError(null);

    try {
      // Initialize prover if needed
      await initProver();
      
      const program = getProgram(connection, wallet);
      const totalAmount = BigInt(params.note.amount);
      const isPartial = params.withdrawAmount < totalAmount;

      // Validate inputs
      if (params.withdrawAmount <= BigInt(0)) {
        throw new Error('Withdrawal amount must be positive');
      }
      if (params.withdrawAmount > totalAmount) {
        throw new Error('Cannot withdraw more than deposited');
      }

      // Parse secrets from note
      const secret = hexToBytes(params.note.secret);
      const nullifierSecret = hexToBytes(params.note.nullifierSecret);

      // Generate new secrets for change (if partial withdrawal)
      let newSecret: Uint8Array = new Uint8Array(32);
      let newNullifierSecret: Uint8Array = new Uint8Array(32);
      let newCommitment: Uint8Array = new Uint8Array(32);
      let changeNote: DepositNote | undefined;

      if (isPartial) {
        const changeSecrets = generateDepositSecrets();
        newSecret = new Uint8Array(changeSecrets.secret);
        newNullifierSecret = new Uint8Array(changeSecrets.nullifierSecret);
        
        const changeAmount = totalAmount - params.withdrawAmount;
        const changePrecommitment = computePrecommitment(newSecret, newNullifierSecret);
        newCommitment = new Uint8Array(computeCommitment(changeAmount, changePrecommitment));
        
        // Create change note for user to save
        changeNote = createDepositNote(newSecret, newNullifierSecret, changeAmount);
      }

      // Fetch Merkle path from on-chain
      const NATIVE_MINT = PublicKey.default;
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      
      // @ts-ignore - Account name comes from IDL
      const merkleAccount = await program.account.merkleTreeState.fetch(merkleTree);
      const commitment = hexToBytes(params.note.commitment!);
      const { path, indices, root } = fetchMerklePath(merkleAccount, commitment);

      // Compute nullifier hash
      const nullifierHash = computeNullifierHash(nullifierSecret);

      // Generate ZK proof
      const proof = await generateWithdrawProof({
        secret,
        nullifierSecret,
        merklePath: path,
        pathIndices: indices,
        root,
        nullifierHash,
        recipient: params.recipient.toBytes(),
        amount: params.withdrawAmount,
      });

      // Derive PDAs
      const [vaultTreasury] = getVaultTreasuryPDA(vault);
      const [nullifierPDA] = getNullifierPDA(vault, nullifierHash);

      // Build and send transaction
      const tx = await program.methods
        .withdrawNative(
          new BN(params.withdrawAmount.toString()),
          Array.from(nullifierHash),
          Array.from(newCommitment),
          Array.from(proof)
        )
        .accounts({
          recipient: params.recipient,
          vault,
          merkleTree,
          vaultTreasury,
          nullifierAccount: nullifierPDA,
          payer: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      tx.feePayer = wallet.publicKey;
      
      const signed = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      return { signature, changeNote };
    } catch (err: any) {
      const errorMessage = err.message || 'Withdrawal failed';
      setError(errorMessage);
      throw err;
    } finally {
      setIsWithdrawing(false);
    }
  }, [connection, wallet]);

  return { withdraw, isWithdrawing, error };
}
