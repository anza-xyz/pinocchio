use {
    crate::{
        instructions::{invalid_argument_error, writable_cpi_account, CpiWriter},
        write_bytes, UNINIT_BYTE, UNINIT_CPI_ACCOUNT, UNINIT_INSTRUCTION_ACCOUNT,
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

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 1;

/// Instruction data length:
///   - discriminator (1 byte)
///   - decimals (1 byte)
///   - mint authority (32 bytes)
///   - freeze authority (33 bytes, optional)
const MAX_DATA_LEN: usize = 67;

/// Like [`super::InitializeMint`], but does not require the Rent
/// sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub struct InitializeMint2<'account> {
    /// The mint to initialize.
    pub mint: &'account AccountView,

    /// The number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// The authority/multisignature to mint tokens.
    pub mint_authority: &'account Address,

    /// The freeze authority/multisignature of the mint.
    pub freeze_authority: Option<&'account Address>,
}

impl InitializeMint2<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 20;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [UNINIT_CPI_ACCOUNT; ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; MAX_DATA_LEN];
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
                from_raw_parts(accounts.as_ptr() as _, written_accounts),
            );
        }

        Ok(())
    }
}

impl CpiWriter for InitializeMint2<'_> {
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

        accounts[0].write(writable_cpi_account(self.mint)?);

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

        accounts[0].write(InstructionAccount::writable(self.mint.address()));

        Ok(ACCOUNTS_LEN)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        if data.len() < MAX_DATA_LEN {
            return Err(invalid_argument_error());
        }

        data[0].write(Self::DISCRIMINATOR);

        data[1].write(self.decimals);

        write_bytes(&mut data[2..34], self.mint_authority.as_array());

        if let Some(freeze_auth) = self.freeze_authority {
            data[34].write(1);

            write_bytes(&mut data[35..MAX_DATA_LEN], freeze_auth.as_array());

            Ok(MAX_DATA_LEN)
        } else {
            data[34].write(0);

            Ok(35)
        }
    }
}
