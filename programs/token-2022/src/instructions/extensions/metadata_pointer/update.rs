use {
    crate::{
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_BYTE,
    },
    core::{
        mem::MaybeUninit,
        slice::{self, from_raw_parts},
    },
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Update the metadata pointer address. Only supported for mints that
/// include the `MetadataPointer` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The metadata pointer authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's metadata pointer authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b, 'c> {
    /// The mint to update.
    pub mint: &'a AccountView,
    /// The metadata pointer authority.
    pub authority: &'a AccountView,
    /// New metadata address (use `None` to clear).
    pub new_metadata_address: Option<&'b Address>,
    /// The Signer accounts if `authority` is a multisig.
    pub signers: &'c [&'a AccountView],
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl Update<'_, '_, '_> {
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
            signers: multisig_accounts,
            token_program,
            ..
        } = self;

        if multisig_accounts.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 and 1 are always present (Mint)
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint.address()));

            if multisig_accounts.is_empty() {
                accounts
                    .get_unchecked_mut(1)
                    .write(InstructionAccount::readonly_signer(authority.address()));
            } else {
                accounts
                    .get_unchecked_mut(1)
                    .write(InstructionAccount::readonly(authority.address()));
            }
        }

        for (account, signer) in accounts[2..].iter_mut().zip(multisig_accounts.iter()) {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut data = [UNINIT_BYTE; 34];

        // Encode discriminators (Metadata + Update)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::MetadataPointer as u8,
                Update::DISCRIMINATOR,
            ],
        );

        // write new_metadata_address Address bytes at offset  [2..34]
        if let Some(new_metadata_address) = self.new_metadata_address {
            write_bytes(&mut data[2..34], new_metadata_address.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[2..34], &[0; 32]);
        }

        let num_accounts = 2 + multisig_accounts.len();

        let instruction = InstructionView {
            program_id: token_program,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
            accounts: unsafe {
                slice::from_raw_parts(accounts.as_ptr() as *const InstructionAccount, num_accounts)
            },
        };

        // Account view array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> =
            MaybeUninit::<&AccountView>::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 and 1 are always present
        unsafe {
            account_views.get_unchecked_mut(0).write(mint);
            account_views.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_view, signer) in account_views[2..].iter_mut().zip(multisig_accounts.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe {
                slice::from_raw_parts(account_views.as_ptr() as *const &AccountView, num_accounts)
            },
            signers,
        )
    }
}
