/**
 * Confidential Swap Hook for ZYNCX Privacy Protocol
 * Uses Arcium MXE for encrypted swap amounts
 */

import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useState, useCallback, useEffect } from 'react';
import { 
  getProgram, 
  getVaultPDA, 
  getEncryptedVaultPDA, 
  getEncryptedPositionPDA,
  getSwapRequestPDA,
  PROGRAM_ID
} from '../program';
import { 
  initArcium, 
  encryptSwapInput, 
  encryptSwapBounds, 
  getEncryptionPubkey,
  decryptSwapResult,
  nonceToNumber
} from '../arcium';

export interface ConfidentialSwapParams {
  amountIn: bigint;           // Will be encrypted!
  minAmountOut: bigint;       // Will be encrypted!
  maxSlippageBps: number;     // e.g., 50 = 0.5%
  srcToken: PublicKey;
  dstToken: PublicKey;
  aggressive?: boolean;       // Execute even if slightly unfavorable
}

export interface SwapStatus {
  pending: boolean;
  completed: boolean;
  failed: boolean;
  result?: {
    amountOut: bigint;
    signature: string;
  };
}

export function useConfidentialSwap() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [isSwapping, setIsSwapping] = useState(false);
  const [swapStatus, setSwapStatus] = useState<SwapStatus>({
    pending: false,
    completed: false,
    failed: false,
  });
  const [error, setError] = useState<string | null>(null);

  // Initialize Arcium on mount
  useEffect(() => {
    initArcium().catch(console.error);
  }, []);

  /**
   * Queue a confidential swap
   * The swap amount is encrypted - only you and MXE can see it
   */
  const queueSwap = useCallback(async (params: ConfidentialSwapParams): Promise<string> => {
    if (!wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    setIsSwapping(true);
    setError(null);
    setSwapStatus({ pending: true, completed: false, failed: false });

    try {
      await initArcium();
      const program = getProgram(connection, wallet);

      // 1. Encrypt swap amount (THIS IS THE KEY PRIVACY FEATURE!)
      const encryptedAmount = await encryptSwapInput(params.amountIn);

      // 2. Encrypt swap bounds
      const encryptedBounds = await encryptSwapBounds(
        params.minAmountOut,
        params.maxSlippageBps,
        params.aggressive || false
      );

      // 3. Get user's encryption pubkey
      const userEncryptionPubkey = getEncryptionPubkey();

      // 4. Derive PDAs
      const [srcVault] = getVaultPDA(params.srcToken);
      const [encVault] = getEncryptedVaultPDA(params.srcToken);
      const [encPosition] = getEncryptedPositionPDA(srcVault, wallet.publicKey);

      // Get computation offset from cluster state (mock for now)
      const computationOffset = Date.now();
      const [swapRequest] = getSwapRequestPDA(new BN(computationOffset));

      // 5. Build and send queue transaction
      const tx = await program.methods
        .queueConfidentialSwap({
          userEncryptionPubkey: Array.from(userEncryptionPubkey),
          amountNonce: new BN(nonceToNumber(encryptedAmount.nonce).toString()),
          encryptedAmount: Array.from(encryptedAmount.ciphertext),
          boundsNonce: new BN(nonceToNumber(encryptedBounds.nonce).toString()),
          encryptedBounds: Array.from(encryptedBounds.ciphertext),
          srcToken: params.srcToken,
          dstToken: params.dstToken,
        })
        .accounts({
          user: wallet.publicKey,
          vault: srcVault,
          encryptedVault: encVault,
          userPosition: encPosition,
          swapRequest,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      tx.feePayer = wallet.publicKey;
      
      const signed = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      // Start listening for callback
      subscribeToSwapCompletion(swapRequest);

      return signature;
    } catch (err: any) {
      const errorMessage = err.message || 'Swap failed';
      setError(errorMessage);
      setSwapStatus({ pending: false, completed: false, failed: true });
      throw err;
    } finally {
      setIsSwapping(false);
    }
  }, [connection, wallet]);

  /**
   * Subscribe to swap completion events
   */
  const subscribeToSwapCompletion = useCallback((swapRequestPDA: PublicKey) => {
    const SWAP_STATUS_OFFSET = 8; // After discriminator
    const RESULT_OFFSET = 200;    // Approximate offset for encrypted result
    const NONCE_OFFSET = 264;     // Approximate offset for result nonce

    const subscriptionId = connection.onAccountChange(
      swapRequestPDA,
      async (accountInfo) => {
        try {
          const data = accountInfo.data;
          const status = data[SWAP_STATUS_OFFSET];
          
          // Status: 0 = Pending, 1 = Completed, 2 = Failed
          if (status === 1) {
            // Extract encrypted result
            const encryptedResult = data.slice(RESULT_OFFSET, RESULT_OFFSET + 64);
            const resultNonce = data.slice(NONCE_OFFSET, NONCE_OFFSET + 16);
            
            // Decrypt with user's key
            const result = await decryptSwapResult(encryptedResult, resultNonce);
            
            setSwapStatus({
              pending: false,
              completed: true,
              failed: false,
              result: {
                amountOut: result.minAmountOut,
                signature: 'callback_signature', // Would come from event
              },
            });
            
            // Cleanup subscription
            connection.removeAccountChangeListener(subscriptionId);
          } else if (status === 2) {
            setSwapStatus({ pending: false, completed: false, failed: true });
            connection.removeAccountChangeListener(subscriptionId);
          }
        } catch (err) {
          console.error('Error processing swap callback:', err);
        }
      },
      'confirmed'
    );

    // Timeout after 60 seconds
    setTimeout(() => {
      connection.removeAccountChangeListener(subscriptionId);
      if (swapStatus.pending) {
        setSwapStatus({ pending: false, completed: false, failed: true });
        setError('Swap timed out');
      }
    }, 60000);
  }, [connection, swapStatus.pending]);

  return { 
    queueSwap, 
    isSwapping, 
    swapStatus, 
    error,
    resetStatus: () => setSwapStatus({ pending: false, completed: false, failed: false }),
  };
}
