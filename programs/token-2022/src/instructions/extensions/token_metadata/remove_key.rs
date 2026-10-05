use {
    crate::{
        instructions::{account_borrow_failed_error, invalid_argument_error},
        write_bytes, UNINIT_BYTE,
    },
    core::{marker::PhantomData, mem::MaybeUninit, slice::from_raw_parts},
    pinocchio_token::{
        instructions::{batch::Batch, CpiWriter},
        TokenInterface,
    },
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// The instruction discriminator.
///
/// Hash of `spl_token_metadata_interface:remove_key_ix`.
const DISCRIMINATOR: [u8; 8] = [234, 18, 32, 56, 89, 141, 37, 181];

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 2;

/// Maximum instruction data length.
const MAX_DATA_LEN: usize = 1024;

/// Maximum instruction data length inside a `Batch`, which encodes the
/// instruction data length as a `u8`.
const MAX_BATCH_DATA_LEN: usize = u8::MAX as usize;

/// Removes a key-value pair from the additional metadata of a
/// `TokenMetadata` entry.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The metadata account.
///   1. `[signer]` The update authority.
pub struct RemoveKey<'account, 'data, Program: TokenInterface> {
    /// The metadata account.
    pub metadata: &'account AccountView,

    /// The update authority.
    pub update_authority: &'account AccountView,

    /// If `true`, the instruction does not fail when the key does not
    /// exist.
    pub idempotent: bool,

    /// The key to remove.
    pub key: &'data str,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

impl<'account, 'data, Program: TokenInterface> RemoveKey<'account, 'data, Program> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: [u8; 8] = DISCRIMINATOR;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = ACCOUNTS_LEN;

    /// Maximum instruction data length.
    pub const MAX_DATA_LEN: usize = MAX_DATA_LEN;

    #[inline(always)]
    pub fn new(
        metadata: &'account AccountView,
        update_authority: &'account AccountView,
        idempotent: bool,
        key: &'data str,
    ) -> Self {
        Self {
            metadata,
            update_authority,
            idempotent,
            key,
            _program: PhantomData,
        }
    }

    /// Invokes the instruction with `Program::ID`.
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_with_unverified_program(&Program::ID)
    }

    /// Invokes the instruction with `Program::ID` and signer seeds.
    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        self.invoke_signed_with_unverified_program(signers, &Program::ID)
    }

    /// Invokes the instruction after verifying the `program` address.
    #[inline(always)]
    pub fn invoke_with_program(&self, program: &Address) -> ProgramResult {
        self.invoke_signed_with_program(&[], program)
    }

    /// Invokes the instruction with signer seeds after verifying the `program`
    /// address.
    #[inline(always)]
    pub fn invoke_signed_with_program(
        &self,
        signers: &[Signer],
        program: &Address,
    ) -> ProgramResult {
        Program::verify(program)?;
        self.invoke_signed_with_unverified_program(signers, program)
    }

    /// Invokes the instruction with `program` without verifying it.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_with_program`.
    ///
    /// # Important
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenInterface::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub fn invoke_with_unverified_program(&self, program: &Address) -> ProgramResult {
        self.invoke_signed_with_unverified_program(&[], program)
    }

    /// Invokes the instruction with signer seeds and `program` without
    /// verifying the program address.
    ///
    /// Use this when `program` has already been verified. Otherwise, prefer
    /// `invoke_signed_with_program`.
    ///
    /// # Important
    ///
    /// This method does not verify that `program` satisfies
    /// [`TokenInterface::verify`]. The caller must ensure the program address
    /// has already been checked and corresponds to the expected
    /// token program.
    #[inline(always)]
    pub fn invoke_signed_with_unverified_program(
        &self,
        signers: &[Signer],
        program: &Address,
    ) -> ProgramResult {
        let mut instruction_accounts = [const { MaybeUninit::uninit() }; ACCOUNTS_LEN];
        let written_instruction_accounts =
            self.write_instruction_accounts(&mut instruction_accounts)?;

        let mut accounts = [const { MaybeUninit::uninit() }; ACCOUNTS_LEN];
        let written_accounts = self.write_accounts(&mut accounts)?;

        let mut instruction_data = [UNINIT_BYTE; MAX_DATA_LEN];
        let written_instruction_data = self.write_instruction_data(&mut instruction_data)?;

        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: program,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: from_raw_parts(instruction_data.as_ptr() as _, written_instruction_data),
                },
                from_raw_parts(accounts.as_ptr() as _, written_accounts),
                signers,
            );
        }

        Ok(())
    }
}

impl<Program: TokenInterface> CpiWriter for RemoveKey<'_, '_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.metadata, self.update_authority, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.metadata, self.update_authority, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.idempotent, self.key, data)
    }
}

impl<Program: TokenInterface> crate::instructions::IntoBatch<Program>
    for RemoveKey<'_, '_, Program>
{
    #[inline(always)]
    fn into_batch<'account, 'state>(
        self,
        batch: &mut Batch<'account, 'state, Program>,
    ) -> ProgramResult
    where
        Self: 'account + 'state,
    {
        batch.push(
            |accounts| write_accounts(self.metadata, self.update_authority, accounts),
            |accounts| write_instruction_accounts(self.metadata, self.update_authority, accounts),
            |data| {
                let len = data.len().min(MAX_BATCH_DATA_LEN);
                write_instruction_data(self.idempotent, self.key, &mut data[..len])
            },
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    metadata: &'account AccountView,
    update_authority: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if metadata.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(metadata, &mut accounts[0]);

    CpiAccount::init_from_account_view(update_authority, &mut accounts[1]);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    metadata: &'account AccountView,
    update_authority: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(metadata.address()));

    accounts[1].write(InstructionAccount::readonly_signer(
        update_authority.address(),
    ));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    idempotent: bool,
    key: &str,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    let expected_data_len = 8 + 1 + 4 + key.len();

    if expected_data_len > MAX_DATA_LEN || data.len() < expected_data_len {
        return Err(invalid_argument_error());
    }

    write_bytes(&mut data[0..8], &DISCRIMINATOR);

    data[8].write(idempotent as u8);

    write_bytes(&mut data[9..13], &(key.len() as u32).to_le_bytes());

    write_bytes(&mut data[13..expected_data_len], key.as_bytes());

    Ok(expected_data_len)
}
