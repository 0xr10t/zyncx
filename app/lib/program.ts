import { PublicKey, Connection } from '@solana/web3.js';
import { AnchorProvider, Program, Idl, BN } from '@coral-xyz/anchor';

// Program ID - deployed on devnet
export const PROGRAM_ID = new PublicKey('6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6');

// Native SOL mint (zero pubkey represents SOL in our system)
export const NATIVE_MINT = new PublicKey(new Uint8Array(32));

// Devnet RPC
export const DEVNET_RPC = 'https://api.devnet.solana.com';

// Jupiter V6 Program ID
export const JUPITER_PROGRAM_ID = new PublicKey('JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4');

// PDA derivation functions
export function getVaultPDA(assetMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), assetMint.toBuffer()],
    PROGRAM_ID
  );
}

export function getMerkleTreePDA(vault: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('merkle_tree'), vault.toBuffer()],
    PROGRAM_ID
  );
}

export function getNullifierPDA(vault: PublicKey, nullifier: Uint8Array): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), nullifier],
    PROGRAM_ID
  );
}

export function getVaultTreasuryPDA(vault: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('vault_treasury'), vault.toBuffer()],
    PROGRAM_ID
  );
}

export function getArciumConfigPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('arcium_config')],
    PROGRAM_ID
  );
}

export function getComputationRequestPDA(requestId: BN): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('computation'), requestId.toArrayLike(Buffer, 'le', 8)],
    PROGRAM_ID
  );
}

// IDL type definitions (simplified for demo)
export interface VaultState {
  bump: number;
  vaultType: { native: {} } | { alternative: {} };
  assetMint: PublicKey;
  merkleTree: PublicKey;
  nonce: BN;
  authority: PublicKey;
  totalDeposited: BN;
}

export interface MerkleTreeState {
  bump: number;
  depth: number;
  size: BN;
  currentRootIndex: number;
  root: number[];
  roots: number[][];
  leaves: number[][];
}

export interface NullifierState {
  bump: number;
  nullifier: number[];
  spent: boolean;
  spentAt: BN;
  vault: PublicKey;
}
