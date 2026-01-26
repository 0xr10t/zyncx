use anchor_lang::prelude::*;

pub const PROOF_SIZE: usize = 256; // Groth16 proof: 2*32 (A) + 2*64 (B) + 2*32 (C) = 256 bytes
pub const PUBLIC_INPUT_SIZE: usize = 32; // Each public input is a 32-byte field element

#[account]
pub struct VerificationKey {
    pub bump: u8,
    pub alpha_g1: [u8; 64],      // G1 point (x, y)
    pub beta_g2: [u8; 128],      // G2 point (x1, x2, y1, y2)
    pub gamma_g2: [u8; 128],     // G2 point
    pub delta_g2: [u8; 128],     // G2 point
    pub ic: Vec<[u8; 64]>,       // IC points (one per public input + 1)
}

impl VerificationKey {
    pub const BASE_SPACE: usize = 8 + // discriminator
        1 +   // bump
        64 +  // alpha_g1
        128 + // beta_g2
        128 + // gamma_g2
        128 + // delta_g2
        4;    // ic vec length prefix

    pub fn space_with_inputs(num_public_inputs: usize) -> usize {
        Self::BASE_SPACE + (num_public_inputs + 1) * 64
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Groth16Proof {
    pub a: [u8; 64],  // G1 point
    pub b: [u8; 128], // G2 point
    pub c: [u8; 64],  // G1 point
}

impl Groth16Proof {
    pub const SIZE: usize = 64 + 128 + 64;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(crate::errors::ZyncxError::InvalidZKProof.into());
        }

        let mut a = [0u8; 64];
        let mut b = [0u8; 128];
        let mut c = [0u8; 64];

        a.copy_from_slice(&bytes[0..64]);
        b.copy_from_slice(&bytes[64..192]);
        c.copy_from_slice(&bytes[192..256]);

        Ok(Self { a, b, c })
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalPublicInputs {
    pub withdrawn_value: [u8; 32],
    pub state_root: [u8; 32],
    pub new_commitment: [u8; 32],
    pub nullifier_hash: [u8; 32],
}

impl WithdrawalPublicInputs {
    pub fn new(
        amount: u64,
        root: [u8; 32],
        new_commitment: [u8; 32],
        nullifier: [u8; 32],
    ) -> Self {
        let mut withdrawn_value = [0u8; 32];
        withdrawn_value[24..32].copy_from_slice(&amount.to_be_bytes());

        Self {
            withdrawn_value,
            state_root: root,
            new_commitment,
            nullifier_hash: nullifier,
        }
    }

    pub fn to_field_elements(&self) -> [[u8; 32]; 4] {
        [
            self.withdrawn_value,
            self.state_root,
            self.new_commitment,
            self.nullifier_hash,
        ]
    }
}

pub fn verify_groth16(
    proof: &Groth16Proof,
    public_inputs: &WithdrawalPublicInputs,
    _vk: Option<&VerificationKey>,
) -> Result<bool> {
    // Groth16 verification on Solana
    //
    // For production use, integrate with groth16-solana crate:
    // https://github.com/Lightprotocol/groth16-solana
    //
    // The verification involves:
    // 1. Parse proof points (A ∈ G1, B ∈ G2, C ∈ G1)
    // 2. Parse public inputs as field elements
    // 3. Compute linear combination of IC points with public inputs
    // 4. Perform pairing check: e(A, B) = e(α, β) · e(L, γ) · e(C, δ)
    //
    // Solana provides alt_bn128 precompiles for pairing operations:
    // - sol_alt_bn128_g1_add
    // - sol_alt_bn128_g1_mul
    // - sol_alt_bn128_pairing
    //
    // Example with groth16-solana:
    // ```rust
    // use groth16_solana::groth16::Groth16Verifier;
    //
    // let mut verifier = Groth16Verifier::new(
    //     &proof.a,
    //     &proof.b,
    //     &proof.c,
    //     &public_inputs.to_field_elements(),
    //     &vk,
    // )?;
    //
    // let result = verifier.verify()?;
    // ```

    let inputs = public_inputs.to_field_elements();
    
    msg!("Verifying Groth16 proof...");
    msg!("Public inputs:");
    msg!("  - withdrawn_value: {:?}", &inputs[0][24..32]);
    msg!("  - state_root: {:?}", &inputs[1][0..8]);
    msg!("  - new_commitment: {:?}", &inputs[2][0..8]);
    msg!("  - nullifier_hash: {:?}", &inputs[3][0..8]);

    // Placeholder: Return true for valid proof structure
    // In production, replace with actual Groth16 verification
    if proof.a == [0u8; 64] && proof.b == [0u8; 128] && proof.c == [0u8; 64] {
        msg!("Invalid proof: all zeros");
        return Ok(false);
    }

    msg!("Proof structure valid (placeholder verification)");
    Ok(true)
}

pub mod alt_bn128 {
    #![allow(dead_code)]

    pub const G1_POINT_SIZE: usize = 64;
    pub const G2_POINT_SIZE: usize = 128;
    pub const SCALAR_SIZE: usize = 32;

    #[derive(Clone, Copy)]
    pub struct G1Point {
        pub x: [u8; 32],
        pub y: [u8; 32],
    }

    impl G1Point {
        pub fn from_bytes(bytes: &[u8; 64]) -> Self {
            let mut x = [0u8; 32];
            let mut y = [0u8; 32];
            x.copy_from_slice(&bytes[0..32]);
            y.copy_from_slice(&bytes[32..64]);
            Self { x, y }
        }

        pub fn to_bytes(&self) -> [u8; 64] {
            let mut result = [0u8; 64];
            result[0..32].copy_from_slice(&self.x);
            result[32..64].copy_from_slice(&self.y);
            result
        }
    }

    #[derive(Clone, Copy)]
    pub struct G2Point {
        pub x1: [u8; 32],
        pub x2: [u8; 32],
        pub y1: [u8; 32],
        pub y2: [u8; 32],
    }

    impl G2Point {
        pub fn from_bytes(bytes: &[u8; 128]) -> Self {
            let mut x1 = [0u8; 32];
            let mut x2 = [0u8; 32];
            let mut y1 = [0u8; 32];
            let mut y2 = [0u8; 32];
            x1.copy_from_slice(&bytes[0..32]);
            x2.copy_from_slice(&bytes[32..64]);
            y1.copy_from_slice(&bytes[64..96]);
            y2.copy_from_slice(&bytes[96..128]);
            Self { x1, x2, y1, y2 }
        }

        pub fn to_bytes(&self) -> [u8; 128] {
            let mut result = [0u8; 128];
            result[0..32].copy_from_slice(&self.x1);
            result[32..64].copy_from_slice(&self.x2);
            result[64..96].copy_from_slice(&self.y1);
            result[96..128].copy_from_slice(&self.y2);
            result
        }
    }
}
