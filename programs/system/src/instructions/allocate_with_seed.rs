use {
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    pinocchio::{
        address::MAX_SEED_LEN,
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
    solana_address::ADDRESS_BYTES,
};

/// Allocate space for and assign an account at an address derived
/// from a base address and a seed.
///
/// ### Accounts:
///   0. `[WRITE]` Allocated account
///   1. `[SIGNER]` Base account
pub struct AllocateWithSeed<'account, 'address, 'seed> {
    /// Allocated account.
    pub account: &'account AccountView,

    /// Base account.
    ///
    /// The account matching the base address below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'account AccountView,

    /// String of ASCII chars, no longer than [`MAX_SEED_LEN`](https://docs.rs/solana-address/latest/solana_address/constant.MAX_SEED_LEN.html).
    pub seed: &'seed str,

    /// Number of bytes of memory to allocate.
    pub space: u64,

    /// Address of program that will own the new account.
    pub owner: &'address Address,
}

impl AllocateWithSeed<'_, '_, '_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 9;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        instruction_accounts[0].write(InstructionAccount::writable(self.account.address()));
        instruction_accounts[1].write(InstructionAccount::readonly_signer(self.base.address()));

        let seed_bytes = self.seed.as_bytes();

        if seed_bytes.len() > MAX_SEED_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base address
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [..  +8]: account space
        // - [.. +32]: owner address
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 116];
        // SAFETY: All writes are within bounds of the allocated data.
        unsafe {
            let dst = instruction_data.as_mut_ptr() as *mut u8;

            copy_nonoverlapping(
                Self::DISCRIMINATOR.to_le_bytes().as_ptr(),
                dst,
                size_of::<u32>(),
            );

            copy_nonoverlapping(
                self.base.address().as_ref().as_ptr(),
                dst.add(4),
                ADDRESS_BYTES,
            );

            copy_nonoverlapping(
                u64::to_le_bytes(seed_bytes.len() as u64).as_ptr(),
                dst.add(36),
                size_of::<u64>(),
            );

            copy_nonoverlapping(seed_bytes.as_ptr(), dst.add(44), seed_bytes.len());

            copy_nonoverlapping(
                self.space.to_le_bytes().as_ptr(),
                dst.add(44 + seed_bytes.len()),
                size_of::<u64>(),
            );

            copy_nonoverlapping(
                self.owner.as_ref().as_ptr(),
                dst.add(52 + seed_bytes.len()),
                ADDRESS_BYTES,
            );
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 2) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 84 + seed_bytes.len()) },
        };

        if self.account.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 2];
        CpiAccount::init_from_account_view(self.account, &mut accounts[0]);
        CpiAccount::init_from_account_view(self.base, &mut accounts[1]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                from_raw_parts(accounts.as_ptr() as _, 2),
                signers,
            )
        };

        Ok(())
    }
}
