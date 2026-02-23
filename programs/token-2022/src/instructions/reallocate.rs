use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_EXTENSION_COUNT, MAX_MULTISIG_SIGNERS},
        UNINIT_BYTE,
    },
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Check to see if a token account is large enough for a list of
/// `ExtensionTypes`, and if not, use reallocation to increase the data
/// size.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The account to reallocate.
///   1. `[signer, writable]` The payer account to fund reallocation
///   2. `[]` System program for reallocation funding
///   3. `[signer]` The account's owner.
///
///   * Multisignature owner
///   0. `[writable]` The account to reallocate.
///   1. `[signer, writable]` The payer account to fund reallocation
///   2. `[]` System program for reallocation funding
///   3. `[]` The account's multisignature owner/delegate.
///   4. ..`4+M` `[signer]` M signer accounts.
pub struct Reallocate<'a, 'b, 'c, 'd> {
    /// The account to reallocate.
    pub account: &'a AccountView,

    /// The payer account to fund reallocation.
    pub payer: &'a AccountView,

    /// System program for reallocation funding.
    pub system_program: &'a AccountView,

    /// The account's multisignature owner/delegate.
    pub owner: &'a AccountView,

    /// The signer accounts for multisignature owner, if applicable.
    pub multisig_signers: &'c [&'a AccountView],

    /// New extension types to include in the reallocated account
    pub extensions: &'d [ExtensionDiscriminator],

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c, 'd> Reallocate<'a, 'b, 'c, 'd> {
    pub const DISCRIMINATOR: u8 = 29;

    /// Creates a new `Reallocate` instruction with a single owner/delegate
    /// authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        account: &'a AccountView,
        payer: &'a AccountView,
        system_program: &'a AccountView,
        owner: &'a AccountView,
        extensions: &'d [ExtensionDiscriminator],
    ) -> Self {
        Self::with_multisig_signers(
            token_program,
            account,
            payer,
            system_program,
            owner,
            extensions,
            &[],
        )
    }

    /// Creates a new `Reallocate` instruction with a multisignature
    /// owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        token_program: &'b Address,
        account: &'a AccountView,
        payer: &'a AccountView,
        system_program: &'a AccountView,
        owner: &'a AccountView,
        extensions: &'d [ExtensionDiscriminator],
        multisig_signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            payer,
            system_program,
            owner,
            multisig_signers,
            extensions,
            token_program,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let expected_accounts = 4 + self.multisig_signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 4 + MAX_MULTISIG_SIGNERS];

        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));

        instruction_accounts[1].write(InstructionAccount::writable_signer(self.payer.address()));

        instruction_accounts[2].write(InstructionAccount::readonly(self.system_program.address()));

        instruction_accounts[3].write(InstructionAccount::new(
            self.owner.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (account, signer) in instruction_accounts[4..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Accounts.

        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_INFO; 4 + MAX_MULTISIG_SIGNERS];

        accounts[0].write(self.account);

        accounts[1].write(self.payer);

        accounts[2].write(self.system_program);

        accounts[3].write(self.owner);

        for (account, signer) in accounts[4..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        // Instruction data.

        if self.extensions.len() > MAX_EXTENSION_COUNT {
            return Err(ProgramError::InvalidArgument);
        }

        let expected_data = 1 + self.extensions.len() * 2;

        // 1 byte (discriminator) + 2 bytes per extension (extension type as `u16`).
        let mut instruction_data = [UNINIT_BYTE; 1 + MAX_EXTENSION_COUNT * 2];

        instruction_data[0].write(Self::DISCRIMINATOR);

        for (i, extension) in self.extensions.iter().enumerate() {
            let offset = 1 + i * 2;
            // SAFETY: `offset` and `offset + 1` are within bounds of `instruction_data`
            // since `extensions.len() <= MAX_EXTENSION_COUNT`.
            //
            // Write the extension type as a little-endian `u16`.
            unsafe {
                instruction_data
                    .get_unchecked_mut(offset)
                    .write(*extension as u8);
                instruction_data.get_unchecked_mut(offset + 1).write(0);
            }
        }

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                // SAFETY: instruction data is initialized.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, expected_data) },
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
