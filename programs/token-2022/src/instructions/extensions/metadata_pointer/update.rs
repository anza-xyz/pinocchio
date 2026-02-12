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
    /// The mint.
    pub mint: &'a AccountView,

    /// The metadata pointer authority.
    pub authority: &'a AccountView,

    /// The signer accounts when `authority` is a multisig.
    pub multisig_signers: &'c [&'a AccountView],

    /// The new account address that holds the metadata.
    pub metadata_address: Option<&'b Address>,

    /// The token program.
    pub token_program: &'b Address,
}

impl<'a, 'b, 'c> Update<'a, 'b, 'c> {
    pub const DISCRIMINATOR: u8 = 1;

    /// Creates a new `Update` instruction with a single owner/delegate
    /// authority.
    #[inline(always)]
    pub fn new(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        metadata_address: Option<&'b Address>,
    ) -> Self {
        Self::with_multisig_signers(token_program, mint, authority, metadata_address, &[])
    }

    /// Creates a new `Update` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        metadata_address: Option<&'b Address>,
        multisig_signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            authority,
            multisig_signers,
            metadata_address,
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

        let expected_accounts = 2 + self.multisig_signers.len();

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 2 + MAX_MULTISIG_SIGNERS];

        instruction_accounts[0].write(InstructionAccount::writable(self.mint.address()));

        instruction_accounts[1].write(InstructionAccount::new(
            self.authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<&AccountView>::uninit() }; 2 + MAX_MULTISIG_SIGNERS];

        accounts[0].write(self.mint);

        accounts[1].write(self.authority);

        for (account_view, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account_view.write(signer);
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 34];

        instruction_data[0].write(ExtensionDiscriminator::MetadataPointer as u8);

        instruction_data[1].write(Self::DISCRIMINATOR);

        write_bytes(
            &mut instruction_data[2..34],
            if let Some(address) = self.metadata_address {
                address.as_ref()
            } else {
                &[0; 32]
            },
        );

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: self.token_program,
                // SAFETY: instruction accounts has `expected_accounts` initialized.
                accounts: unsafe {
                    slice::from_raw_parts(
                        instruction_accounts.as_ptr() as *const InstructionAccount,
                        expected_accounts,
                    )
                },
                // SAFETY: instruction data is initialized.
                data: unsafe {
                    from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len())
                },
            },
            // SAFETY: accounts has `expected_accounts` initialized.
            unsafe {
                slice::from_raw_parts(accounts.as_ptr() as *const &AccountView, expected_accounts)
            },
            signers,
        )
    }
}
