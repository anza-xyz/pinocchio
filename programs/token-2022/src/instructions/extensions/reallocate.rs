use {
    crate::{
        instructions::{ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
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

/// Reallocate space in a token account.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The token account.
///   1. `[signer]` The token account owner.
///   2. `[]` The system program.
///
///   * Multisignature authority
///   0. `[writable]` The token account.
///   1. `[readonly]` The multisig account that owns the token account.
///   2. `[]` The system program.
///   3. `..3+M` `[signer]` M signer accounts.
pub struct Reallocate<'a, 'b, 'c> {
    /// The token account to reallocate.
    pub token_account: &'a AccountView,

    /// The owner of the token account (single or multisig).
    pub authority: &'a AccountView,

    /// The system program.
    pub system_program: &'a AccountView,

    /// Signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],

    /// The extension types to reallocate.
    pub extension_types: &'c [ExtensionDiscriminator],

    /// The token program.
    pub token_program: &'b Address,
}

impl Reallocate<'_, '_, '_> {
    /// `N` is the size of the instruction data buffer:
    /// `1 + num_extension_types * 2`.
    #[inline(always)]
    pub fn invoke<const N: usize>(&self) -> ProgramResult {
        self.invoke_signed::<N>(&[])
    }

    /// `N` is the size of the instruction data buffer:
    /// `1 + num_extension_types * 2`.
    #[inline(always)]
    pub fn invoke_signed<const N: usize>(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let expected_accounts = 2 + self.signers.len();

        // Instruction accounts.

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 2 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // token_account
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(self.token_account.address()));

            // authority
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly_signer(
                    self.authority.address(),
                ));

            // system program
            instruction_accounts
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(self.system_program.address()));

            // signer accounts
            for (account, signer) in instruction_accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
                account.write(InstructionAccount::readonly_signer(signer.address()));
            }
        }

        // Instruction data layout:
        // - [0]: extension instruction discriminator (1 byte, u8)
        // - [1..]: extension types (2 bytes each, u16 LE)
        let mut instruction_data = [UNINIT_BYTE; N];

        write_bytes(
            &mut instruction_data,
            &[ExtensionDiscriminator::Reallocate as u8],
        );

        for (i, ext) in self.extension_types.iter().enumerate() {
            let ext_byte = unsafe { *(ext as *const ExtensionDiscriminator as *const u8) };
            write_bytes(
                &mut instruction_data[1 + i * 2..],
                &(ext_byte as u16).to_le_bytes(),
            );
        }

        let instruction = InstructionView {
            program_id: self.token_program,
            data: unsafe { slice::from_raw_parts(instruction_data.as_ptr() as _, N) },
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
            },
        };

        // Accounts.

        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_ACCOUNT_VIEWS; 3 + MAX_MULTISIG_SIGNERS];

        // SAFETY: The allocation is valid to the maximum number of accounts.
        unsafe {
            // token_account
            accounts.get_unchecked_mut(0).write(self.token_account);

            // authority
            accounts.get_unchecked_mut(1).write(self.authority);

            // system program
            accounts.get_unchecked_mut(2).write(self.system_program);

            // signer accounts
            for (account, signer) in accounts
                .get_unchecked_mut(3..)
                .iter_mut()
                .zip(self.signers.iter())
            {
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
