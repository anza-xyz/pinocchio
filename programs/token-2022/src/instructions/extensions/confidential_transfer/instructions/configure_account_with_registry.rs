use {
    crate::{instructions::ExtensionDiscriminator, UNINIT_ACCOUNT_REF, UNINIT_INSTRUCTION_ACCOUNT},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Configures confidential transfers for a token account.
///
/// This instruction is identical to the `ConfigureAccount` account except
/// that a valid `ElGamalRegistry` account is expected in place of the
/// `VerifyPubkeyValidity` proof.
///
/// An `ElGamalRegistry` account is valid if it shares the same owner with
/// the token account. If a valid `ElGamalRegistry` account is provided,
// spell-checker:ignore ElGamal
/// then the program skips the verification of the ElGamal pubkey
/// validity proof as well as the token owner signature.
///
/// If the token account is not large enough to include the new
/// confidential transfer extension, then optionally reallocate the
/// account to increase the data size. To reallocate, a payer account to
/// fund the reallocation and the system account should be included in the
/// instruction.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The SPL Token account.
///   1. `[]` The corresponding SPL Token mint.
///   2. `[]` The ElGamal registry account.
///   3. `[signer, writable]` (Optional) The payer account to fund reallocation
///   4. `[]` (Optional) System program for reallocation funding
pub struct ConfigureAccountWithRegistry<'a> {
    pub token_account: &'a AccountView,
    pub mint: &'a AccountView,
    pub elgamal_registry: &'a AccountView,
    pub reallocation_payer: Option<&'a AccountView>,
    /// System program required for reallocation
    pub system_program: Option<&'a AccountView>,
    pub token_program: &'a Address,
}

impl ConfigureAccountWithRegistry<'_> {
    const DISCRIMINATOR: u8 = 13;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signer: &[Signer]) -> ProgramResult {
        // instruction account

        let mut expected_accounts: usize = 3;

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 5];

        // SAFETY: allocation is valid to the max accounts
        unsafe {
            // token account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // token mint
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(self.mint.address()));

            // elgamal registry account
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(
                    self.elgamal_registry.address(),
                ));

            // account reallocation payer
            if let Some(payer_account) = self.reallocation_payer {
                instruction_accounts
                    .get_unchecked_mut(expected_accounts)
                    .write(InstructionAccount::writable_signer(payer_account.address()));
                expected_accounts += 1;
            }

            // system program for reallocation
            if let Some(system_program_account) = self.system_program {
                instruction_accounts
                    .get_unchecked_mut(expected_accounts)
                    .write(InstructionAccount::readonly(
                        system_program_account.address(),
                    ));
                expected_accounts += 1;
            }
        }

        // instruction data: extension discriminators + exension instruction
        // discriminator
        let instruction_data = [
            ExtensionDiscriminator::ConfidentialTransfer as u8,
            Self::DISCRIMINATOR,
        ];

        // instruction
        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: instruction_data.as_slice(),
        };

        // Accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 5];

        // SAFETY: allocation is valid to the max accounts
        unsafe {
            // token account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // token mint
            accounts.get_unchecked_mut(1).write(self.mint);

            // elgamal registry account
            accounts.get_unchecked_mut(2).write(self.elgamal_registry);

            let mut i = 3usize;

            // account reallocation payer
            if let Some(payer_account) = self.reallocation_payer {
                accounts.get_unchecked_mut(i).write(payer_account);
                i += 1;
            }

            // system program for reallocation
            if let Some(system_program_account) = self.system_program {
                accounts.get_unchecked_mut(i).write(system_program_account);
            }
        }

        invoke_signed_with_bounds::<5>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signer,
        )
    }
}
