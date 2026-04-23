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

/// Transfer lamports from a derived address.
///
/// ### Accounts:
///   0. `[WRITE]` Funding account
///   1. `[SIGNER]` Base for funding account
///   2. `[WRITE]` Recipient account
pub struct TransferWithSeed<'account, 'address, 'seed> {
    /// Funding account.
    pub from: &'account AccountView,

    /// Base account.
    ///
    /// The account matching the base [`Address`] below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: &'account AccountView,

    /// Recipient account.
    pub to: &'account AccountView,

    /// Amount of lamports to transfer.
    pub lamports: u64,

    /// String of ASCII chars, no longer than [`MAX_SEED_LEN`](https://docs.rs/solana-address/latest/solana_address/constant.MAX_SEED_LEN.html).
    pub seed: &'seed str,

    /// Address of program that will own the new account.
    pub owner: &'address Address,
}

impl TransferWithSeed<'_, '_, '_> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: u32 = 11;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts.
        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 3];
        instruction_accounts[0].write(InstructionAccount::writable(self.from.address()));
        instruction_accounts[1].write(InstructionAccount::readonly_signer(self.base.address()));
        instruction_accounts[2].write(InstructionAccount::writable(self.to.address()));

        let seed_bytes = self.seed.as_bytes();

        if seed_bytes.len() > MAX_SEED_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports amount
        // - [12..20]: seed length
        // - [20..  ]: seed (max 32)
        // - [.. +32]: owner address
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 84];
        // SAFETY: All writes are within bounds of the allocated data.
        unsafe {
            let dst = instruction_data.as_mut_ptr() as *mut u8;

            copy_nonoverlapping(
                Self::DISCRIMINATOR.to_le_bytes().as_ptr(),
                dst,
                size_of::<u32>(),
            );

            copy_nonoverlapping(
                self.lamports.to_le_bytes().as_ptr(),
                dst.add(4),
                size_of::<u64>(),
            );

            copy_nonoverlapping(
                u64::to_le_bytes(seed_bytes.len() as u64).as_ptr(),
                dst.add(12),
                size_of::<u64>(),
            );

            copy_nonoverlapping(seed_bytes.as_ptr(), dst.add(20), seed_bytes.len());

            copy_nonoverlapping(
                self.owner.as_ref().as_ptr(),
                dst.add(20 + seed_bytes.len()),
                ADDRESS_BYTES,
            );
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            // SAFETY: `instruction_accounts` was initialized.
            accounts: unsafe { from_raw_parts(instruction_accounts.as_ptr() as _, 3) },
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 52 + seed_bytes.len()) },
        };

        if self.from.is_borrowed() | self.to.is_borrowed() {
            return Err(ProgramError::AccountBorrowFailed);
        }

        let mut accounts = [const { MaybeUninit::<CpiAccount>::uninit() }; 3];
        CpiAccount::init_from_account_view(self.from, &mut accounts[0]);
        CpiAccount::init_from_account_view(self.base, &mut accounts[1]);
        CpiAccount::init_from_account_view(self.to, &mut accounts[2]);

        // SAFETY: `accounts` was initialized and not borrowed.
        unsafe {
            invoke_signed_unchecked(
                &instruction,
                from_raw_parts(accounts.as_ptr() as _, 3),
                signers,
            )
        };

        Ok(())
    }
}
