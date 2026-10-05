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
/// Hash of `spl_token_group_interface:initialize_token_group`.
const DISCRIMINATOR: [u8; 8] = [121, 113, 108, 39, 54, 51, 0, 4];

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 3;

/// Instruction data length:
///   - discriminator (8 bytes)
///   - update authority (32 bytes)
///   - max size (8 bytes)
const DATA_LEN: usize = 48;

/// Initializes a new `TokenGroup` in the provided account.
///
/// Token-2022 only supports storing the group in the mint itself, so the
/// group account must be the mint. The mint must have the `GroupPointer`
/// extension pointing to itself and enough lamports to cover the rent of
/// the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[]` The mint.
///   2. `[signer]` The mint authority.
pub struct InitializeGroup<'account, 'address, Program: TokenInterface> {
    /// The group account.
    pub group: &'account AccountView,

    /// The mint.
    pub mint: &'account AccountView,

    /// The mint authority.
    pub mint_authority: &'account AccountView,

    /// The authority that can sign to update the group.
    pub update_authority: Option<&'address Address>,

    /// The maximum number of group members.
    pub max_size: u64,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

impl<'account, 'address, Program: TokenInterface> InitializeGroup<'account, 'address, Program> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: [u8; 8] = DISCRIMINATOR;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = ACCOUNTS_LEN;

    /// Instruction data length.
    pub const DATA_LEN: usize = DATA_LEN;

    #[inline(always)]
    pub fn new(
        group: &'account AccountView,
        mint: &'account AccountView,
        mint_authority: &'account AccountView,
        update_authority: Option<&'address Address>,
        max_size: u64,
    ) -> Self {
        Self {
            group,
            mint,
            mint_authority,
            update_authority,
            max_size,
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

        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];
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

impl<Program: TokenInterface> CpiWriter for InitializeGroup<'_, '_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(self.group, self.mint, self.mint_authority, accounts)
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(self.group, self.mint, self.mint_authority, accounts)
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(self.update_authority, self.max_size, data)
    }
}

impl<Program: TokenInterface> crate::instructions::IntoBatch<Program>
    for InitializeGroup<'_, '_, Program>
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
            |accounts| write_accounts(self.group, self.mint, self.mint_authority, accounts),
            |accounts| {
                write_instruction_accounts(self.group, self.mint, self.mint_authority, accounts)
            },
            |data| write_instruction_data(self.update_authority, self.max_size, data),
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    group: &'account AccountView,
    mint: &'account AccountView,
    mint_authority: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if group.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(group, &mut accounts[0]);

    CpiAccount::init_from_account_view(mint, &mut accounts[1]);

    CpiAccount::init_from_account_view(mint_authority, &mut accounts[2]);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    group: &'account AccountView,
    mint: &'account AccountView,
    mint_authority: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(group.address()));

    accounts[1].write(InstructionAccount::readonly(mint.address()));

    accounts[2].write(InstructionAccount::readonly_signer(
        mint_authority.address(),
    ));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(
    update_authority: Option<&Address>,
    max_size: u64,
    data: &mut [MaybeUninit<u8>],
) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    write_bytes(&mut data[0..8], &DISCRIMINATOR);

    write_bytes(
        &mut data[8..40],
        if let Some(update_authority) = update_authority {
            update_authority.as_ref()
        } else {
            &[0u8; 32]
        },
    );

    write_bytes(&mut data[40..DATA_LEN], &max_size.to_le_bytes());

    Ok(DATA_LEN)
}
