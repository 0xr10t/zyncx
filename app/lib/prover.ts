/**
 * Noir ZK Proof Generation for ZYNCX Privacy Protocol
 * 
 * This module handles generating zero-knowledge proofs for private withdrawals
 * using the Noir circuit compiled in mixer/target/mixer.json
 */

import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import circuit from '../../mixer/target/mixer.json';

// Singleton instances
let noir: Noir | null = null;
let backend: BarretenbergBackend | null = null;

/**
 * Initialize the Noir prover and backend
 * This should be called once before generating proofs
 */
export async function initProver() {
  if (!noir) {
    // @ts-ignore - Circuit JSON format compatibility
    backend = new BarretenbergBackend(circuit);
    // @ts-ignore - Circuit JSON format compatibility
    noir = new Noir(circuit);
  }
  return { noir, backend };
}

/**
 * Input structure for withdraw proof generation
 */
export interface WithdrawProofInputs {
  // Private inputs (known only to user)
  secret: Uint8Array;              // 32-byte secret from deposit
  nullifierSecret: Uint8Array;     // 32-byte nullifier secret
  merklePath: Uint8Array[];        // Array of 32-byte sibling hashes
  pathIndices: number[];           // Binary path (0=left, 1=right)
  
  // Public inputs (visible on-chain)
  root: Uint8Array;                // Current Merkle tree root
  nullifierHash: Uint8Array;       // Hash of nullifier (prevents double-spend)
  recipient: Uint8Array;           // Recipient's public key (32 bytes)
  amount: bigint;                  // Withdrawal amount in lamports
}

/**
 * Convert Uint8Array to Field element (bigint)
 */
function bytesToField(bytes: Uint8Array): string {
  // Convert bytes to hex string, then to bigint
  const hex = Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  return BigInt('0x' + hex).toString();
}

/**
 * Generate a zero-knowledge proof for a private withdrawal
 * 
 * The proof demonstrates:
 * 1. Knowledge of secret and nullifier_secret
 * 2. The commitment exists in the Merkle tree
 * 3. The nullifier is correctly computed
 * 4. The withdrawal amount matches the deposit
 * 
 * @param inputs - Proof inputs (private and public)
 * @returns The generated proof as Uint8Array
 */
export async function generateWithdrawProof(
  inputs: WithdrawProofInputs
): Promise<Uint8Array> {
  const { noir, backend } = await initProver();
  
  if (!noir || !backend) {
    throw new Error('Prover not initialized');
  }

  // Convert inputs to Noir-compatible format
  const witnessInputs = {
    // Private inputs
    secret: bytesToField(inputs.secret),
    nullifier_secret: bytesToField(inputs.nullifierSecret),
    merkle_path: inputs.merklePath.map(p => bytesToField(p)),
    path_indices: inputs.pathIndices.map(i => i.toString()),
    
    // Public inputs
    root: bytesToField(inputs.root),
    nullifier_hash: bytesToField(inputs.nullifierHash),
    recipient: bytesToField(inputs.recipient),
    amount: inputs.amount.toString(),
  };

  try {
    // Execute the circuit to generate witness
    const { witness } = await noir.execute(witnessInputs);
    
    // Generate the proof using Barretenberg backend
    const proof = await backend.generateProof(witness);
    
    return proof.proof;
  } catch (error) {
    console.error('Proof generation failed:', error);
    throw new Error(`Failed to generate proof: ${error}`);
  }
}

/**
 * Verify a proof (useful for testing)
 * 
 * @param proof - The proof to verify
 * @param publicInputs - Public inputs used in proof generation
 * @returns true if proof is valid
 */
export async function verifyProof(
  proof: Uint8Array,
  publicInputs: {
    root: Uint8Array;
    nullifierHash: Uint8Array;
    recipient: Uint8Array;
    amount: bigint;
  }
): Promise<boolean> {
  const { backend } = await initProver();
  
  if (!backend) {
    throw new Error('Backend not initialized');
  }

  const publicInputsFormatted = {
    root: bytesToField(publicInputs.root),
    nullifier_hash: bytesToField(publicInputs.nullifierHash),
    recipient: bytesToField(publicInputs.recipient),
    amount: publicInputs.amount.toString(),
  };

  try {
    const isValid = await backend.verifyProof({
      proof,
      publicInputs: Object.values(publicInputsFormatted),
    });
    
    return isValid;
  } catch (error) {
    console.error('Proof verification failed:', error);
    return false;
  }
}

/**
 * Clean up resources
 */
export async function cleanupProver() {
  if (backend) {
    await backend.destroy();
    backend = null;
    noir = null;
  }
}
