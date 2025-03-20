use crate::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey
};

// Needs replacement
use solana_sanitize::SanitizeError;


const INSTRUCTIONS_ID: Pubkey = [
    6, 167, 213, 23, 24, 123, 209, 102, 53, 218, 212, 4, 85, 253, 194, 192, 193, 36, 198, 143, 33,
    86, 117, 165, 219, 186, 203, 95, 8, 0, 0, 0,
];

pub enum InstructionSysvarError {
    InvalidAccountData,
    InvalidAccountId,
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct Instructions<'a> {
    pub(crate) account_info: &'a AccountInfo,
}


impl<'a> From<&'a AccountInfo> for Instructions<'a> {
    fn from(account_info: &'a AccountInfo) -> Self {
        Self { account_info }
    }
}

impl<'a> Instructions<'a> {
    pub fn new(account_info: &'a AccountInfo) -> Result<Self, ProgramError> {
        let sysvar = Self::new_unchecked(account_info);
        if !sysvar.check_id() {
            return Err(ProgramError::Custom(
                InstructionSysvarError::InvalidAccountId as u32,
            ));
        }
        Ok(sysvar)
    }
    pub fn new_unchecked(account_info: &'a AccountInfo) -> Self {
        Self { account_info }
    }
    pub fn check_id(&self) -> bool {
        self.account_info.key() == &INSTRUCTIONS_ID
    }
    pub fn get_instruction_count(&self) -> Result<usize, ProgramError> {
        let mut current = 0;
        let data = self.account_info.try_borrow_data()?;
        let num_instructions = read_u16(&mut current, &data)
            .map_err(|_| ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32))?;
        Ok(num_instructions as usize)
    }
    pub fn load_instruction_at_checked(
        self,
        index: usize,
    ) -> Result<IntrospectedInstruction, ProgramError> {
        // We need to make calculations based on the data, but we don't need to keep
        // the Ref alive after this function returns
        unsafe {
            let data_ref = self.account_info.try_borrow_data()?;
            let data_ptr = data_ref.as_ptr();

            let mut current = 0;

            // Get number of instructions
            let num_instructions = read_u16(&mut current, &data_ref).map_err(|_| {
                ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
            })?;

            if index >= num_instructions as usize {
                return Err(ProgramError::Custom(
                    InstructionSysvarError::InvalidAccountData as u32,
                ));
            }

            // Calculate offset to this instruction's location
            current += index * 2;
            let instruction_start = read_u16(&mut current, &data_ref).map_err(|_| {
                ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
            })?;

            // Move to the start of the instruction
            current = instruction_start as usize;

            // Read the number of accounts
            let num_accounts = read_u16(&mut current, &data_ref).map_err(|_| {
                ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
            })?;

            // Calculate important offsets
            let program_id_offset = current + (num_accounts as usize * 33);
            let ix_data_offset = program_id_offset + core::mem::size_of::<Pubkey>();

            // Read instruction data length
            let mut data_len_pos = ix_data_offset;
            let ix_data_len = read_u16(&mut data_len_pos, &data_ref).map_err(|_| {
                ProgramError::Custom(InstructionSysvarError::InvalidAccountData as u32)
            })?;

            // Calculate total instruction length
            let total_len = ix_data_offset + 2 + ix_data_len as usize;

            // Create the IntrospectedInstruction with raw pointer and metadata
            Ok(IntrospectedInstruction {
                data_ptr: data_ptr.add(instruction_start as usize),
                data_len: total_len - instruction_start as usize,
                num_accounts,
                // Offset is relative to the start of the instruction
                program_id_offset: program_id_offset - instruction_start as usize,
                // Offset is relative to the start of the instruction
                ix_data_offset: ix_data_offset + 2 - instruction_start as usize, // +2 to skip the length field
                ix_data_len,
            })
        }
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedInstruction {
    pub data_ptr: *const u8,      // Pointer to the start of instruction data
    pub data_len: usize,          // Length of the entire instruction data
    pub num_accounts: u16,        // Number of accounts in this instruction
    pub program_id_offset: usize, // Offset to the program ID
    pub ix_data_offset: usize,    // Offset to the instruction-specific data
    pub ix_data_len: u16,         // Length of instruction-specific data
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct IntrospectedAccountMeta {
    pub meta_byte: u8,
    pub pubkey: Pubkey,
}

// Define these constants at the top of your file
const IS_SIGNER_BIT: u8 = 0; // Assuming bit 1 for signer flag
const IS_WRITABLE_BIT: u8 = 1; // Assuming bit 0 for writable flag

impl IntrospectedAccountMeta {
    pub fn is_writable(&self) -> bool {
        (self.meta_byte & (1 << IS_WRITABLE_BIT)) != 0
    }

    pub fn is_signer(&self) -> bool {
        (self.meta_byte & (1 << IS_SIGNER_BIT)) != 0
    }

    pub fn key(&self) -> &Pubkey {
        &self.pubkey
    }

    pub fn to_account_meta(&self) -> AccountMeta {
        AccountMeta::new(self.key(), self.is_signer(), self.is_writable())
    }
}

impl IntrospectedInstruction {
    pub fn get_program_id(&self) -> &Pubkey {
        unsafe {
            let program_id_ptr = self.data_ptr.add(self.program_id_offset) as *const Pubkey;
            &*program_id_ptr
        }
    }
    pub fn get_instruction_data(&self) -> &[u8] {
        unsafe {
            let instruction_data_ptr = self.data_ptr.add(self.ix_data_offset) as *const u8;
            core::slice::from_raw_parts(instruction_data_ptr, self.ix_data_len as usize)
        }
    }
    pub fn get_account_meta_at(
        &self,
        index: usize,
    ) -> Result<&IntrospectedAccountMeta, SanitizeError> {
        if index >= self.num_accounts as usize {
            return Err(SanitizeError::IndexOutOfBounds);
        }
        Ok(self.get_account_meta_at_unchecked(index))
    }
    pub fn get_account_meta_at_unchecked(&self, index: usize) -> &IntrospectedAccountMeta {
        unsafe {
            let account_meta_ptr = self
                .data_ptr
                .add(2 + index * core::mem::size_of::<IntrospectedAccountMeta>() as usize)
                as *const IntrospectedAccountMeta;
            &*account_meta_ptr
        }
    }

    pub fn get_account_metas(&self) -> &[IntrospectedAccountMeta] {
        unsafe {
            let account_metas_ptr = self.data_ptr.add(2) as *const IntrospectedAccountMeta;
            core::slice::from_raw_parts(account_metas_ptr, self.num_accounts as usize)
        }
    }
    pub fn to_instruction<'s, 'a, 'b>(
        &'s self,
        account_meta_buffer: &'b mut [AccountMeta<'a>],
    ) -> Result<Instruction<'a, 'b, 's, 's>, SanitizeError>
    where
        'a: 'b,
        's: 'a,
    {
        let metas = self.get_account_metas();
        if account_meta_buffer.len() < metas.len() {
            return Err(SanitizeError::IndexOutOfBounds);
        }

        // Fill the buffer with account metas
        for (i, meta) in metas.iter().enumerate() {
            account_meta_buffer[i] = meta.to_account_meta();
        }
        Ok(Instruction {
            program_id: self.get_program_id(),
            accounts: &account_meta_buffer[..metas.len()],
            data: self.get_instruction_data(),
        })
    }
}

// Copy of solana_serialize_utils::read_u16
/// Read a 16-bit unsigned integer (little-endian) from the buffer and advance the current position
pub fn read_u16(current: &mut usize, data: &[u8]) -> Result<u16, SanitizeError> {
    if data.len() < *current + 2 {
        return Err(SanitizeError::IndexOutOfBounds);
    }
    let value = unsafe {
        let ptr = data.as_ptr().add(*current) as *const u16;
        u16::from_le(ptr.read_unaligned())
    };
    *current += 2;
    Ok(value)
}