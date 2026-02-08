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
///   1. `[signer]`   The group member pointer authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]`         The group member pointer authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b, 'c> {
    /// The mint.
    pub mint: &'a AccountView,

    /// The group member pointer authority.
    pub authority: &'a AccountView,

    /// The new account address that holds the group.
    pub member_address: Option<&'b Address>,

    /// The signer accounts if `authority` is a multisig
    pub signers: &'c [&'a AccountView],

    /// Token Program
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
        member_address: Option<&'b Address>,
    ) -> Self {
        Self {
            mint,
            authority,
            signers: &[],
            member_address,
            token_program,
        }
    }

    /// Creates a new `Update` instruction with a multisignature owner/delegate
    /// authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig(
        token_program: &'b Address,
        mint: &'a AccountView,
        authority: &'a AccountView,
        member_address: Option<&'b Address>,
        signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            authority,
            signers,
            member_address,
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
            Err(ProgramError::InvalidArgument)?;
        }

        let expected_accounts = 2 + self.signers.len();

        // Instruction accounts.

        const UNINIT_META: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The expected number of accounts has been validated to be less than
        // the maximum allocated.
        unsafe {
            // mint
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.mint.address()));

            // authority
            accounts.get_unchecked_mut(1).write(InstructionAccount::new(
                self.authority.address(),
                false,
                self.signers.is_empty(),
            ));

            // signer accounts
            for (account, signer) in accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 34];

        // discriminators
        write_bytes(
            &mut instruction_data[..2],
            &[
                ExtensionDiscriminator::GroupMemberPointer as u8,
                Self::DISCRIMINATOR,
            ],
        );
        // member_address
        write_bytes(
            &mut instruction_data[2..34],
            if let Some(member_address) = self.member_address {
                member_address.as_ref()
            } else {
                &[0u8; 32]
            },
        );

        // Instruction.

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
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
            for (account, signer) in accounts
                .get_unchecked_mut(2..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(*signer);
            }
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
