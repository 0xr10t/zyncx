// ============================================================================
// ZYNCX ARCIS MPC CIRCUITS
// ============================================================================
// Simplified circuits matching the voting example pattern - single return values
// ============================================================================

use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    /// Vault state stored encrypted on-chain
    #[derive(Copy, Clone)]
    pub struct VaultState {
        pub pending_deposits: u64,
        pub total_liquidity: u64,
        pub total_deposited: u64,
    }

    /// Initialize a new vault with zeroed encrypted state
    #[instruction]
    pub fn init_vault(mxe: Mxe) -> Enc<Mxe, VaultState> {
        let initial_state = VaultState {
            pending_deposits: 0,
            total_liquidity: 0,
            total_deposited: 0,
        };
        mxe.from_arcis(initial_state)
    }

    /// Process a deposit - just updates vault state
    #[instruction]
    pub fn process_deposit(
        deposit_amount: u64,
        vault_state: Enc<Mxe, VaultState>,
    ) -> Enc<Mxe, VaultState> {
        let mut vault = vault_state.to_arcis();
        vault.pending_deposits = vault.pending_deposits + deposit_amount;
        vault.total_deposited = vault.total_deposited + deposit_amount;
        vault_state.owner.from_arcis(vault)
    }

    /// Evaluate swap - returns boolean for whether swap should execute
    #[instruction]
    pub fn confidential_swap(
        encrypted_min_out: Enc<Shared, u64>,
        current_output: u64,
    ) -> bool {
        let min_out = encrypted_min_out.to_arcis();
        (current_output >= min_out).reveal()
    }
}
