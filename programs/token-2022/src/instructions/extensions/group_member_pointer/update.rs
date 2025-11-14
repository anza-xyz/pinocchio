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

/// Update the group member pointer address. Only supported for mints that
/// include the `GroupMemberPointer` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The group member pointer authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's group member pointer authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b, 'c> {
    /// Mint Account
    pub mint: &'a AccountView,
    /// The group member pointer authority.
    pub authority: &'a AccountView,
    /// The new account address that holds the member
    pub member_address: Option<&'b Address>,
    /// The Signer accounts if `authority` is a multisig
    pub signers: &'c [&'a AccountView],
    /// Token Program
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
            signers: account_signers,
            token_program,
            ..
        } = self;

        if account_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let num_accounts = 2 + account_signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 and 1 are always present
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint.address()));

            if account_signers.is_empty() {
                accounts
                    .get_unchecked_mut(1)
                    .write(InstructionAccount::readonly_signer(authority.address()));
            } else {
                accounts
                    .get_unchecked_mut(1)
                    .write(InstructionAccount::readonly(authority.address()));
            }
        }

        for (account, signer) in accounts[2..].iter_mut().zip(account_signers.iter()) {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut data = [UNINIT_BYTE; 34];

        // Encode discriminators (GroupMemberPointer + Update)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::GroupMemberPointer as u8,
                Update::DISCRIMINATOR,
            ],
        );

        // write member_address bytes at offset  [2..34]
        if let Some(member_address) = self.member_address {
            write_bytes(&mut data[2..34], member_address.to_bytes().as_ref());
        }

        let instruction = InstructionView {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(accounts.as_ptr() as _, num_accounts) },
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        // Account view array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 and 1 are always present
        unsafe {
            account_views.get_unchecked_mut(0).write(mint);
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
