use dep::poseidon::poseidon::bn254::{hash_1, hash_2, hash_3, hash_4};

// ============================================================================
// ZYNCX PRIVACY CIRCUIT - MULTI-TOKEN + PARTIAL WITHDRAWALS
// ============================================================================
// 
// This circuit enables:
// 1. Shielded deposits with hidden amounts
// 2. Partial withdrawals (withdraw some, keep the rest)
// 3. Cross-token swaps (SOL -> USDC etc.)
// 4. Full withdrawals
// 5. Double-spend prevention via nullifiers
// 6. Front-running protection via recipient binding
//
// CRITICAL: Commitment includes TOKEN_MINT
// =========================================
// commitment = Poseidon(secret, nullifier_secret, amount, token_mint)
//
// This allows:
// - Different vaults for different tokens (SOL vault, USDC vault)
// - Swaps that consume SOL commitment and create USDC commitment
// - Each vault has its own Merkle tree
//
// ============================================================================

global TREE_DEPTH: u32 = 20; // Supports 2^20 = ~1 million commitments

// ============================================================================
// MAIN CIRCUIT - WITHDRAWAL (Partial or Full, Same Token)
// ============================================================================
fn main(
    // Private inputs - original commitment data
    secret: Field,
    nullifier_secret: Field,
    original_amount: Field,
    token_mint: Field,
    merkle_path: [Field; TREE_DEPTH],
    path_indices: [Field; TREE_DEPTH],
    
    // Private inputs - new commitment data (for change)
    new_secret: Field,
    new_nullifier_secret: Field,

    // Public inputs
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    withdraw_amount: pub Field,
    new_commitment: pub Field,
    token_mint_public: pub Field,
) {
    // Step 1: Verify token_mint consistency
    assert(token_mint == token_mint_public, "Token mint mismatch");

    // Step 2: Reconstruct original commitment
    let original_commitment = compute_commitment_with_token(
        secret, nullifier_secret, original_amount, token_mint
    );

    // Step 3: Verify nullifier
    let computed_nullifier = compute_nullifier(nullifier_secret);
    assert(computed_nullifier == nullifier_hash, "Invalid nullifier");

    // Step 4: Verify Merkle membership
    let computed_root = compute_merkle_root(original_commitment, merkle_path, path_indices);
    assert(computed_root == root, "Merkle proof verification failed");

    // Step 5: Verify math
    let remaining_amount = original_amount - withdraw_amount;
    assert(withdraw_amount != 0, "Withdrawal amount cannot be zero");

    // Step 6: Verify new_commitment
    if remaining_amount == 0 {
        assert(new_commitment == 0, "Full withdrawal must have zero new_commitment");
    } else {
        let expected = compute_commitment_with_token(
            new_secret, new_nullifier_secret, remaining_amount, token_mint
        );
        assert(new_commitment == expected, "Invalid new commitment for change");
    }

    // Step 7: Constrain recipient
    assert(recipient != 0, "Invalid recipient address");
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn compute_commitment(secret: Field, nullifier_secret: Field, amount: Field) -> Field {
    hash_3([secret, nullifier_secret, amount])
}

fn compute_commitment_with_token(
    secret: Field, nullifier_secret: Field, amount: Field, token_mint: Field
) -> Field {
    hash_4([secret, nullifier_secret, amount, token_mint])
}

fn compute_nullifier(nullifier_secret: Field) -> Field {
    hash_1([nullifier_secret])
}

fn compute_merkle_root(
    leaf: Field,
    path: [Field; TREE_DEPTH],
    indices: [Field; TREE_DEPTH],
) -> Field {
    let mut current = leaf;
    for i in 0..TREE_DEPTH {
        let path_element = path[i];
        let index = indices[i];
        assert((index == 0) | (index == 1), "Path index must be binary");
        let (left, right) = if index == 0 { (current, path_element) } else { (path_element, current) };
        current = hash_2([left, right]);
    }
    current
}

fn get_zero_value(level: u32) -> Field {
    if level == 0 {
        hash_1([0])
    } else {
        let prev = get_zero_value(level - 1);
        hash_2([prev, prev])
    }
}

// ============================================================================
// SWAP CIRCUIT - CROSS-TOKEN EXCHANGE
// ============================================================================

fn swap_circuit(
    src_secret: Field,
    src_nullifier_secret: Field,
    src_amount: Field,
    src_token_mint: Field,
    merkle_path: [Field; TREE_DEPTH],
    path_indices: [Field; TREE_DEPTH],
    dst_secret: Field,
    dst_nullifier_secret: Field,
    dst_amount: Field,
    src_root: Field,
    src_nullifier_hash: Field,
    src_token_mint_public: Field,
    dst_token_mint_public: Field,
    dst_commitment: Field,
    min_dst_amount: Field,
) {
    assert(src_token_mint == src_token_mint_public, "Source token mint mismatch");

    let src_commitment = compute_commitment_with_token(
        src_secret, src_nullifier_secret, src_amount, src_token_mint
    );

    let computed_nullifier = compute_nullifier(src_nullifier_secret);
    assert(computed_nullifier == src_nullifier_hash, "Invalid source nullifier");

    let computed_root = compute_merkle_root(src_commitment, merkle_path, path_indices);
    assert(computed_root == src_root, "Source commitment not in tree");

    let expected_dst = compute_commitment_with_token(
        dst_secret, dst_nullifier_secret, dst_amount, dst_token_mint_public
    );
    assert(dst_commitment == expected_dst, "Invalid destination commitment");

    // Cast to u64 for comparison (Fields don't support comparison)
    assert(dst_amount as u64 >= min_dst_amount as u64, "Slippage exceeded");
}

// ============================================================================
// TESTS
// ============================================================================

#[test]
fn test_compute_commitment() {
    let commitment = compute_commitment(0x1234, 0x5678, 1000);
    assert(commitment != 0);
}

#[test]
fn test_compute_commitment_with_token() {
    // Use hex values for token mints (simulated SOL and USDC mint addresses)
    let sol_mint = 0x1111111111111111111111111111111111111111111111111111111111111111;
    let usdc_mint = 0x2222222222222222222222222222222222222222222222222222222222222222;
    let c1 = compute_commitment_with_token(0x1234, 0x5678, 1000, sol_mint);
    let c2 = compute_commitment_with_token(0x1234, 0x5678, 1000, usdc_mint);
    assert(c1 != 0);
    assert(c1 != c2);
}

#[test]
fn test_compute_nullifier() {
    let n1 = compute_nullifier(0xabcd);
    let n2 = compute_nullifier(0xabcd);
    let n3 = compute_nullifier(0x9999);
    assert(n1 == n2);
    assert(n1 != n3);
}

#[test]
fn test_merkle_proof_single_leaf() {
    let leaf = 0x1234;
    let zero = get_zero_value(0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut indices: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_zero = zero;
    for i in 0..TREE_DEPTH {
        path[i] = current_zero;
        current_zero = hash_2([current_zero, current_zero]);
    }
    let root = compute_merkle_root(leaf, path, indices);
    assert(root != 0);
}

#[test]
fn test_partial_withdrawal_with_token() {
    let secret = 0x1111;
    let nullifier_secret = 0x2222;
    let original_amount = 100_000_000_000;
    let token_mint = 0x1111111111111111111111111111111111111111111111111111111111111111;
    let withdraw_amount = 30_000_000_000;
    let remaining_amount = 70_000_000_000;
    let recipient = 0xdeadbeef;
    let new_secret = 0x3333;
    let new_nullifier_secret = 0x4444;

    let original_commitment = compute_commitment_with_token(secret, nullifier_secret, original_amount, token_mint);
    let nullifier_hash = compute_nullifier(nullifier_secret);
    let new_commitment = compute_commitment_with_token(new_secret, new_nullifier_secret, remaining_amount, token_mint);

    let zero = get_zero_value(0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut indices: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_zero = zero;
    for i in 0..TREE_DEPTH {
        path[i] = current_zero;
        current_zero = hash_2([current_zero, current_zero]);
    }
    let root = compute_merkle_root(original_commitment, path, indices);

    main(
        secret, nullifier_secret, original_amount, token_mint, path, indices,
        new_secret, new_nullifier_secret,
        root, nullifier_hash, recipient, withdraw_amount, new_commitment, token_mint
    );
}

#[test]
fn test_full_withdrawal_with_token() {
    let secret = 0xaaaa;
    let nullifier_secret = 0xbbbb;
    let amount = 50_000_000_000;
    let token_mint = 0x1111111111111111111111111111111111111111111111111111111111111111;
    let recipient = 0xcafe;

    let original_commitment = compute_commitment_with_token(secret, nullifier_secret, amount, token_mint);
    let nullifier_hash = compute_nullifier(nullifier_secret);

    let zero = get_zero_value(0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut indices: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_zero = zero;
    for i in 0..TREE_DEPTH {
        path[i] = current_zero;
        current_zero = hash_2([current_zero, current_zero]);
    }
    let root = compute_merkle_root(original_commitment, path, indices);

    main(
        secret, nullifier_secret, amount, token_mint, path, indices,
        0, 0,
        root, nullifier_hash, recipient, amount, 0, token_mint
    );
}

#[test]
fn test_cross_token_swap() {
    let src_secret = 0xaaaa;
    let src_nullifier_secret = 0xbbbb;
    let src_amount = 10_000_000_000;
    let src_token_mint = 0x1111;
    let dst_secret = 0xcccc;
    let dst_nullifier_secret = 0xdddd;
    let dst_amount = 1000_000_000;
    let dst_token_mint = 0x2222;
    let min_dst_amount = 990_000_000;

    let src_commitment = compute_commitment_with_token(src_secret, src_nullifier_secret, src_amount, src_token_mint);
    let dst_commitment = compute_commitment_with_token(dst_secret, dst_nullifier_secret, dst_amount, dst_token_mint);
    let nullifier_hash = compute_nullifier(src_nullifier_secret);

    let zero = get_zero_value(0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut indices: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_zero = zero;
    for i in 0..TREE_DEPTH {
        path[i] = current_zero;
        current_zero = hash_2([current_zero, current_zero]);
    }
    let src_root = compute_merkle_root(src_commitment, path, indices);

    swap_circuit(
        src_secret, src_nullifier_secret, src_amount, src_token_mint, path, indices,
        dst_secret, dst_nullifier_secret, dst_amount,
        src_root, nullifier_hash, src_token_mint, dst_token_mint, dst_commitment, min_dst_amount
    );
}

#[test(should_fail_with = "Slippage exceeded")]
fn test_swap_slippage_fails() {
    let src_secret = 0xaaaa;
    let src_nullifier_secret = 0xbbbb;
    let src_amount = 10_000_000_000;
    let src_token_mint = 0x1111;
    let dst_secret = 0xcccc;
    let dst_nullifier_secret = 0xdddd;
    let dst_amount = 900_000_000;
    let dst_token_mint = 0x2222;
    let min_dst_amount = 1000_000_000;

    let src_commitment = compute_commitment_with_token(src_secret, src_nullifier_secret, src_amount, src_token_mint);
    let dst_commitment = compute_commitment_with_token(dst_secret, dst_nullifier_secret, dst_amount, dst_token_mint);
    let nullifier_hash = compute_nullifier(src_nullifier_secret);

    let zero = get_zero_value(0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut indices: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_zero = zero;
    for i in 0..TREE_DEPTH {
        path[i] = current_zero;
        current_zero = hash_2([current_zero, current_zero]);
    }
    let src_root = compute_merkle_root(src_commitment, path, indices);

    swap_circuit(
        src_secret, src_nullifier_secret, src_amount, src_token_mint, path, indices,
        dst_secret, dst_nullifier_secret, dst_amount,
        src_root, nullifier_hash, src_token_mint, dst_token_mint, dst_commitment, min_dst_amount
    );
}

#[test(should_fail_with = "Invalid new commitment for change")]
fn test_invalid_change_commitment_fails() {
    let secret = 0x1111;
    let nullifier_secret = 0x2222;
    let original_amount = 100_000_000_000;
    let token_mint = 0x1111111111111111111111111111111111111111111111111111111111111111;
    let withdraw_amount = 30_000_000_000;
    let recipient = 0xabc;
    let new_secret = 0x3333;
    let new_nullifier_secret = 0x4444;
    let wrong_commitment = compute_commitment_with_token(new_secret, new_nullifier_secret, 99_000_000_000, token_mint);

    let original_commitment = compute_commitment_with_token(secret, nullifier_secret, original_amount, token_mint);
    let nullifier_hash = compute_nullifier(nullifier_secret);

    let zero = get_zero_value(0);
    let mut path: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut indices: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current_zero = zero;
    for i in 0..TREE_DEPTH {
        path[i] = current_zero;
        current_zero = hash_2([current_zero, current_zero]);
    }
    let root = compute_merkle_root(original_commitment, path, indices);

    main(
        secret, nullifier_secret, original_amount, token_mint, path, indices,
        new_secret, new_nullifier_secret,
        root, nullifier_hash, recipient, withdraw_amount, wrong_commitment, token_mint
    );
}
