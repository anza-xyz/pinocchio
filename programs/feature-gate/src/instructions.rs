//! Feature Gate program instructions.

use {
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Discriminator for the `RevokePendingActivation` instruction.
pub const REVOKE_PENDING_ACTIVATION_DISCRIMINATOR: u8 = 0;

/// Revoke a pending feature activation.
///
/// This instruction will burn any lamports in the feature account by
/// transferring them to the incinerator. A "pending" feature activation
/// is a feature account that has been allocated and assigned, but
/// hasn't yet been updated by the runtime with an `activation_slot`.
///
/// Features that _have_ been activated by the runtime cannot be
/// revoked — the instruction will fail with
/// `FeatureGateError::FeatureAlreadyActivated`.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Feature account.
///   1. `[WRITE]` Incinerator account
///      (`1nc1nerator11111111111111111111111111111111`).
///   2. `[]` System program.
pub struct RevokePendingActivation<'a> {
    /// Feature account. Must be writable and signed by the feature
    /// keypair.
    pub feature: &'a AccountView,
    /// Incinerator account. Must be writable.
    ///
    /// Lamports transferred here are burned at the end of the current
    /// block. See [`crate::INCINERATOR_ID`] for the canonical address.
    pub incinerator: &'a AccountView,
    /// System program.
    pub system_program: &'a AccountView,
}

impl RevokePendingActivation<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts:
        //   0. `[w+s]` Feature account
        //   1. `[w]`   Incinerator
        //   2. `[]`    System program
        let instruction_accounts: [InstructionAccount; 3] = [
            InstructionAccount::writable_signer(self.feature.address()),
            InstructionAccount::writable(self.incinerator.address()),
            InstructionAccount::readonly(self.system_program.address()),
        ];

        // Instruction data is a single discriminator byte.
        let instruction_data = [REVOKE_PENDING_ACTIVATION_DISCRIMINATOR];

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[self.feature, self.incinerator, self.system_program],
            signers,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_matches_spl_reference() {
        // The SPL Feature Gate program uses discriminator `0` for
        // `RevokePendingActivation` — the sole instruction of the
        // program per SIMD-0089.
        assert_eq!(REVOKE_PENDING_ACTIVATION_DISCRIMINATOR, 0);
    }
}
