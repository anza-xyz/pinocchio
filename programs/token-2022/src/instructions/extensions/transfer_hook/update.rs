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

pub struct UpdateTransferHook<'a, 'b> {
    /// Mint Account to update.
    pub mint_account: &'a AccountView,
    /// Authority Account.
    pub authority: &'a AccountView,
    /// Signer Accounts (for multisig support)
    pub signers: &'a [AccountView],
    /// Program that authorizes the transfer
    pub program_id: Option<&'b Address>,
    /// Token Program
    pub token_program: &'b Address,
}

impl UpdateTransferHook<'_, '_> {
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
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint_account.address()));
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

        for (account_meta, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(account_signers.iter())
        {
            account_meta.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut data = [0u8; 34];

        // Set discriminators (TransferHook + Update)
        data[..2].copy_from_slice(&[
            ExtensionDiscriminator::TransferHook as u8,
            UpdateTransferHook::DISCRIMINATOR,
        ]);

        // Set program_id at offset [2..34]
        if let Some(x) = self.program_id {
            data[2..34].copy_from_slice(x.to_bytes().as_ref());
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
        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            account_views.get_unchecked_mut(0).write(mint_account);
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
