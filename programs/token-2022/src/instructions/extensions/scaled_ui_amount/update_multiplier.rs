use {
    crate::{
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_BYTE,
    },
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Update the multiplier. Only supported for mints that include the
/// `ScaledUiAmount` extension.
///
/// Fails if the multiplier is less than or equal to 0 or if it's
/// [subnormal](https://en.wikipedia.org/wiki/Subnormal_number).
///
/// The authority provides a new multiplier and a UNIX timestamp on which
/// it should take effect. If the timestamp is before the current time,
/// immediately sets the multiplier.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The multiplier authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's multisignature multiplier authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct UpdateMultiplier<'a, 'b, 'c> {
    /// The mint.
    pub mint: &'a AccountView,

    /// The multiplier authority.
    pub authority: &'a AccountView,

    /// The signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],

    /// The new multiplier
    pub multiplier: f64,

    /// Timestamp at which the new multiplier will take effect.
    pub effective_timestamp: i64,

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> UpdateMultiplier<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `UpdateMultiplier` instruction with a single
    /// owner/delegate authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        multiplier: f64,
        effective_timestamp: i64,
    ) -> Self {
        Self {
            mint,
            authority,
            signers: &[],
            multiplier,
            effective_timestamp,
            token_program,
        }
    }

    /// Creates a new `UpdateMultiplier` instruction with a multisignature
    /// owner/delegate authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        multiplier: f64,
        effective_timestamp: i64,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            authority,
            signers,
            multiplier,
            effective_timestamp,
            token_program,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let expected_accounts = 2 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            // mint
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // authority
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::new(
                    self.authority.address(),
                    false,
                    self.signers.is_empty(),
                ));

            // signer accounts
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 18];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::ScaledUiAmount as u8,
                Self::DISCRIMINATOR,
            ],
        );
        // mutiplier
        write_bytes(&mut instruction_data[2..10], &self.multiplier.to_le_bytes());
        // effective_timestamp
        write_bytes(
            &mut instruction_data[10..18],
            &self.effective_timestamp.to_le_bytes(),
        );

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
            data: unsafe {
                slice::from_raw_parts(
                    instruction_data.as_ptr() as *const _,
                    instruction_data.len(),
                )
            },
        };

        // Accounts.

        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_ACCOUNT_VIEWS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            // mint
            accounts.get_unchecked_mut(0).write(self.mint);

            // authority
            accounts.get_unchecked_mut(1).write(self.authority);

            // signer accounts
            for (account, signer) in accounts[2..].iter_mut().zip(self.signers.iter()) {
                account.write(signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
