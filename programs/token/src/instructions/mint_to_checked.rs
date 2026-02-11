use {
    crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Mints new tokens to an account.
///
/// ### Accounts:
///   * Single mint authority
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[SIGNER]` The mint's minting authority.
///
///   * Multisignature mint authority
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[]` The mint's multisignature minting authority.
///   3. ..3+M `[SIGNER]` M signer accounts
pub struct MintToChecked<'a, 'b> {
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Token Account.
    pub account: &'a AccountView,
    /// Mint Authority
    pub mint_authority: &'a AccountView,
    /// Multisignature signers.
    pub multisig_signers: &'b [&'a AccountView],
    /// Amount
    pub amount: u64,
    /// Decimals
    pub decimals: u8,
}

impl<'a, 'b> MintToChecked<'a, 'b> {
    /// Creates a new `MintToChecked` instruction with a single mint authority.
    #[inline(always)]
    pub fn new(
        mint: &'a AccountView,
        account: &'a AccountView,
        mint_authority: &'a AccountView,
        amount: u64,
        decimals: u8,
    ) -> Self {
        Self::with_multisig_signers(mint, account, mint_authority, amount, decimals, &[])
    }

    /// Creates a new `MintToChecked` instruction with a
    /// multisignature mint authority and signer accounts.
    #[inline(always)]
    pub fn with_multisig_signers(
        mint: &'a AccountView,
        account: &'a AccountView,
        mint_authority: &'a AccountView,
        amount: u64,
        decimals: u8,
        multisig_signers: &'b [&'a AccountView],
    ) -> Self {
        Self {
            mint,
            account,
            mint_authority,
            multisig_signers,
            amount,
            decimals,
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

        let expected_accounts = 3 + self.multisig_signers.len();

        // Instruction accounts.

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        instruction_accounts[0].write(InstructionAccount::writable(self.mint.address()));

        instruction_accounts[1].write(InstructionAccount::writable(self.account.address()));

        instruction_accounts[2].write(InstructionAccount::new(
            self.mint_authority.address(),
            false,
            self.multisig_signers.is_empty(),
        ));

        for (account, signer) in instruction_accounts[3..]
            .iter_mut()
            .zip(self.multisig_signers.iter())
        {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Accounts.

        let mut accounts =
            [const { MaybeUninit::<&AccountView>::uninit() }; 3 + MAX_MULTISIG_SIGNERS];

        accounts[0].write(self.mint);

        accounts[1].write(self.account);

        accounts[2].write(self.mint_authority);

        for (account, signer) in accounts[3..].iter_mut().zip(self.multisig_signers.iter()) {
            account.write(signer);
        }

        // Instruction data.
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1..9]: amount (8 bytes, u64)
        // - [9]: decimals (1 byte, u8)

        let mut instruction_data = [UNINIT_BYTE; 10];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[14]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        write_bytes(&mut instruction_data[9..], &[self.decimals]);

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &InstructionView {
                program_id: &crate::ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 10) },
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
