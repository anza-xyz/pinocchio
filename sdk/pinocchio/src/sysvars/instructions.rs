use crate::{
    account_info::AccountInfo, instruction::AccountMeta, program_error::ProgramError,
    pubkey::Pubkey, sanitize_error::SanitizeError,
};

use core::{marker::PhantomData, mem::size_of};

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
/// It is typically used internally within the `load_current_index` function, which
/// performs the necessary address verification. However, to optimize performance for users
/// who have already conducted this check elsewhere in their code, we have exposed it as an
/// unsafe function rather than making it private, as was done in the original implementation.
#[inline(always)]
pub unsafe fn load_current_index_unchecked(data: &[u8]) -> u16 {
    let len = data.len();
    u16::from_le(*(data.as_ptr().add(len - 2) as *const u16))
}

/// Load the current `Instruction`'s index in the currently executing
/// `Transaction`.
///
/// # Errors
///
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
/// Return [`ProgramError::AccountBorrowFailed`] if the account has already been borrowed.
#[inline(always)]
pub fn load_current_index(
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<u16, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let instruction_sysvar = instruction_sysvar_account_info.try_borrow_data()?;
    let index = unsafe { load_current_index_unchecked(&instruction_sysvar) };
    Ok(index)
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
/// It is typically used internally within the `load_instruction_at` function, which
/// performs the necessary address and index verification. However, to optimize performance for 
/// users who have already conducted this check elsewhere in their code or are sure that the 
/// index is in bounds, we have exposed it as an unsafe function rather than making it private, 
/// as was done in the original implementation.
pub unsafe fn deserialize_instruction_unchecked(index: usize, data: &[u8]) -> Result<IntrospectedInstruction, SanitizeError> {
    let offset = u16::from_le(*(data
        .as_ptr()
        .add(size_of::<u16>() + index * size_of::<u16>()) as *const u16));

    Ok(IntrospectedInstruction {
        raw: data.as_ptr().add(offset as usize),
        marker: PhantomData,
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
/// It is typically used internally within the `load_instruction_at` function, which
/// performs the necessary address verification. However, to optimize performance for users 
/// who have already conducted this check elsewhere in their code, we have exposed it as an 
/// unsafe function rather than making it private, as was done in the original implementation.
#[inline(always)]
pub unsafe fn load_instruction_at_unchecked(
    index: usize, 
    data: &[u8]
) -> Result<IntrospectedInstruction, SanitizeError> {
    let num_instructions = u16::from_le(*(data.as_ptr() as *const u16));
    if index >= num_instructions as usize {
        return Err(SanitizeError::IndexOutOfBounds);
    }

    deserialize_instruction_unchecked(index, data)
}

/// Load an `Instruction` in the currently executing `Transaction` at the
/// specified index.
/// 
/// # Errors
/// 
/// Returns [`ProgramError::UnsupportedSysvar`] if the given account's ID is not equal to [`ID`].
/// Returns [`ProgramError::InvalidArgument`] if the index is out of bounds.
#[inline(always)]
pub fn load_instruction_at(
    index: usize,
    instruction_sysvar_account_info: &AccountInfo,
) -> Result<IntrospectedInstruction, ProgramError> {
    if instruction_sysvar_account_info.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }

    let instruction_sysvar_data = SysvarAccountInfo::from_account_info(instruction_sysvar_account_info)?; 
    unsafe {
        load_instruction_at_unchecked(index, &instruction_sysvar_data.try_borrow_data()?)
            .map_err(|err| match err {
                SanitizeError::IndexOutOfBounds => ProgramError::InvalidArgument,
                _ => ProgramError::InvalidInstructionData,
            })
    }
}

/// Returns the `Instruction` relative to the current `Instruction` in the
/// currently executing `Transaction`.
///
/// `data` is the instructions sysvar account data.
///
/// # Safety
/// 
/// This function is unsafe because it does not verify the address of the sysvar account.
/// 
/// It is typically used internally within the `get_instruction_relative` function, which
/// performs the necessary address verification. However, to optimize performance for users 
/// who have already conducted this check elsewhere in their code, we have exposed it as an 
/// unsafe function rather than making it private, as was done in the original implementation.
#[inline(always)]
pub unsafe fn get_instruction_relative_unchecked(
    current_index: i64, 
    index_relative_to_current: i64,
    data: &[u8]
) -> Result<IntrospectedInstruction, SanitizeError> {
    let index = current_index.saturating_add(index_relative_to_current);
    if index < 0 {
        return Err(SanitizeError::IndexOutOfBounds);
    }

    let num_instructions = u16::from_le(*(data.as_ptr() as *const u16));
    if index >= num_instructions as i64 {
        return Err(SanitizeError::IndexOutOfBounds);
    }

    deserialize_instruction_unchecked(index as usize, data)
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

    let current_index = load_current_index(&instruction_sysvar_account_info)? as i64;
    let instruction_sysvar_data = SysvarAccountInfo::from_account_info(instruction_sysvar_account_info)?;

    unsafe {
        get_instruction_relative_unchecked(current_index, index_relative_to_current, &instruction_sysvar_data.try_borrow_data()?)
            .map_err(|err| match err {
                SanitizeError::IndexOutOfBounds => ProgramError::InvalidArgument,
                _ => ProgramError::InvalidInstructionData,
            })
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct SysvarAccountInfo<'a> {
    pub raw: *const u8,
    pub marker: PhantomData<&'a [u8]>,
}

impl<'a> SysvarAccountInfo<'a> {
    pub fn from_account_info(instruction_sysvar_account_info: &AccountInfo) -> Result<Self, ProgramError> {
        let data = instruction_sysvar_account_info.try_borrow_data()?;
        Ok(Self { raw: data.as_ptr(), marker: PhantomData })
    }

    pub fn try_borrow_data(&self) -> Result<&'a [u8], ProgramError> {
        let data_len = unsafe { *self.raw.sub(8) as *mut u64 };
        let data = unsafe { core::slice::from_raw_parts(self.raw, data_len as usize) };
        Ok(data)
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedInstruction<'a> {
    pub raw: *const u8,
    pub marker: PhantomData<&'a [u8]>,
}

impl<'a> IntrospectedInstruction<'a> {
    /// Get the account meta at the specified index.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe because it does not verify if the index is out of bounds.
    /// 
    /// It is typically used internally within the `get_account_meta_at` function, which
    /// performs the necessary index verification. However, to optimize performance for users 
    /// who are sure that the index is in bounds, we have exposed it as an unsafe function.
    pub unsafe fn get_account_meta_at_unchecked(&self, index: usize) -> &IntrospectedAccountMeta {
        let offset = core::mem::size_of::<u16>() + (index * IntrospectedAccountMeta::LEN);
        &*(self.raw.add(offset) as *const IntrospectedAccountMeta)
    }

    /// Get the account meta at the specified index.
    /// 
    /// # Errors
    /// 
    /// Returns [`SanitizeError::IndexOutOfBounds`] if the index is out of bounds.
    pub fn get_account_meta_at(
        &self,
        index: usize,
    ) -> Result<&IntrospectedAccountMeta, SanitizeError> {
        if index >= unsafe { u16::from_le(*(self.raw as *const u16)) } as usize {
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
                .add(size_of::<u16>() + num_accounts as usize * size_of::<IntrospectedAccountMeta>())
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
                        + num_accounts as usize * size_of::<IntrospectedAccountMeta>()
                        + size_of::<Pubkey>(),
                ) as *const u16),
            );

            core::slice::from_raw_parts(
                self.raw.add(
                    size_of::<u16>()
                        + num_accounts as usize * size_of::<IntrospectedAccountMeta>()
                        + size_of::<Pubkey>()
                        + size_of::<u16>(),
                ),
                data_len as usize,
            )
        }
    }
}

/// The bit positions for the signer flags in the `AccountMeta`.
const IS_SIGNER: u8 = 0b00000001;

/// The bit positions for the writable flags in the `AccountMeta`.
const IS_WRITABLE: u8 = 0b00000010;

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedAccountMeta {
    /// Account flags:
    ///   * bit `0`: signer
    ///   * bit `1`: writable
    flags: u8,

    /// The account key.
    pub key: Pubkey,
}

impl IntrospectedAccountMeta {
    const LEN: usize = core::mem::size_of::<Self>();

    /// Indicate whether the account is writable or not.
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        (self.flags & IS_WRITABLE) != 0
    }

    /// Indicate whether the account is a signer or not.
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        (self.flags & IS_SIGNER) != 0
    }

    /// Convert the `IntrospectedAccountMeta` to an `AccountMeta`.
    pub fn to_account_meta(&self) -> AccountMeta {
        AccountMeta::new(&self.key, self.is_writable(), self.is_signer())
    }
}