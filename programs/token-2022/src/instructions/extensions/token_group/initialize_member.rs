use {
    crate::{
        instructions::{account_borrow_failed_error, invalid_argument_error},
        write_bytes,
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
/// Hash of `spl_token_group_interface:initialize_member`.
const DISCRIMINATOR: [u8; 8] = [152, 32, 222, 176, 223, 237, 116, 134];

/// Expected number of accounts.
const ACCOUNTS_LEN: usize = 5;

/// Instruction data length:
///   - discriminator (8 bytes)
const DATA_LEN: usize = 8;

/// Initializes a new `TokenGroupMember` in the provided account and adds
/// it to the group, incrementing the group size.
///
/// Token-2022 only supports storing the member in the member mint itself,
/// so the member account must be the member mint. The member mint must
/// have the `GroupMemberPointer` extension pointing to itself and enough
/// lamports to cover the rent of the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The member account.
///   1. `[]` The member mint.
///   2. `[signer]` The member mint authority.
///   3. `[writable]` The group account.
///   4. `[signer]` The group update authority.
pub struct InitializeMember<'account, Program: TokenInterface> {
    /// The member account.
    pub member: &'account AccountView,

    /// The member mint.
    pub member_mint: &'account AccountView,

    /// The member mint authority.
    pub member_mint_authority: &'account AccountView,

    /// The group account.
    pub group: &'account AccountView,

    /// The group update authority.
    pub group_update_authority: &'account AccountView,

    /// Phantom data for the program.
    _program: PhantomData<Program>,
}

impl<'account, Program: TokenInterface> InitializeMember<'account, Program> {
    /// The instruction discriminator.
    pub const DISCRIMINATOR: [u8; 8] = DISCRIMINATOR;

    /// Expected number of accounts.
    pub const ACCOUNTS_LEN: usize = ACCOUNTS_LEN;

    /// Instruction data length.
    pub const DATA_LEN: usize = DATA_LEN;

    #[inline(always)]
    pub fn new(
        member: &'account AccountView,
        member_mint: &'account AccountView,
        member_mint_authority: &'account AccountView,
        group: &'account AccountView,
        group_update_authority: &'account AccountView,
    ) -> Self {
        Self {
            member,
            member_mint,
            member_mint_authority,
            group,
            group_update_authority,
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

        unsafe {
            invoke_signed_unchecked(
                &InstructionView {
                    program_id: program,
                    accounts: from_raw_parts(
                        instruction_accounts.as_ptr() as _,
                        written_instruction_accounts,
                    ),
                    data: &DISCRIMINATOR,
                },
                from_raw_parts(accounts.as_ptr() as _, written_accounts),
                signers,
            );
        }

        Ok(())
    }
}

impl<Program: TokenInterface> CpiWriter for InitializeMember<'_, Program> {
    #[inline(always)]
    fn write_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_accounts(
            self.member,
            self.member_mint,
            self.member_mint_authority,
            self.group,
            self.group_update_authority,
            accounts,
        )
    }

    #[inline(always)]
    fn write_instruction_accounts<'cpi>(
        &self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi,
    {
        write_instruction_accounts(
            self.member,
            self.member_mint,
            self.member_mint_authority,
            self.group,
            self.group_update_authority,
            accounts,
        )
    }

    #[inline(always)]
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
        write_instruction_data(data)
    }
}

impl<Program: TokenInterface> crate::instructions::IntoBatch<Program>
    for InitializeMember<'_, Program>
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
            |accounts| {
                write_accounts(
                    self.member,
                    self.member_mint,
                    self.member_mint_authority,
                    self.group,
                    self.group_update_authority,
                    accounts,
                )
            },
            |accounts| {
                write_instruction_accounts(
                    self.member,
                    self.member_mint,
                    self.member_mint_authority,
                    self.group,
                    self.group_update_authority,
                    accounts,
                )
            },
            write_instruction_data,
        )
    }
}

#[inline(always)]
fn write_accounts<'account, 'out>(
    member: &'account AccountView,
    member_mint: &'account AccountView,
    member_mint_authority: &'account AccountView,
    group: &'account AccountView,
    group_update_authority: &'account AccountView,
    accounts: &mut [MaybeUninit<CpiAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    if member.is_borrowed() | group.is_borrowed() {
        return Err(account_borrow_failed_error());
    }

    CpiAccount::init_from_account_view(member, &mut accounts[0]);

    CpiAccount::init_from_account_view(member_mint, &mut accounts[1]);

    CpiAccount::init_from_account_view(member_mint_authority, &mut accounts[2]);

    CpiAccount::init_from_account_view(group, &mut accounts[3]);

    CpiAccount::init_from_account_view(group_update_authority, &mut accounts[4]);

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_accounts<'account, 'out>(
    member: &'account AccountView,
    member_mint: &'account AccountView,
    member_mint_authority: &'account AccountView,
    group: &'account AccountView,
    group_update_authority: &'account AccountView,
    accounts: &mut [MaybeUninit<InstructionAccount<'out>>],
) -> Result<usize, ProgramError>
where
    'account: 'out,
{
    if accounts.len() < ACCOUNTS_LEN {
        return Err(invalid_argument_error());
    }

    accounts[0].write(InstructionAccount::writable(member.address()));

    accounts[1].write(InstructionAccount::readonly(member_mint.address()));

    accounts[2].write(InstructionAccount::readonly_signer(
        member_mint_authority.address(),
    ));

    accounts[3].write(InstructionAccount::writable(group.address()));

    accounts[4].write(InstructionAccount::readonly_signer(
        group_update_authority.address(),
    ));

    Ok(ACCOUNTS_LEN)
}

#[inline(always)]
fn write_instruction_data(data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError> {
    if data.len() < DATA_LEN {
        return Err(invalid_argument_error());
    }

    write_bytes(&mut data[..DATA_LEN], &DISCRIMINATOR);

    Ok(DATA_LEN)
}
