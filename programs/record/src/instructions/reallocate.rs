use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Reallocate additional space in a record account.
///
/// ### Accounts:
///   0. `[WRITE]` Record account, must be previously initialized.
///   1. `[SIGNER]` Record authority.
pub struct Reallocate<'a> {
    /// Record account.
    pub account: &'a AccountView,
    /// Record authority.
    pub authority: &'a AccountView,

    /// The length of the data to hold in the record account excluding meta data.
    pub data_length: u64,
}

impl Reallocate<'_> {
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

        // instruction data
        // -  [0..1]: instruction discriminator
        // -  [1..9]: data length
        let mut instruction_data = core::mem::MaybeUninit::<[u8; 9]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping([4u8].as_ptr(), ptr, 1);
            core::ptr::copy_nonoverlapping(self.data_length.to_le_bytes().as_ptr(), ptr.add(1), 8);
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: &instruction_accounts,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 9) },
        };

        invoke_signed(&instruction, &[self.account, self.authority], signers)
    }
}
