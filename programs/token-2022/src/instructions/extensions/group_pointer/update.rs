use {
    crate::{instructions::extensions::ExtensionDiscriminator, instructions::MAX_MULTISIG_SIGNERS},
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Update the group pointer address. Only supported for mints that
/// include the `GroupPointer` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The group pointer authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's group pointer authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountView,
    /// The group pointer authority.
    pub authority: &'a AccountView,
    /// The new account address that holds the group
    pub group_address: Option<&'b Address>,
    /// The Signer accounts if `authority` is a multisig
    pub signers: &'a [AccountView],
    /// Token Program
    pub token_program: &'b Address,
}

impl Update<'_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            authority,
            signers: account_signers,
            token_program,
            ..
        } = self;

        if account_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let num_accounts = 2 + account_signers.len();

        // Instruction accounts
        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint.address()));
            // - Index 1 is always present
            if account_signers.is_empty() {
                instruction_accounts
                    .get_unchecked_mut(1)
                    .write(InstructionAccount::readonly_signer(authority.address()));
            } else {
                instruction_accounts
                    .get_unchecked_mut(1)
                    .write(InstructionAccount::readonly(authority.address()));
            }
        }

        for (instruction_account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(account_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut data = [0u8; 34];

        // Encode discriminators (GroupPointer + Update)
        data[..2].copy_from_slice(&[
            ExtensionDiscriminator::GroupPointer as u8,
            Update::DISCRIMINATOR,
        ]);

        // write group_address address bytes at offset [2..34]
        if let Some(group_address) = self.group_address {
            data[2..34].copy_from_slice(group_address.to_bytes().as_ref());
        } else {
            data[2..34].copy_from_slice(&[0; 32]);
        }

        let instruction = InstructionView {
            program_id: token_program,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: data.as_ref(),
        };

        // Account info array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            account_views.get_unchecked_mut(0).write(mint);
            // - Index 1 is always present
            account_views.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_view, signer) in account_views[2..].iter_mut().zip(account_signers.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(account_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
