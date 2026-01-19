use {
    crate::instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub struct UpdateMultiplier<'a, 'b, 'c> {
    /// Mint account to initialize
    pub mint_account: &'a AccountView,
    /// Multiplier authority
    pub authority: &'a AccountView,
    /// Signer accounts if the authority is a multisig
    pub signers: &'c [&'a AccountView],
    /// The new multiplier
    pub multiplier: f64,
    /// Timestamp at which the new multiplier will take effect
    pub effective_timestamp: i64,
    /// Token Program
    pub token_program: &'b Address,
}

impl UpdateMultiplier<'_, '_, '_> {
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
            signers: multisig_accounts,
            token_program,
            ..
        } = self;

        if multisig_accounts.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 2 + MAX_MULTISIG_SIGNERS

            // - Index 0 is always present (mint account)
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint_account.address()));

            // - Index 1 is always present (multiplier authority)
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    authority.address(),
                    false,
                    multisig_accounts.is_empty(),
                ));
        }

        for (instruction_account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(multisig_accounts.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // build instruction
        let data = &mut [0; 18];
        data[0] = ExtensionDiscriminator::ScaledUiAmount as u8;
        data[1] = Self::DISCRIMINATOR;
        data[2..10].copy_from_slice(&self.multiplier.to_le_bytes());
        data[10..18].copy_from_slice(&self.effective_timestamp.to_le_bytes());

        let num_accounts = 2 + multisig_accounts.len();

        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
        };

        // Account view array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_views` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            account_views.get_unchecked_mut(0).write(mint_account);
            // - Index 1 is always present
            account_views.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_view, signer) in account_views[2..].iter_mut().zip(multisig_accounts.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(account_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
