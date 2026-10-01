use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Write to the provided record account.
///
/// ### Accounts:
///   0. `[WRITE]` Record account, must be previously initialized.
///   1. `[SIGNER]` Current record authority.
pub struct Write<'a> {
    /// Record account.
    pub account: &'a AccountView,
    /// Record authority.
    pub authority: &'a AccountView,

    ///  Offset to start writing record.
    pub offset: u64,
    ///  Data to replace the existing record data
    pub data: &'a [u8],
}

impl Write<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.account.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let data_len = self.data.len();

        // instruction data
        // -  [0..1]: instruction discriminator
        // -  [1..9]: offset
        // -  [9..13]: data length (u32)
        // -  [13..]: data
        let mut instruction_data = core::mem::MaybeUninit::<[u8; 13]>::uninit();

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping([1u8].as_ptr(), ptr, 1);
            core::ptr::copy_nonoverlapping(self.offset.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping((data_len as u32).to_le_bytes().as_ptr(), ptr.add(9), 4);
            core::ptr::copy_nonoverlapping(self.data.as_ptr(), ptr.add(13), data_len);
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: unsafe {
                core::slice::from_raw_parts(
                    instruction_data.as_ptr() as *const u8,
                    1 + 8 + 4 + data_len,
                )
            },
        };

        invoke_signed(&instruction, &[self.account, self.authority], signers)
    }
}
