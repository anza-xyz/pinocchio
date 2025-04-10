use core::mem::MaybeUninit;

use crate::{
    cpi::invoke_signed_unchecked, instruction::Account, program_error::ProgramError, syscalls,
};

use super::Runtime;

pub struct SolanaRuntime;

impl Runtime for SolanaRuntime {
    ////////////////////////////////////////////////////////////////////////////
    // LOG SYS CALLS
    ////////////////////////////////////////////////////////////////////////////

    fn sol_log(message: &str) {
        unsafe {
            crate::syscalls::sol_log_(message.as_ptr(), message.len() as u64);
        }
    }

    fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        unsafe {
            crate::syscalls::sol_log_64_(arg1, arg2, arg3, arg4, arg5);
        }
    }

    fn sol_log_data(data: &[&[u8]]) {
        unsafe {
            crate::syscalls::sol_log_data(data as *const _ as *const u8, data.len() as u64);
        };
    }

    fn sol_log_compute_units() {
        unsafe {
            crate::syscalls::sol_log_compute_units_();
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // MEM SYS CALLS
    ////////////////////////////////////////////////////////////////////////////

    unsafe fn sol_memcpy(dst: &mut [u8], src: &[u8], n: usize) {
        syscalls::sol_memcpy_(dst.as_mut_ptr(), src.as_ptr(), n as u64);
    }

    unsafe fn sol_memmove(dst: *mut u8, src: *mut u8, n: usize) {
        syscalls::sol_memmove_(dst, src, n as u64);
    }

    unsafe fn sol_memcmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
        let mut result = 0;
        syscalls::sol_memcmp_(s1.as_ptr(), s2.as_ptr(), n as u64, &mut result as *mut i32);
        result
    }

    unsafe fn sol_memset(s: &mut [u8], c: u8, n: usize) {
        syscalls::sol_memset_(s.as_mut_ptr(), c, n as u64);
    }

    ////////////////////////////////////////////////////////////////////////////
    // CPI CALLS
    ////////////////////////////////////////////////////////////////////////////

    fn invoke_signed<const ACCOUNTS: usize>(
        instruction: &crate::instruction::Instruction,
        account_infos: &[&crate::account_info::AccountInfo; ACCOUNTS],
        signers_seeds: &[crate::instruction::Signer],
    ) -> crate::ProgramResult {
        if instruction.accounts.len() < ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
        let mut accounts = [UNINIT; ACCOUNTS];

        for index in 0..ACCOUNTS {
            let account_info = account_infos[index];
            let account_meta = &instruction.accounts[index];

            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }

            if account_meta.is_writable {
                account_info.check_borrow_mut_data()?;
                account_info.check_borrow_mut_lamports()?;
            } else {
                account_info.check_borrow_data()?;
                account_info.check_borrow_lamports()?;
            }

            accounts[index].write(Account::from(account_infos[index]));
        }

        unsafe {
            invoke_signed_unchecked(
                instruction,
                core::slice::from_raw_parts(accounts.as_ptr() as _, ACCOUNTS),
                signers_seeds,
            );
        }

        Ok(())
    }

    unsafe fn invoke_signed_access_unchecked<const ACCOUNTS: usize>(
        instruction: &crate::instruction::Instruction,
        account_infos: &[&crate::account_info::AccountInfo; ACCOUNTS],
        signers_seeds: &[crate::instruction::Signer],
    ) -> crate::ProgramResult {
        if instruction.accounts.len() < ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
        let mut accounts = [UNINIT; ACCOUNTS];

        for index in 0..ACCOUNTS {
            let account_info = account_infos[index];
            let account_meta = &instruction.accounts[index];

            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }

            accounts[index].write(Account::from(account_infos[index]));
        }

        unsafe {
            invoke_signed_unchecked(
                instruction,
                core::slice::from_raw_parts(accounts.as_ptr() as _, ACCOUNTS),
                signers_seeds,
            );
        }

        Ok(())
    }

    fn sol_create_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        // Call via a system call to perform the calculation
        let mut bytes = core::mem::MaybeUninit::<[u8; PUBKEY_BYTES]>::uninit();

        let result = unsafe {
            crate::syscalls::sol_create_program_address(
                seeds as *const _ as *const u8,
                seeds.len() as u64,
                program_id as *const _ as *const u8,
                bytes.as_mut_ptr() as *mut u8,
            )
        };

        match result {
            // SAFETY: The syscall has initialized the bytes.
            crate::SUCCESS => Ok(unsafe { bytes.assume_init() }),
            _ => Err(result.into()),
        }
    }
}
