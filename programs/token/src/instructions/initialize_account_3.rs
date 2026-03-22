use {
    crate::{instructions::Batchable, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, ptr, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

const INITIALIZE_ACCOUNT_3_INSTRUCTION_DATA_LEN: usize = 33;

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitializeAccount3<'a> {
    /// New Account.
    pub account: &'a AccountView,
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Owner of the new Account.
    pub owner: &'a Address,
}

impl InitializeAccount3<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 18;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts);

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 2];
        let written_accounts = self.write_accounts(&mut accounts);

        let mut instruction_data = [UNINIT_BYTE; INITIALIZE_ACCOUNT_3_INSTRUCTION_DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data);

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

impl super::sealed::Sealed for InitializeAccount3<'_> {}

impl Batchable for InitializeAccount3<'_> {
    #[inline(always)]
    fn write_accounts(&self, accounts: &mut [MaybeUninit<CpiAccount>]) -> usize {
        accounts[0].write(CpiAccount::from(self.account));
        accounts[1].write(CpiAccount::from(self.mint));
        2
    }

    #[inline(always)]
    fn write_instruction_accounts(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount>],
    ) -> usize {
        // SAFETY: The written address references are borrowed from `self`, and
        // callers must not use the output buffer after `self` expires.
        unsafe {
            ptr::write(
                accounts[0].as_mut_ptr(),
                InstructionAccount::writable(&*(self.account.address() as *const _)),
            );
            ptr::write(
                accounts[1].as_mut_ptr(),
                InstructionAccount::readonly(&*(self.mint.address() as *const _)),
            );
        }

        2
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> usize {
        data[0].write(Self::DISCRIMINATOR);
        write_bytes(
            &mut data[1..INITIALIZE_ACCOUNT_3_INSTRUCTION_DATA_LEN],
            self.owner.as_array(),
        );
        INITIALIZE_ACCOUNT_3_INSTRUCTION_DATA_LEN
    }
}
