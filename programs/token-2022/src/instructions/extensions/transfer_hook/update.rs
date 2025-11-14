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

/// Update the Transfer Hook extension on a mint.
///
/// Expected accounts:
/// **Single authority**
/// 0. `[writable]` The mint account to update.
/// 1. `[signer]` The authority of the mint account.
///
/// **Multisignature authority**
/// 0. `[writable]` The mint account to update.
/// 1. `[readonly]` The multisig account that is the authority of the mint
///    account.
/// 2. `[signer]` M signer accounts (as required by the multisig).
pub struct UpdateTransferHook<'a, 'b, 'c> {
    /// Mint Account to update.
    pub mint_account: &'a AccountView,
    /// Authority Account.
    pub authority: &'a AccountView,
    /// Signer Accounts (for multisig support)
    pub signers: &'c [&'a AccountView],
    /// Program that authorizes the transfer
    pub program_id: Option<&'b Address>,
    /// Token Program
    pub token_program: &'b Address,
}

impl UpdateTransferHook<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
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
        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 and 1 are always present
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint_account.address()));

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

        // Set discriminators (TransferHook + Update)
        write_bytes(
            &mut data[..2],
            &[
                ExtensionDiscriminator::TransferHook as u8,
                UpdateTransferHook::DISCRIMINATOR,
            ],
        );

        // Set program_id at offset [2..34]
        if let Some(program_id) = self.program_id {
            write_bytes(&mut data[2..34], program_id.to_bytes().as_ref());
        } else {
            write_bytes(&mut data[2..34], &[0; 32]);
        }

        let instruction = InstructionView {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(accounts.as_ptr() as _, num_accounts) },
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
        // - Index 0 and 1 are always present
        unsafe {
            account_views.get_unchecked_mut(0).write(mint_account);
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
