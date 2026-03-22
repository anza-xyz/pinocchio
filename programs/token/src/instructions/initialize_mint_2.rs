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

const INITIALIZE_MINT_2_INSTRUCTION_DATA_LEN: usize = 67;

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
pub struct InitializeMint2<'a> {
    /// Mint Account.
    pub mint: &'a AccountView,
    /// Decimals.
    pub decimals: u8,
    /// Mint Authority.
    pub mint_authority: &'a Address,
    /// Freeze Authority.
    pub freeze_authority: Option<&'a Address>,
}

impl InitializeMint2<'_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u8 = 20;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 1];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts);

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 1];
        let written_accounts = self.write_accounts(&mut accounts);

        let mut instruction_data = [UNINIT_BYTE; INITIALIZE_MINT_2_INSTRUCTION_DATA_LEN];
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

impl super::sealed::Sealed for InitializeMint2<'_> {}

impl Batchable for InitializeMint2<'_> {
    #[inline(always)]
    fn write_accounts(&self, accounts: &mut [MaybeUninit<CpiAccount>]) -> usize {
        accounts[0].write(CpiAccount::from(self.mint));
        1
    }

    #[inline(always)]
    fn write_instruction_accounts(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount>],
    ) -> usize {
        // SAFETY: The written address reference is borrowed from `self`, and
        // callers must not use the output buffer after `self` expires.
        unsafe {
            ptr::write(
                accounts[0].as_mut_ptr(),
                InstructionAccount::writable(&*(self.mint.address() as *const _)),
            );
        }

        1
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> usize {
        data[0].write(Self::DISCRIMINATOR);
        data[1].write(self.decimals);
        write_bytes(&mut data[2..34], self.mint_authority.as_array());

        if let Some(freeze_auth) = self.freeze_authority {
            data[34].write(1);
            write_bytes(
                &mut data[35..INITIALIZE_MINT_2_INSTRUCTION_DATA_LEN],
                freeze_auth.as_array(),
            );
            INITIALIZE_MINT_2_INSTRUCTION_DATA_LEN
        } else {
            data[34].write(0);
            35
        }
    }
}
