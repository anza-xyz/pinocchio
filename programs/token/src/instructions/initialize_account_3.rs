use {
    crate::{
        instructions::{invalid_argument_error, CpiWriter},
        write_bytes, UNINIT_BYTE,
    },
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

const ACCOUNTS_LEN: usize = 2;

const DATA_LEN: usize = 33;

/// Like [`super::InitializeAccount2`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitializeAccount3<'a> {
    /// The account to initialize.
    pub account: &'a AccountView,

    /// The mint this account will be associated with.
    pub mint: &'a AccountView,

    /// The new account's owner/multisignature.
    pub owner: &'a Address,
}

impl InitializeAccount3<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 18;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data)?;

        unsafe {
            invoke_unchecked(
                &InstructionView {
                    program_id: &crate::ID,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: from_raw_parts(instruction_data.as_ptr() as _, written_instruction_data),
                },
                from_raw_parts(accounts.as_ptr() as *const CpiAccount, written_accounts),
            );
        }

        Ok(())
    }
}

impl CpiWriter for InitializeAccount3<'_> {
    #[inline(always)]
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        if accounts.len() < ACCOUNTS_LEN {
            return Err(invalid_argument_error());
        }

        accounts[0].write(CpiAccount::from(self.account));
        accounts[1].write(CpiAccount::from(self.mint));

        Ok(ACCOUNTS_LEN)
    }

    #[inline(always)]
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        'source: 'cpi,
    {
        if accounts.len() < ACCOUNTS_LEN {
            return Err(invalid_argument_error());
        }

        accounts[0].write(InstructionAccount::writable(self.account.address()));
        accounts[1].write(InstructionAccount::readonly(self.mint.address()));

        Ok(ACCOUNTS_LEN)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        if data.len() < DATA_LEN {
            return Err(invalid_argument_error());
        }

        data[0].write(Self::DISCRIMINATOR);
        write_bytes(&mut data[1..DATA_LEN], self.owner.as_array());

        Ok(DATA_LEN)
    }
}
