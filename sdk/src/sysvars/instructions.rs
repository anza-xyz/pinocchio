#[cfg(feature = "cpi")]
use crate::instruction::InstructionAccount;
use {
    crate::{
        account::{AccountView, Ref},
        address::ADDRESS_BYTES,
        error::ProgramError,
        Address,
    },
    core::{marker::PhantomData, mem::size_of, ops::Deref},
};

/// Instructions sysvar ID `Sysvar1nstructions1111111111111111111111111`.
pub const INSTRUCTIONS_ID: Address = Address::new_from_array([
    0x06, 0xa7, 0xd5, 0x17, 0x18, 0x7b, 0xd1, 0x66, 0x35, 0xda, 0xd4, 0x04, 0x55, 0xfd, 0xc2, 0xc0,
    0xc1, 0x24, 0xc6, 0x8f, 0x21, 0x56, 0x75, 0xa5, 0xdb, 0xba, 0xcb, 0x5f, 0x08, 0x00, 0x00, 0x00,
]);

#[derive(Clone, Debug)]
pub struct Instructions<T>
where
    T: Deref<Target = [u8]>,
{
    data: T,
}

impl<T> Instructions<T>
where
    T: Deref<Target = [u8]>,
{
    /// Creates a new `Instructions` struct.
    ///
    /// `data` is the instructions sysvar account data.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the provided data
    /// is from the Sysvar Account.
    #[inline(always)]
    pub unsafe fn new_unchecked(data: T) -> Self {
        Instructions { data }
    }

    /// Load the number of instructions in the currently executing
    /// `Transaction`.
    #[inline(always)]
    pub fn num_instructions(&self) -> usize {
        // SAFETY: The first 2 bytes of the Instructions sysvar data represents the
        // number of instructions.
        u16::from_le_bytes(unsafe { *(self.data.as_ptr() as *const [u8; 2]) }) as usize
    }

    /// Load the current `Instruction`'s index in the currently executing
    /// `Transaction`.
    #[inline(always)]
    pub fn load_current_index(&self) -> u16 {
        let len = self.data.len();
        // SAFETY: The last 2 bytes of the Instructions sysvar data represents the
        // current instruction index.
        unsafe { u16::from_le_bytes(*(self.data.as_ptr().add(len - 2) as *const [u8; 2])) }
    }

    /// Creates and returns an `IntrospectedInstruction` for the instruction at
    /// the specified index.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the provided index
    /// is out of bounds. It is typically used internally with the
    /// `load_instruction_at` or `get_instruction_relative` functions, which
    /// perform the necessary index verification.
    #[inline(always)]
    pub unsafe fn deserialize_instruction_unchecked(
        &self,
        index: usize,
    ) -> IntrospectedInstruction {
        let offset = *(self
            .data
            .as_ptr()
            .add(size_of::<u16>() + index * size_of::<u16>()) as *const u16);

        IntrospectedInstruction::new_unchecked(self.data.as_ptr().add(offset as usize))
    }

    /// Creates and returns an `IntrospectedInstruction` for the instruction at
    /// the specified index.
    #[inline(always)]
    pub fn load_instruction_at(
        &self,
        index: usize,
    ) -> Result<IntrospectedInstruction, ProgramError> {
        if index >= self.num_instructions() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // SAFETY: The index was checked to be in bounds.
        Ok(unsafe { self.deserialize_instruction_unchecked(index) })
    }

    /// Creates and returns an `IntrospectedInstruction` relative to the current
    /// `Instruction` in the currently executing `Transaction.
    #[inline(always)]
    pub fn get_instruction_relative(
        &self,
        index_relative_to_current: i64,
    ) -> Result<IntrospectedInstruction, ProgramError> {
        let current_index = self.load_current_index() as i64;
        let index = current_index.saturating_add(index_relative_to_current);

        if index < 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        self.load_instruction_at(index as usize)
    }
}

impl<'a> TryFrom<&'a AccountView> for Instructions<Ref<'a, [u8]>> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(account_view: &'a AccountView) -> Result<Self, Self::Error> {
        if account_view.address() != &INSTRUCTIONS_ID {
            return Err(ProgramError::UnsupportedSysvar);
        }

        Ok(Instructions {
            data: account_view.try_borrow()?,
        })
    }
}

#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectedInstruction<'a> {
    pub raw: *const u8,
    marker: PhantomData<&'a [u8]>,
}

impl IntrospectedInstruction<'_> {
    /// Create a new `IntrospectedInstruction`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not verify anything about the
    /// pointer.
    ///
    /// It is private and used internally within the
    /// `get_instruction_account_at` function, which performs the necessary
    /// index verification. However, to optimize performance for users
    /// who are sure that the index is in bounds, we have exposed it as an
    /// unsafe function.
    #[inline(always)]
    unsafe fn new_unchecked(raw: *const u8) -> Self {
        Self {
            raw,
            marker: PhantomData,
        }
    }

    /// Get the number of accounts of the `Instruction`.
    #[inline(always)]
    pub fn num_account_metas(&self) -> usize {
        // SAFETY: The first 2 bytes represent the number of accounts in the
        // instruction.
        u16::from_le_bytes(unsafe { *(self.raw as *const [u8; 2]) }) as usize
    }

    /// Get the instruction account at the specified index.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not verify if the index is out
    /// of bounds.
    ///
    /// It is typically used internally within the `get_instruction_account_at`
    /// function, which performs the necessary index verification. However,
    /// to optimize performance for users who are sure that the index is in
    /// bounds, we have exposed it as an unsafe function.
    #[inline(always)]
    pub unsafe fn get_instruction_account_at_unchecked(
        &self,
        index: usize,
    ) -> &IntrospectedInstructionAccount {
        let offset = core::mem::size_of::<u16>() + (index * IntrospectedInstructionAccount::LEN);
        &*(self.raw.add(offset) as *const IntrospectedInstructionAccount)
    }

    /// Get the instruction account at the specified index.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError::InvalidArgument`] if the index is out of bounds.
    #[inline(always)]
    pub fn get_instruction_account_at(
        &self,
        index: usize,
    ) -> Result<&IntrospectedInstructionAccount, ProgramError> {
        // SAFETY: The first 2 bytes represent the number of accounts in the
        // instruction.
        let num_accounts = self.num_account_metas();

        if index >= num_accounts {
            return Err(ProgramError::InvalidArgument);
        }

        // SAFETY: The index was checked to be in bounds.
        Ok(unsafe { self.get_instruction_account_at_unchecked(index) })
    }

    /// Get the program ID of the `Instruction`.
    #[inline(always)]
    pub fn get_program_id(&self) -> &Address {
        // SAFETY: The first 2 bytes represent the number of accounts in the
        // instruction.
        let num_accounts = self.num_account_metas();

        // SAFETY: The program ID is located after the instruction accounts.
        unsafe {
            &*(self
                .raw
                .add(size_of::<u16>() + num_accounts * size_of::<IntrospectedInstructionAccount>())
                as *const Address)
        }
    }

    /// Get the instruction data of the `Instruction`.
    #[inline(always)]
    pub fn get_instruction_data(&self) -> &[u8] {
        // SAFETY: The first 2 bytes represent the number of accounts in the
        // instruction.
        let offset =
            self.num_account_metas() * size_of::<IntrospectedInstructionAccount>() + ADDRESS_BYTES;

        // SAFETY: The instruction data length is located after the program ID.
        let data_len = u16::from_le_bytes(unsafe {
            *(self.raw.add(size_of::<u16>() + offset) as *const [u8; 2])
        });

        // SAFETY: The instruction data is located after the data length.
        unsafe {
            core::slice::from_raw_parts(
                self.raw.add(size_of::<u16>() + offset + size_of::<u16>()),
                data_len as usize,
            )
        }
    }
}

/// The bit positions for the signer flags in the `InstructionAccount`.
const IS_SIGNER: u8 = 0b00000001;

/// The bit positions for the writable flags in the `InstructionAccount`.
const IS_WRITABLE: u8 = 0b00000010;

#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectedInstructionAccount {
    /// Account flags:
    ///   * bit `0`: signer
    ///   * bit `1`: writable
    flags: u8,

    /// The account key.
    pub key: Address,
}

impl IntrospectedInstructionAccount {
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

    #[cfg(feature = "cpi")]
    /// Convert the `IntrospectedInstructionAccount` to an `InstructionAccount`.
    #[inline(always)]
    pub fn to_instruction_account(&self) -> InstructionAccount {
        InstructionAccount::new(&self.key, self.is_writable(), self.is_signer())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    /// Build a single instruction blob (num_accounts, account metas, program_id, data).
    fn build_instruction_blob(
        num_account_metas: u16,
        account_flags_and_keys: &[(u8, Address)],
        program_id: &Address,
        data: &[u8],
    ) -> Vec<u8> {
        assert_eq!(
            num_account_metas as usize,
            account_flags_and_keys.len(),
            "account_flags_and_keys length must match num_account_metas"
        );
        let mut out = Vec::new();
        out.extend_from_slice(&num_account_metas.to_le_bytes());
        for (flags, key) in account_flags_and_keys {
            out.push(*flags);
            out.extend_from_slice(key.as_ref());
        }
        out.extend_from_slice(program_id.as_ref());
        out.extend_from_slice(&(data.len() as u16).to_le_bytes());
        out.extend_from_slice(data);
        out
    }

    /// Build the full Instructions sysvar buffer: header (num_instructions, offsets, current_index)
    /// and instruction blobs at the given offsets.
    fn build_instructions_sysvar_buffer(
        instruction_blobs: &[Vec<u8>],
        current_index: u16,
    ) -> Vec<u8> {
        let num_instructions = instruction_blobs.len() as u16;
        let header_len = 2 + 2 * num_instructions as usize;
        let mut offsets = Vec::with_capacity(instruction_blobs.len());
        let mut offset = header_len;
        for blob in instruction_blobs {
            offsets.push(offset as u16);
            offset += blob.len();
        }
        let current_index_offset = offset;
        let total_len = current_index_offset + 2;

        let mut buffer = vec![0u8; total_len];
        buffer[0..2].copy_from_slice(&num_instructions.to_le_bytes());
        for (i, &off) in offsets.iter().enumerate() {
            buffer[2 + i * 2..4 + i * 2].copy_from_slice(&off.to_le_bytes());
        }
        let mut write_offset = header_len;
        for blob in instruction_blobs {
            buffer[write_offset..write_offset + blob.len()].copy_from_slice(blob);
            write_offset += blob.len();
        }
        buffer[current_index_offset..total_len].copy_from_slice(&current_index.to_le_bytes());
        buffer
    }

    fn make_address(byte: u8) -> Address {
        Address::new_from_array([byte; 32])
    }

    #[test]
    fn test_num_instructions_and_load_current_index() {
        let program_id = make_address(1);
        let blob = build_instruction_blob(0, &[], &program_id, b"data");
        let buffer = build_instructions_sysvar_buffer(&[blob], 0);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };

        assert_eq!(instructions.num_instructions(), 1);
        assert_eq!(instructions.load_current_index(), 0);
    }

    #[test]
    fn test_load_current_index_last_instruction() {
        let program_id = make_address(2);
        let blob = build_instruction_blob(0, &[], &program_id, b"");
        let buffer = build_instructions_sysvar_buffer(&[blob.clone(), blob], 1);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };

        assert_eq!(instructions.num_instructions(), 2);
        assert_eq!(instructions.load_current_index(), 1);
    }

    #[test]
    fn test_load_instruction_at_in_bounds() {
        let program_id = make_address(10);
        let data = b"hello";
        let blob = build_instruction_blob(0, &[], &program_id, data);
        let buffer = build_instructions_sysvar_buffer(&[blob], 0);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };

        let ix = instructions.load_instruction_at(0).expect("in bounds");
        assert_eq!(ix.num_account_metas(), 0);
        assert_eq!(ix.get_program_id(), &program_id);
        assert_eq!(ix.get_instruction_data(), data);
    }

    #[test]
    fn test_load_instruction_at_out_of_bounds() {
        let program_id = make_address(0);
        let blob = build_instruction_blob(0, &[], &program_id, b"");
        let buffer = build_instructions_sysvar_buffer(&[blob], 0);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };

        assert!(instructions.load_instruction_at(1).is_err());
        assert!(instructions.load_instruction_at(99).is_err());
    }

    #[test]
    fn test_get_instruction_relative() {
        let program_id0 = make_address(1);
        let program_id1 = make_address(2);
        let blob0 = build_instruction_blob(0, &[], &program_id0, b"zero");
        let blob1 = build_instruction_blob(0, &[], &program_id1, b"one");
        let buffer = build_instructions_sysvar_buffer(&[blob0, blob1], 1);

        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };

        let current = instructions.get_instruction_relative(0).expect("current");
        assert_eq!(current.get_program_id(), &program_id1);
        assert_eq!(current.get_instruction_data(), b"one");

        let prev = instructions.get_instruction_relative(-1).expect("previous");
        assert_eq!(prev.get_program_id(), &program_id0);
        assert_eq!(prev.get_instruction_data(), b"zero");

        let next = instructions.get_instruction_relative(1);
        assert!(next.is_err());
    }

    #[test]
    fn test_get_instruction_relative_negative_invalid() {
        let program_id = make_address(0);
        let blob = build_instruction_blob(0, &[], &program_id, b"");
        let buffer = build_instructions_sysvar_buffer(&[blob], 0);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };

        assert!(instructions.get_instruction_relative(-1).is_err());
    }

    #[test]
    fn test_instruction_with_account_metas() {
        let program_id = make_address(100);
        let signer_writable = 0b0000_0011u8;
        let key1 = make_address(201);
        let key2 = make_address(202);
        let blob = build_instruction_blob(
            2,
            &[
                (signer_writable, key1.clone()),
                (0b0000_0010, key2.clone()), // writable only
            ],
            &program_id,
            b"ixdata",
        );
        let buffer = build_instructions_sysvar_buffer(&[blob], 0);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };
        let ix = instructions.load_instruction_at(0).expect("in bounds");

        assert_eq!(ix.num_account_metas(), 2);

        let acc0 = ix.get_instruction_account_at(0).expect("account 0");
        assert!(acc0.is_signer());
        assert!(acc0.is_writable());
        assert_eq!(acc0.key, key1);

        let acc1 = ix.get_instruction_account_at(1).expect("account 1");
        assert!(!acc1.is_signer());
        assert!(acc1.is_writable());
        assert_eq!(acc1.key, key2);

        assert!(ix.get_instruction_account_at(2).is_err());
    }

    #[test]
    fn test_introspected_instruction_data_empty() {
        let program_id = make_address(0);
        let blob = build_instruction_blob(0, &[], &program_id, b"");
        let buffer = build_instructions_sysvar_buffer(&[blob], 0);
        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };
        let ix = instructions.load_instruction_at(0).expect("in bounds");

        assert_eq!(ix.get_instruction_data(), b"");
    }

    #[test]
    fn test_multiple_instructions_offsets() {
        let p0 = make_address(0);
        let p1 = make_address(1);
        let p2 = make_address(2);
        let b0 = build_instruction_blob(0, &[], &p0, b"a");
        let b1 = build_instruction_blob(0, &[], &p1, b"bb");
        let b2 = build_instruction_blob(0, &[], &p2, b"ccc");
        let buffer = build_instructions_sysvar_buffer(&[b0, b1, b2], 1);

        let instructions = unsafe { Instructions::new_unchecked(buffer.as_slice()) };
        assert_eq!(instructions.num_instructions(), 3);

        let ix0 = instructions.load_instruction_at(0).expect("0");
        let ix1 = instructions.load_instruction_at(1).expect("1");
        let ix2 = instructions.load_instruction_at(2).expect("2");

        assert_eq!(ix0.get_program_id(), &p0);
        assert_eq!(ix0.get_instruction_data(), b"a");
        assert_eq!(ix1.get_program_id(), &p1);
        assert_eq!(ix1.get_instruction_data(), b"bb");
        assert_eq!(ix2.get_program_id(), &p2);
        assert_eq!(ix2.get_instruction_data(), b"ccc");
    }
}
