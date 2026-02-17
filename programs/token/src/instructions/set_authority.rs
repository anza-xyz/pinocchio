use {
    crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AuthorityType {
    MintTokens = 0,
    FreezeAccount = 1,
    AccountOwner = 2,
    CloseAccount = 3,
}

/// Sets a new authority of a mint or account.
///
/// ### Accounts:
///   * Single authority
///   0. `[WRITE]` The mint or account to change the authority of.
///   1. `[SIGNER]` The current authority of the mint or account.
///
///   * Multisignature authority
///   0. `[WRITE]` The mint or account to change the authority of.
///   1. `[]` The current multisignature authority of the mint or account.
///   2. `..2+M` `[SIGNER]` M signer accounts
pub struct SetAuthority<'a, 'b, 'c> {
    /// Account (Mint or Token)
    pub account: &'a AccountView,
    /// Authority of the Account.
    pub authority: &'a AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'c [&'a AccountView],
    /// The type of authority to update.
    pub authority_type: AuthorityType,
    /// The new authority
    pub new_authority: Option<&'b Address>,
}

impl<'a, 'b, 'c> SetAuthority<'a, 'b, 'c> {
    /// Creates a new `SetAuthority` instruction with a single authority.
    #[inline(always)]
    pub fn new(
        account: &'a AccountView,
        authority: &'a AccountView,
        authority_type: AuthorityType,
        new_authority: Option<&'b Address>,
    ) -> Self {
        Self::with_multisig_signers(account, authority, authority_type, new_authority, &[])
    }

    /// Creates a new `SetAuthority` instruction with a
    /// multisignature authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        account: &'a AccountView,
        authority: &'a AccountView,
        authority_type: AuthorityType,
        new_authority: Option<&'b Address>,
        multisig_signers: &'c [&'a AccountView],
    ) -> Self {
        Self {
            account,
            authority,
            multisig_signers,
            authority_type,
            new_authority,
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

        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));

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

        accounts[0].write(self.account);

        accounts[1].write(self.authority);

        for (account, signer) in accounts[2..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        // Instruction data.
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1]: authority_type (1 byte, u8)
        // - [2]: new_authority presence flag (1 byte, AuthorityType)
        // - [3..35] new_authority (optional, 32 bytes, Address)

        let mut instruction_data = [UNINIT_BYTE; 35];
        let mut length = instruction_data.len();

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[6]);
        // Set authority_type as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[self.authority_type as u8]);

        if let Some(new_authority) = self.new_authority {
            // Set new_authority as [u8; 32] at offset [2..35]
            write_bytes(&mut instruction_data[2..3], &[1]);
            write_bytes(&mut instruction_data[3..], new_authority.as_array());
        } else {
            write_bytes(&mut instruction_data[2..3], &[0]);
            // Adjust length if no new authority
            length = 3;
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: &crate::ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, length) },
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
