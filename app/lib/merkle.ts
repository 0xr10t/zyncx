/**
 * Merkle Tree Path Computation for ZYNCX
 * 
 * Computes authentication paths for Merkle tree membership proofs
 */

import { poseidonHash } from './crypto';

export const TREE_DEPTH = 20; // Must match circuit depth

/**
 * Merkle tree node representation
 */
export interface MerkleNode {
  hash: Uint8Array;
  left?: MerkleNode;
  right?: MerkleNode;
}

/**
 * Merkle path for a specific leaf
 */
export interface MerklePath {
  path: Uint8Array[];      // Sibling hashes along the path
  indices: number[];       // Path indices (0=left, 1=right)
  root: Uint8Array;        // Computed root
}

/**
 * Compute Merkle path for a commitment in the tree
 * 
 * @param leaves - All leaf commitments in the tree
 * @param targetCommitment - The commitment to find path for
 * @returns Merkle path or null if commitment not found
 */
export function computeMerklePath(
  leaves: Uint8Array[],
  targetCommitment: Uint8Array
): MerklePath | null {
  // Find the index of target commitment
  const leafIndex = leaves.findIndex(leaf => 
    arraysEqual(leaf, targetCommitment)
  );
  
  if (leafIndex === -1) {
    return null; // Commitment not found
  }

  const path: Uint8Array[] = [];
  const indices: number[] = [];
  
  // Pad leaves to next power of 2
  const paddedLeaves = padLeaves(leaves);
  let currentLevel = paddedLeaves;
  let currentIndex = leafIndex;

  // Build path from leaf to root
  for (let level = 0; level < TREE_DEPTH; level++) {
    const isRightNode = currentIndex % 2 === 1;
    const siblingIndex = isRightNode ? currentIndex - 1 : currentIndex + 1;
    
    // Get sibling hash (or zero if out of bounds)
    const sibling = siblingIndex < currentLevel.length
      ? currentLevel[siblingIndex]
      : getZeroValue(level);
    
    path.push(sibling);
    indices.push(isRightNode ? 1 : 0);
    
    // Move to parent level
    currentLevel = computeParentLevel(currentLevel, level);
    currentIndex = Math.floor(currentIndex / 2);
  }

  // Compute root
  const root = computeRootFromPath(targetCommitment, path, indices);

  return { path, indices, root };
}

/**
 * Compute Merkle root from a leaf and its path
 */
export function computeRootFromPath(
  leaf: Uint8Array,
  path: Uint8Array[],
  indices: number[]
): Uint8Array {
  let current = leaf;

  for (let i = 0; i < path.length; i++) {
    const sibling = path[i];
    const isRight = indices[i] === 1;
    
    // Hash in correct order based on position
    current = isRight
      ? poseidonHash(sibling, current)  // Current is right child
      : poseidonHash(current, sibling); // Current is left child
  }

  return current;
}

/**
 * Compute parent level from current level
 */
function computeParentLevel(
  currentLevel: Uint8Array[],
  level: number
): Uint8Array[] {
  const parentLevel: Uint8Array[] = [];
  const zeroValue = getZeroValue(level);

  for (let i = 0; i < currentLevel.length; i += 2) {
    const left = currentLevel[i];
    const right = i + 1 < currentLevel.length
      ? currentLevel[i + 1]
      : zeroValue;
    
    const parent = poseidonHash(left, right);
    parentLevel.push(parent);
  }

  return parentLevel;
}

/**
 * Pad leaves array to next power of 2
 */
function padLeaves(leaves: Uint8Array[]): Uint8Array[] {
  const targetSize = Math.pow(2, TREE_DEPTH);
  const padded = [...leaves];
  
  while (padded.length < targetSize) {
    padded.push(getZeroValue(0));
  }
  
  return padded;
}

/**
 * Get zero value for a given tree level
 * Must match the circuit's get_zero_value function
 */
export function getZeroValue(level: number): Uint8Array {
  // Level 0 uses hash of 0
  if (level === 0) {
    return poseidonHash(new Uint8Array(32));
  }
  
  // Each level's zero is hash of two zeros from level below
  const prevZero = getZeroValue(level - 1);
  return poseidonHash(prevZero, prevZero);
}

/**
 * Compare two Uint8Arrays for equality
 */
function arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

/**
 * Fetch Merkle tree state from on-chain account
 * 
 * @param program - Anchor program instance
 * @param merkleTreePDA - Public key of Merkle tree account
 * @returns Array of leaf commitments
 */
export async function fetchMerkleTreeLeaves(
  program: any,
  merkleTreePDA: any
): Promise<Uint8Array[]> {
  try {
    const merkleTreeAccount = await program.account.merkleTreeState.fetch(
      merkleTreePDA
    );
    
    // Extract leaves from the tree
    // The tree stores leaves as [u8; 32] arrays
    const leaves: Uint8Array[] = [];
    const leavesData = merkleTreeAccount.leaves;
    
    for (let i = 0; i < merkleTreeAccount.nextIndex; i++) {
      const leaf = new Uint8Array(leavesData.slice(i * 32, (i + 1) * 32));
      leaves.push(leaf);
    }
    
    return leaves;
  } catch (error) {
    console.error('Failed to fetch Merkle tree:', error);
    throw new Error(`Failed to fetch Merkle tree: ${error}`);
  }
}

/**
 * Verify a Merkle proof locally (for testing)
 */
export function verifyMerklePath(
  leaf: Uint8Array,
  path: Uint8Array[],
  indices: number[],
  expectedRoot: Uint8Array
): boolean {
  const computedRoot = computeRootFromPath(leaf, path, indices);
  return arraysEqual(computedRoot, expectedRoot);
}
