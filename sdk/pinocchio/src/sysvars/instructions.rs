use crate::{
    account_info::AccountInfo, instruction::AccountMeta, program_error::ProgramError,
    pubkey::Pubkey, sanitize_error::SanitizeError,
};

use core::mem::size_of;

/// Sysvar1nstructions1111111111111111111111111
pub const INSTRUCTIONS_ID: Pubkey = [
    0x06, 0xa7, 0xd5, 0x17, 0x18, 0x7b, 0xd1, 0x66, 
    0x35, 0xda, 0xd4, 0x04, 0x55, 0xfd, 0xc2, 0xc0,
    0xc1, 0x24, 0xc6, 0x8f, 0x21, 0x56, 0x75, 0xa5, 
    0xdb, 0xba, 0xcb, 0x5f, 0x08, 0x00, 0x00, 0x00,
];

pub struct Instructions();

/// Load the current `Instruction`'s index in the currently executing
/// `Transaction`.
///
/// `data` is the instructions sysvar account data.
///
/// # Safety
///
/// This function is unsafe because it does not verify the address of the sysvar account.
/// 
/// It is typically used internally within the `load_current_index_checked` function, which
/// performs the necessary address verification. However, to optimize performance for users
/// who have already conducted this check elsewhere in their code, we have exposed it as an
/// unsafe function rather than making it private, as was done in the original implementation.
#[inline(always)]
pub unsafe fn load_current_index(data: &[u8]) -> u16 {
    let len = data.len();
    *(data.as_ptr().add(len - 2) as *const u16)
}

/// Load the current `Instruction`'s index in the currently executing
/// `Transaction`.
///
/// # Errors
///
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
/// Return [`ProgramError::AccountBorrowFailed`] if the account has already been borrowed.
#[inline(always)]
pub fn load_current_index_checked(
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<u16, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let instruction_sysvar = instruction_sysvar_account_info.try_borrow_data()?;
    let index = unsafe { load_current_index(&instruction_sysvar) };
    Ok(index)
}

/// Store the current `Instruction`'s index in the instructions sysvar data.
#[inline(always)]
pub fn store_current_index(data: &mut [u8], instruction_index: u16) {
    let last_index = data.len() - 2;
    unsafe {
        *(data.as_mut_ptr().add(last_index) as *mut u16) = instruction_index.to_le();
    }
}

/// Load an `Instruction` in the currently executing `Transaction` at the
/// specified index.
///
/// `data` is the instructions sysvar account data.
///
/// # Safety
/// 
/// This function is unsafe because it does not verify the address of the sysvar account and
/// does not check if the index is out of bounds.
/// 
/// It is typically used internally within the `load_instruction_at_checked` function, which
/// performs the necessary address and index verification. However, to optimize performance for 
/// users who have already conducted this check elsewhere in their code or are sure that the 
/// index is in bounds, we have exposed it as an unsafe function rather than making it private, 
/// as was done in the original implementation.
pub unsafe fn deserialize_instruction(index: usize, data: &[u8]) -> Result<IntrospectedInstruction, SanitizeError> {
    let num_instructions = u16::from_le(*(data.as_ptr() as *const u16));
    if index >= num_instructions as usize {
        return Err(SanitizeError::IndexOutOfBounds);
    }

    let offset = u16::from_le(*(data
        .as_ptr()
        .add(size_of::<u16>() + index * size_of::<u16>()) as *const u16));

    Ok(IntrospectedInstruction {
        raw: data.as_ptr().add(offset as usize),
    })
}

/// Load an `Instruction` in the currently executing `Transaction` at the
/// specified index.
///
/// `data` is the instructions sysvar account data.
///
/// # Safety
/// 
/// This function is unsafe because it does not verify the address of the sysvar account.
/// 
/// It is typically used internally within the `load_instruction_at_checked` function, which
/// performs the necessary address verification. However, to optimize performance for users 
/// who have already conducted this check elsewhere in their code, we have exposed it as an 
/// unsafe function rather than making it private, as was done in the original implementation.
#[inline(always)]
pub unsafe fn load_instruction_at(
    index: usize, 
    data: &[u8]
) -> Result<IntrospectedInstruction, SanitizeError> {
    deserialize_instruction(index, data)
}

/// Load an `Instruction` in the currently executing `Transaction` at the
/// specified index.
/// 
/// # Errors
/// 
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
/// Returns [`ProgramError::InvalidArgument`] if the index is out of bounds.
#[inline(always)]
pub fn load_instruction_at_checked(
    index: usize,
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<IntrospectedInstruction, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let instruction_sysvar_data = instruction_sysvar_account_info.try_borrow_data()?;
    unsafe {
        load_instruction_at(index, &instruction_sysvar_data)
            .map(|instr| instr.clone())
            .map_err(|err| match err {
                SanitizeError::IndexOutOfBounds => ProgramError::InvalidArgument,
                _ => ProgramError::InvalidInstructionData,
            })
    }
}

/// Returns the `Instruction` relative to the current `Instruction` in the
/// currently executing `Transaction`.
///
/// # Errors
///
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
/// Returns [`ProgramError::InvalidArgument`] if the index is out of bounds.
pub fn get_instruction_relative(
    index_relative_to_current: i64,
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<IntrospectedInstruction, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let current_index = load_current_index_checked(&instruction_sysvar_account_info)? as i64;
    let index = current_index.saturating_add(index_relative_to_current);
    if index < 0 {
        return Err(ProgramError::InvalidArgument);
    }

    let instruction_sysvar_data = instruction_sysvar_account_info.try_borrow_data()?;
    unsafe {
        load_instruction_at(index as usize, &instruction_sysvar_data)
            .map(|instr| instr.clone())
            .map_err(|err| match err {
                SanitizeError::IndexOutOfBounds => ProgramError::InvalidArgument,
                _ => ProgramError::InvalidInstructionData,
            })
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedInstruction {
    pub raw: *const u8,
}

impl IntrospectedInstruction {
    /// Get the account meta at the specified index.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe because it does not verify if the index is out of bounds.
    /// 
    /// It is typically used internally within the `get_account_meta_at` function, which
    /// performs the necessary index verification. However, to optimize performance for users 
    /// who are sure that the index is in bounds, we have exposed it as an unsafe function.
    pub unsafe fn get_account_meta_at_unchecked(&self, index: usize) -> IntrospectedAccountMeta {
        IntrospectedAccountMeta {
            raw: self
                .raw
                .add(size_of::<u16>() + index * IntrospectedAccountMeta::SPACE)
                    as *const u8,
        }
    }

    /// Get the account meta at the specified index.
    /// 
    /// # Errors
    /// 
    /// Returns [`SanitizeError::IndexOutOfBounds`] if the index is out of bounds.
    pub fn get_account_meta_at(
        &self,
        index: usize,
    ) -> Result<IntrospectedAccountMeta, SanitizeError> {
        if index >= unsafe { *(self.raw as *const u16) } as usize {
            return Err(SanitizeError::IndexOutOfBounds);
        }

        Ok(unsafe { self.get_account_meta_at_unchecked(index) })
    }

    /// Get the program ID of the `Instruction`.
    pub fn get_program_id(&self) -> &Pubkey {
        unsafe {
            let num_accounts = *(self.raw as *const u16);
            &*(self
                .raw
                .add(size_of::<u16>() + num_accounts as usize * IntrospectedAccountMeta::SPACE)
                    as *const Pubkey)
        }
    }

    /// Get the instruction data of the `Instruction`.
    pub fn get_instruction_data(&self) -> &[u8] {
        unsafe {
            let num_accounts = u16::from_le(*(self.raw as *const u16));
            let data_len = u16::from_le(
                *(self.raw.add(
                    size_of::<u16>()
                        + num_accounts as usize * IntrospectedAccountMeta::SPACE
                        + size_of::<Pubkey>(),
                ) as *const u16),
            );

            core::slice::from_raw_parts(
                self.raw.add(
                    size_of::<u16>()
                        + num_accounts as usize * IntrospectedAccountMeta::SPACE
                        + size_of::<Pubkey>()
                        + size_of::<u16>(),
                ),
                data_len as usize,
            )
        }
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedAccountMeta {
    pub raw: *const u8,
}

/// The bit positions for the signer flags in the `AccountMeta`.
const IS_SIGNER_BIT: u8 = 0;

/// The bit positions for the writable flags in the `AccountMeta`.
const IS_WRITABLE_BIT: u8 = 1;

impl IntrospectedAccountMeta {
    const SPACE: usize = 33; 

    /// Get the writable flag of the `AccountMeta`.
    pub fn is_writable(&self) -> bool {
        unsafe { (*self.raw & (1 << IS_WRITABLE_BIT)) != 0 }
    }

    /// Get the signer flag of the `AccountMeta`.
    pub fn is_signer(&self) -> bool {
        unsafe { (*self.raw & (1 << IS_SIGNER_BIT)) != 0 }
    }

    /// Get the key of the `AccountMeta`.
    pub fn key(&self) -> &Pubkey {
        unsafe { &*(self.raw.add(size_of::<u8>()) as *const Pubkey) }
    }

    /// Convert the `IntrospectedAccountMeta` to a `AccountMeta`.
    pub fn to_account_meta(&self) -> AccountMeta {
        AccountMeta::new(self.key(), self.is_signer(), self.is_writable())
    }
}
