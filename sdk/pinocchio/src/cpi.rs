//! Cross-program invocation helpers.

use core::{mem::MaybeUninit, ops::Deref};

use crate::{
    account_info::{AccountInfo, BorrowState},
    instruction::{Account, AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

/// Maximum number of accounts that can be passed to a cross-program invocation.
pub const MAX_CPI_ACCOUNTS: usize = 64;

/// A constant to indicate that an account is read-only when passed as
/// an account priviledge.
pub const READONLY: bool = false;

/// A constant to indicate that an account is writable when passed as
/// an account priviledge.
pub const WRITABLE: bool = true;

/// A type representing the privileges of an account for
/// `invoke_instruction` and `invoke_instruction_signed`.
///
/// The first element of the tuple indicates whether the account is
/// writable or not, and the second element indicates whether the
/// account is a signer or not.
pub type Priviledge = (bool, bool);

/// Invoke a cross-program instruction.
///
/// This function is a convenience wrapper around [`invoke_signed`] that
/// passes an empty signer seeds slice.
///
/// # Important
///
/// The accounts on the `account_infos` slice must be in the same order as the
/// `accounts` field of the `instruction`.
#[inline(always)]
pub fn invoke<const ACCOUNTS: usize>(
    instruction: &Instruction,
    account_infos: &[&AccountInfo; ACCOUNTS],
) -> ProgramResult {
    invoke_signed(instruction, account_infos, &[])
}

/// Invoke a cross-program instruction from a slice of `AccountInfo`s.
///
/// This function is a convenience wrapper around [`slice_invoke_signed`]
/// that passes an empty signer seeds slice.
///
/// # Important
///
/// The accounts on the `account_infos` slice must be in the same order as the
/// `accounts` field of the `instruction`.
#[inline(always)]
pub fn slice_invoke(instruction: &Instruction, account_infos: &[&AccountInfo]) -> ProgramResult {
    slice_invoke_signed(instruction, account_infos, &[])
}

/// Invoke a cross-program instruction with signatures.
///
/// This function performs validation of the `account_infos` slice to ensure that:
///   1. The accounts match the expected accounts in the instruction, i.e., their
///      `Pubkey` matches the `pubkey` in the `AccountMeta`.
///   2. The borrow state of the accounts is compatible with the mutability of the
///      accounts in the instruction.
///
/// This validation is done to ensure that the borrow checker rules are followed,
/// consuming CUs in the process. There are two alternatives to this function that
/// have lower CU consumption:
///   * `invoke_instruction_signed` - does not perform `Pubkey` validation since
///      it creates each `AccountMeta` from the `Account` directly.
///   * `invoke_signed_unchecked` - does not perform any validation. This should
///      only be used when the caller is sure that the borrow checker rules are
///      followed.
///
/// # Important
///
/// The accounts on the `account_infos` slice must be in the same order as the
/// `accounts` field of the `instruction`.
#[inline]
pub fn invoke_signed<const ACCOUNTS: usize>(
    instruction: &Instruction,
    account_infos: &[&AccountInfo; ACCOUNTS],
    signers_seeds: &[Signer],
) -> ProgramResult {
    const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
    let mut accounts = [UNINIT; ACCOUNTS];

    account_infos
        .iter()
        .zip(instruction.accounts.iter())
        .enumerate()
        .try_for_each(|(index, (account_info, account_meta))| {
            // In order to check whether the borrow state is compatible
            // with the invocation, we need to check that we have the
            // correct account info and meta pair
            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }

            // Check whether all account infos can be safely borrowed according
            // to their mutability on the instruction or not.
            let state = if account_meta.is_writable {
                BorrowState::Borrowed
            } else {
                BorrowState::MutablyBorrowed
            };

            if account_info.is_borrowed(state) {
                return Err(ProgramError::AccountBorrowFailed);
            }

            // SAFETY: There are `ACCOUNTS` account infos.
            unsafe {
                accounts
                    .get_unchecked_mut(index)
                    .write(Account::from(*account_info));
            }

            Ok(())
        })?;

    // SAFETY: At this point it is guaranteed that all account infos are
    // borrowable according to their mutability on the instruction.
    unsafe {
        invoke_signed_unchecked(
            instruction,
            core::slice::from_raw_parts(accounts.as_ptr() as _, ACCOUNTS),
            signers_seeds,
        );
    }

    Ok(())
}

/// Invoke a cross-program instruction with signatures from a slice of
/// `AccountInfo`s.
///
/// This function performs validation of the `account_infos` slice to ensure that:
///   1. The accounts match the expected accounts in the instruction, i.e., their
///      `Pubkey` matches the `pubkey` in the `AccountMeta`.
///   2. The borrow state of the accounts is compatible with the mutability of the
///      accounts in the instruction.
///
/// This validation is done to ensure that the borrow checker rules are followed,
/// consuming CUs in the process. The `invoke_signed_unchecked` is an alternative
/// to this function that have lower CU consumption since it does not perform
/// any validation. This should only be used when the caller is sure that the borrow
/// checker rules are followed.
///
/// # Important
///
/// The accounts on the `account_infos` slice must be in the same order as the
/// `accounts` field of the `instruction`.
#[inline]
pub fn slice_invoke_signed(
    instruction: &Instruction,
    account_infos: &[&AccountInfo],
    signers_seeds: &[Signer],
) -> ProgramResult {
    if account_infos.len() > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
    let mut accounts = [UNINIT; MAX_CPI_ACCOUNTS];

    account_infos
        .iter()
        .zip(instruction.accounts.iter())
        .enumerate()
        .try_for_each(|(index, (account_info, account_meta))| {
            // In order to check whether the borrow state is compatible
            // with the invocation, we need to check that we have the
            // correct account info and meta pair
            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }

            // Check whether all account infos can be safely borrowed according
            // to their mutability on the instruction or not.
            let state = if account_meta.is_writable {
                BorrowState::Borrowed
            } else {
                BorrowState::MutablyBorrowed
            };

            if account_info.is_borrowed(state) {
                return Err(ProgramError::AccountBorrowFailed);
            }

            // SAFETY: There are `MAX_CPI_ACCOUNTS` account infos in the
            // worst case.
            unsafe {
                accounts
                    .get_unchecked_mut(index)
                    .write(Account::from(*account_info));
            }

            Ok(())
        })?;

    // SAFETY: At this point it is guaranteed that all account infos are
    // borrowable according to their mutability on the instruction, which
    // in the worst case is more than what is needed.
    unsafe {
        invoke_signed_unchecked(
            instruction,
            core::slice::from_raw_parts(accounts.as_ptr() as _, account_infos.len()),
            signers_seeds,
        );
    }

    Ok(())
}

/// Invoke a cross-program instruction but don't enforce Rust's aliasing rules.
///
/// This function does not check that [`Account`]s are properly borrowable.
/// Those checks consume CUs that this function avoids.
///
/// # Safety
///
/// If any of the writable accounts passed to the callee contain data that is
/// borrowed within the calling program, and that data is written to by the
/// callee, then Rust's aliasing rules will be violated and cause undefined
/// behavior.
#[inline(always)]
pub unsafe fn invoke_unchecked(instruction: &Instruction, accounts: &[Account]) {
    invoke_signed_unchecked(instruction, accounts, &[])
}

/// Invoke a cross-program instruction with signatures but don't enforce Rust's
/// aliasing rules.
///
/// This function does not check that [`Account`]s are properly borrowable.
/// Those checks consume CUs that this function avoids.
///
/// # Safety
///
/// If any of the writable accounts passed to the callee contain data that is
/// borrowed within the calling program, and that data is written to by the
/// callee, then Rust's aliasing rules will be violated and cause undefined
/// behavior.
#[inline(always)]
pub unsafe fn invoke_signed_unchecked(
    instruction: &Instruction,
    accounts: &[Account],
    signers_seeds: &[Signer],
) {
    invoke_instruction_signed_unchecked(
        instruction.program_id,
        instruction.accounts,
        instruction.data,
        accounts,
        signers_seeds,
    );
}

/// Invoke a cross-program instruction.
#[inline(always)]
pub fn invoke_instruction<const ACCOUNTS: usize>(
    program_id: &Pubkey,
    priviledges: &[Priviledge; ACCOUNTS],
    instruction_data: &[u8],
    accounts: &[Account; ACCOUNTS],
) -> ProgramResult {
    invoke_instruction_signed(program_id, priviledges, instruction_data, accounts, &[])
}

/// Invoke a cross-program instruction with signatures.
#[inline]
pub fn invoke_instruction_signed<const ACCOUNTS: usize>(
    program_id: &Pubkey,
    priviledges: &[Priviledge; ACCOUNTS],
    instruction_data: &[u8],
    accounts: &[Account; ACCOUNTS],
    signers_seeds: &[Signer],
) -> ProgramResult {
    let mut account_metas = MaybeUninit::<[AccountMeta; ACCOUNTS]>::uninit();
    let mut account_metas_ptr = account_metas.as_mut_ptr() as *mut AccountMeta;

    accounts
        .iter()
        .zip(priviledges.iter())
        .try_for_each(|(account, priviledge)| {
            // Check whether all account infos can be safely borrowed according
            // to their mutability on the instruction or not.
            if account.is_borrowed(match priviledge.0 {
                true => BorrowState::Borrowed,
                false => BorrowState::MutablyBorrowed,
            }) {
                return Err(ProgramError::AccountBorrowFailed);
            }

            // SAFETY: There are `ACCOUNTS` account metas.
            unsafe {
                account_metas_ptr.write(AccountMeta::new(
                    account.key(),
                    priviledge.0,
                    priviledge.1,
                ));

                account_metas_ptr = account_metas_ptr.add(1);
            }

            Ok(())
        })?;

    // SAFETY: At this point it is guaranteed that all account infos are
    // borrowable according to their mutability on the instruction.
    unsafe {
        invoke_instruction_signed_unchecked(
            program_id,
            account_metas.assume_init_ref(),
            instruction_data,
            accounts,
            signers_seeds,
        );
    }

    Ok(())
}

/// Invoke a cross-program instruction with signatures but don't enforce Rust's
/// aliasing rules.
///
/// This function does not check that [`Account`]s are properly borrowable.
/// Those checks consume CUs that this function avoids.
///
/// # Safety
///
/// If any of the writable accounts passed to the callee contain data that is
/// borrowed within the calling program, and that data is written to by the
/// callee, then Rust's aliasing rules will be violated and cause undefined
/// behavior.
#[inline(always)]
pub unsafe fn invoke_instruction_signed_unchecked(
    program_id: &Pubkey,
    account_metas: &[AccountMeta],
    instruction_data: &[u8],
    accounts: &[Account],
    signers_seeds: &[Signer],
) {
    #[cfg(target_os = "solana")]
    {
        /// An `Instruction` as expected by `sol_invoke_signed_c`.
        ///
        /// DO NOT EXPOSE THIS STRUCT:
        ///
        /// To ensure pointers are valid upon use, the scope of this struct should
        /// only be limited to the stack where sol_invoke_signed_c happens and then
        /// discarded immediately after.
        #[repr(C)]
        struct CInstruction<'a> {
            /// Public key of the program.
            program_id: *const Pubkey,

            /// Accounts expected by the program instruction.
            accounts: *const AccountMeta<'a>,

            /// Number of accounts expected by the program instruction.
            accounts_len: u64,

            /// Data expected by the program instruction.
            data: *const u8,

            /// Length of the data expected by the program instruction.
            data_len: u64,
        }

        let cpi_instruction = CInstruction {
            program_id,
            accounts: account_metas.as_ptr(),
            accounts_len: account_metas.len() as u64,
            data: instruction_data.as_ptr(),
            data_len: instruction_data.len() as u64,
        };

        unsafe {
            crate::syscalls::sol_invoke_signed_c(
                &cpi_instruction as *const _ as *const u8,
                accounts as *const _ as *const u8,
                accounts.len() as u64,
                signers_seeds as *const _ as *const u8,
                signers_seeds.len() as u64,
            )
        };
    }

    #[cfg(not(target_os = "solana"))]
    {
        core::hint::black_box((
            program_id,
            account_metas,
            instruction_data,
            accounts,
            signers_seeds,
        ));
        unreachable!();
    }
}

/// Maximum size that can be set using [`set_return_data`].
pub const MAX_RETURN_DATA: usize = 1024;

/// Set the running program's return data.
///
/// Return data is a dedicated per-transaction buffer for data passed
/// from cross-program invoked programs back to their caller.
///
/// The maximum size of return data is [`MAX_RETURN_DATA`]. Return data is
/// retrieved by the caller with [`get_return_data`].
#[inline(always)]
pub fn set_return_data(data: &[u8]) {
    #[cfg(target_os = "solana")]
    unsafe {
        crate::syscalls::sol_set_return_data(data.as_ptr(), data.len() as u64)
    };

    #[cfg(not(target_os = "solana"))]
    core::hint::black_box(data);
}

/// Get the return data from an invoked program.
///
/// For every transaction there is a single buffer with maximum length
/// [`MAX_RETURN_DATA`], paired with a [`Pubkey`] representing the program ID of
/// the program that most recently set the return data. Thus the return data is
/// a global resource and care must be taken to ensure that it represents what
/// is expected: called programs are free to set or not set the return data; and
/// the return data may represent values set by programs multiple calls down the
/// call stack, depending on the circumstances of transaction execution.
///
/// Return data is set by the callee with [`set_return_data`].
///
/// Return data is cleared before every CPI invocation &mdash; a program that
/// has invoked no other programs can expect the return data to be `None`; if no
/// return data was set by the previous CPI invocation, then this function
/// returns `None`.
///
/// Return data is not cleared after returning from CPI invocations &mdash; a
/// program that has called another program may retrieve return data that was
/// not set by the called program, but instead set by a program further down the
/// call stack; or, if a program calls itself recursively, it is possible that
/// the return data was not set by the immediate call to that program, but by a
/// subsequent recursive call to that program. Likewise, an external RPC caller
/// may see return data that was not set by the program it is directly calling,
/// but by a program that program called.
///
/// For more about return data see the [documentation for the return data proposal][rdp].
///
/// [rdp]: https://docs.solanalabs.com/proposals/return-data
#[inline]
pub fn get_return_data() -> Option<ReturnData> {
    #[cfg(target_os = "solana")]
    {
        const UNINIT_BYTE: core::mem::MaybeUninit<u8> = core::mem::MaybeUninit::<u8>::uninit();
        let mut data = [UNINIT_BYTE; MAX_RETURN_DATA];
        let mut program_id = MaybeUninit::<Pubkey>::uninit();

        let size = unsafe {
            crate::syscalls::sol_get_return_data(
                data.as_mut_ptr() as *mut u8,
                data.len() as u64,
                program_id.as_mut_ptr() as *mut Pubkey,
            )
        };

        if size == 0 {
            None
        } else {
            Some(ReturnData {
                program_id: unsafe { program_id.assume_init() },
                data,
                size: core::cmp::min(size as usize, MAX_RETURN_DATA),
            })
        }
    }

    #[cfg(not(target_os = "solana"))]
    core::hint::black_box(None)
}

/// Struct to hold the return data from an invoked program.
pub struct ReturnData {
    /// Program that most recently set the return data.
    program_id: Pubkey,

    /// Return data set by the program.
    data: [core::mem::MaybeUninit<u8>; MAX_RETURN_DATA],

    /// Length of the return data.
    size: usize,
}

impl ReturnData {
    /// Returns the program that most recently set the return data.
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Return the data set by the program.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as _, self.size) }
    }
}

impl Deref for ReturnData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
