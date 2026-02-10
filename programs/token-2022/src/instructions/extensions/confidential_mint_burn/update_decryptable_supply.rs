use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, AE_CIPHERTEXT_LEN, UNINIT_ACCOUNT_REF, UNINIT_BYTE,
        UNINIT_INSTRUCTION_ACCOUNT,
    },
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Updates the decrypt-able supply of the mint
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The SPL Token mint.
///   1. `[signer]` Confidential mint authority.
///
///   * Multisignature authority
///   0. `[writable]` The SPL Token mint.
///   1. `[]` The multisig authority account owner.
///   2. ..`[signer]` Required M signer accounts for the SPL Token Multisig
pub struct DecryptableSuppply<'a, 'b, 'data> {
    /// The Token mint
    pub mint: &'a AccountView,
    /// The Confidential mint authority,
    pub authority: &'a AccountView,
    /// The multisig signers
    pub multisig_signers: &'b [&'a AccountView],
    /// The token program
    pub token_program: &'a Address,

    /// Data expected:
    ///
    /// The new decrypt-able supply
    pub new_decryptable_supply: &'data [u8; AE_CIPHERTEXT_LEN],
}

impl DecryptableSuppply<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 2;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        // instruction accounts

        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        // token mint
        instruction_accounts[0].write(InstructionAccount::writable(self.mint.address()));

        // confidential token mint authority
        instruction_accounts[1].write(InstructionAccount::new(
            self.authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        // multisig authority signers
        for (account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // instruction data

        let mut instruction_data = [UNINIT_BYTE; 2 + AE_CIPHERTEXT_LEN];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ConfidentialMintBurn as u8,
                Self::DISCRIMINATOR,
            ],
        );

        // new decrypt-able supply
        write_bytes(
            &mut instruction_data[2..2 + AE_CIPHERTEXT_LEN],
            self.new_decryptable_supply.as_ref(),
        );

        // instruction

        let expected_accounts = 2 + self.multisig_signers.len();

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        // Accounts
        let mut accounts = [UNINIT_ACCOUNT_REF; 2 + MAX_MULTISIG_SIGNERS];

        // token mint
        accounts[0].write(self.mint);

        // confidential token mint authority
        accounts[1].write(self.authority);

        // multisig signers
        for (account, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(*signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
