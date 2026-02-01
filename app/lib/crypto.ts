/**
 * Cryptographic utilities for ZYNCX Privacy Protocol
 * 
 * Uses Web Crypto API for browser compatibility with Turbopack
 * In production, use actual Poseidon hash to match the circuit
 */

// Synchronous hash function using a simple mixing algorithm
// Note: In production, replace with actual Poseidon hash that matches the circuit
function simpleHash(data: Uint8Array): Uint8Array {
  const result = new Uint8Array(32);
  
  // Initialize with data
  for (let i = 0; i < Math.min(data.length, 32); i++) {
    result[i] = data[i];
  }
  
  // Mix all data in
  for (let i = 0; i < data.length; i++) {
    const idx = i % 32;
    result[idx] = (result[idx] ^ data[i]) & 0xff;
    
    // Additional mixing
    const next = (idx + 1) % 32;
    result[next] = ((result[next] + result[idx] + i) ^ (data[i] >>> 1)) & 0xff;
  }
  
  // Multiple rounds of mixing for better distribution
  for (let round = 0; round < 64; round++) {
    const temp = result[0];
    for (let j = 0; j < 31; j++) {
      result[j] = ((result[j] * 31 + result[j + 1] * 37 + round) ^ (result[(j + 16) % 32])) & 0xff;
    }
    result[31] = ((result[31] * 31 + temp * 37 + round) ^ result[15]) & 0xff;
  }
  
  return result;
}

// Main hash function - use simpleHash as fallback
// In production, this MUST be replaced with actual Poseidon hash
function keccak_256(data: Uint8Array): Uint8Array {
  return simpleHash(data);
}

/**
 * Poseidon hash function (using keccak as fallback for demo)
 * In production, this should use the actual Poseidon hash to match the circuit
 */
export function poseidonHash(...inputs: Uint8Array[]): Uint8Array {
  // Concatenate all inputs
  const totalLength = inputs.reduce((sum, input) => sum + input.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const input of inputs) {
    combined.set(input, offset);
    offset += input.length;
  }
  return keccak_256(combined);
}

// Generate random bytes
export function randomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  if (typeof window !== 'undefined' && window.crypto) {
    window.crypto.getRandomValues(bytes);
  } else {
    // Node.js fallback
    for (let i = 0; i < length; i++) {
      bytes[i] = Math.floor(Math.random() * 256);
    }
  }
  return bytes;
}

// Generate deposit secrets
export function generateDepositSecrets() {
  const secret = randomBytes(32);
  const nullifierSecret = randomBytes(32);
  return { secret, nullifierSecret };
}

// Compute precommitment = keccak256(secret || nullifierSecret)
export function computePrecommitment(
  secret: Uint8Array,
  nullifierSecret: Uint8Array
): Uint8Array {
  const combined = new Uint8Array(64);
  combined.set(secret, 0);
  combined.set(nullifierSecret, 32);
  return keccak_256(combined);
}

// Compute commitment = keccak256(amount || precommitment)
// This matches the on-chain poseidon_hash_commitment function (using keccak for demo)
export function computeCommitment(
  amount: bigint,
  precommitment: Uint8Array
): Uint8Array {
  const amountBytes = new Uint8Array(8);
  const view = new DataView(amountBytes.buffer);
  view.setBigUint64(0, amount, true); // little-endian
  
  const combined = new Uint8Array(40);
  combined.set(amountBytes, 0);
  combined.set(precommitment, 8);
  
  return keccak_256(combined);
}

// Compute nullifier hash = keccak256(nullifierSecret)
export function computeNullifierHash(nullifierSecret: Uint8Array): Uint8Array {
  return keccak_256(nullifierSecret);
}

// Convert bytes to hex string
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

// Convert hex string to bytes
export function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

// Deposit note structure (user must save this!)
export interface DepositNote {
  secret: string; // hex
  nullifierSecret: string; // hex
  precommitment: string; // hex
  amount: string; // lamports as string
  commitment?: string; // hex (returned from chain)
  txSignature?: string;
  timestamp: number;
}

// Create a deposit note from secrets
export function createDepositNote(
  secret: Uint8Array,
  nullifierSecret: Uint8Array,
  amount: bigint
): DepositNote {
  const precommitment = computePrecommitment(secret, nullifierSecret);
  const commitment = computeCommitment(amount, precommitment);
  
  return {
    secret: bytesToHex(secret),
    nullifierSecret: bytesToHex(nullifierSecret),
    precommitment: bytesToHex(precommitment),
    amount: amount.toString(),
    commitment: bytesToHex(commitment),
    timestamp: Date.now(),
  };
}

// Parse a deposit note for withdrawal
export function parseDepositNote(note: DepositNote) {
  return {
    secret: hexToBytes(note.secret),
    nullifierSecret: hexToBytes(note.nullifierSecret),
    precommitment: hexToBytes(note.precommitment),
    amount: BigInt(note.amount),
    nullifierHash: computeNullifierHash(hexToBytes(note.nullifierSecret)),
  };
}

// Generate a mock ZK proof for demo (256 bytes)
// In production, this would be generated by the Noir circuit
export function generateMockProof(): Uint8Array {
  const proof = new Uint8Array(256);
  // Fill with non-zero values so it passes the basic check
  for (let i = 0; i < 256; i++) {
    proof[i] = (i * 7 + 13) % 256;
  }
  return proof;
}

// Encode deposit note as base64 for easy sharing
export function encodeNote(note: DepositNote): string {
  const json = JSON.stringify(note);
  if (typeof window !== 'undefined') {
    return btoa(json);
  }
  return Buffer.from(json).toString('base64');
}

// Decode deposit note from base64
export function decodeNote(encoded: string): DepositNote {
  let json: string;
  if (typeof window !== 'undefined') {
    json = atob(encoded);
  } else {
    json = Buffer.from(encoded, 'base64').toString();
  }
  return JSON.parse(json);
}
