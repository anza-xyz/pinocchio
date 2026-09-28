use {
    crate::get_extra_account_metas_address,
    alloc::vec::Vec,
    core::{mem::MaybeUninit, ptr::copy_nonoverlapping, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_unchecked, CpiAccount},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
    spl_discriminator::SplDiscriminate,
    spl_list_view::ListView,
    spl_tlv_account_resolution::{
        account::ExtraAccountMeta, error::AccountResolutionError, solana_instruction::AccountMeta,
    },
    spl_type_length_value::state::{TlvState, TlvStateBorrowed},
};

/// TLV instruction type only used to define the discriminator. The actual data
/// is entirely managed by `ExtraAccountMetaList`, and it is the only data
/// contained by this type.
#[derive(SplDiscriminate)]
#[discriminator_hash_input("spl-transfer-hook-interface:execute")]
struct ExecuteInstruction;

/// Runs additional transfer logic.
///
/// Accounts expected by this instruction:
///
///   0. `[]` Source account.
///   1. `[]` Token mint.
///   2. `[]` Destination account.
///   3. `[]` Source account's owner/delegate.
///   4. `[]` (Optional) Validation account.
///   5. ..`5+M` `[]` `M` optional additional accounts, written in validation
///      account data.
pub struct Execute<'account, 'address, 'optional, AdditionalAccount: AsRef<AccountView>>
where
    'optional: 'account,
    'optional: 'address,
{
    /// Program ID of the transfer hook program.
    pub program_id: &'address Address,

    /// Source account.
    pub source: &'account AccountView,

    /// Token mint.
    pub mint: &'account AccountView,

    /// Destination account.
    pub destination: &'account AccountView,

    /// Source account's owner/delegate.
    pub authority: &'account AccountView,

    /// Optional additional accounts, written in validation account data.
    pub additional_accounts: &'optional [AdditionalAccount],

    /// Amount of tokens to transfer.
    ///
    /// When Token-2022 invokes a hook from a confidential transfer, the
    /// amount is not known to the token program and `u64::MAX` is passed
    /// as a convention. A hook that enforces amount-based policy must
    /// handle this sentinel explicitly.
    pub amount: u64,
}

impl<'account, 'address> Execute<'account, 'address, '_, &'account AccountView> {
    /// Creates a new `Execute` instruction.
    #[inline(always)]
    pub fn new(
        program_id: &'address Address,
        source: &'account AccountView,
        mint: &'account AccountView,
        destination: &'account AccountView,
        authority: &'account AccountView,
        amount: u64,
    ) -> Self {
        Self::with_additional_accounts(
            program_id,
            source,
            mint,
            destination,
            authority,
            &[],
            amount,
        )
    }
}

impl<'account, 'address, 'optional, AdditionalAccount: AsRef<AccountView>>
    Execute<'account, 'address, 'optional, AdditionalAccount>
{
    /// Creates a new `Execute` instruction.
    #[inline(always)]
    pub fn with_additional_accounts(
        program_id: &'address Address,
        source: &'account AccountView,
        mint: &'account AccountView,
        destination: &'account AccountView,
        authority: &'account AccountView,
        additional_accounts: &'optional [AdditionalAccount],
        amount: u64,
    ) -> Self {
        Self {
            program_id,
            source,
            mint,
            destination,
            authority,
            additional_accounts,
            amount,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // instruction data
        // - [0..8 ]: instruction discriminator
        // - [8..16]: amount
        let mut instruction_data = [const { MaybeUninit::<u8>::uninit() }; 16];
        // SAFETY: All writes are within bounds of the allocated data.
        unsafe {
            let dst = instruction_data.as_mut_ptr() as *mut u8;

            copy_nonoverlapping(ExecuteInstruction::SPL_DISCRIMINATOR_SLICE.as_ptr(), dst, 8);

            copy_nonoverlapping(
                self.amount.to_le_bytes().as_ptr(),
                dst.add(8),
                size_of::<u64>(),
            );
        }
        // SAFETY: `instruction_data` was initialized.
        let instruction_data = unsafe { from_raw_parts(instruction_data.as_ptr() as _, 16) };

        let validation_address =
            get_extra_account_metas_address(self.mint.address(), self.program_id);

        let mut accounts = Vec::with_capacity(5 + self.additional_accounts.len());
        accounts.push(CpiAccount::from(self.source));
        accounts.push(CpiAccount::from(self.mint));
        accounts.push(CpiAccount::from(self.destination));
        accounts.push(CpiAccount::from(self.authority));

        let mut instruction_accounts = Vec::with_capacity(5 + self.additional_accounts.len());
        instruction_accounts.push(InstructionAccount::readonly(self.source.address()));
        instruction_accounts.push(InstructionAccount::readonly(self.mint.address()));
        instruction_accounts.push(InstructionAccount::readonly(self.destination.address()));
        instruction_accounts.push(InstructionAccount::readonly(self.authority.address()));

        if let Some(validation) = self
            .additional_accounts
            .iter()
            .find(|&x| *x.as_ref().address() == validation_address)
        {
            accounts.push(CpiAccount::from(validation.as_ref()));
            instruction_accounts.push(InstructionAccount::readonly(&validation_address));

            self.resolve_additional_accounts::<ExecuteInstruction>(
                instruction_data,
                &mut instruction_accounts,
                &mut accounts,
                validation.as_ref(),
            )?;
        }

        let instruction = InstructionView {
            program_id: self.program_id,
            accounts: instruction_accounts.as_slice(),
            // SAFETY: `instruction_data` was initialized.
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 16) },
        };

        // SAFETY: `resolve_additional_accounts` validates that any writable account
        // does not have active borrows.
        unsafe { invoke_signed_unchecked(&instruction, accounts.as_slice(), &[]) };

        Ok(())
    }

    /// Add the additional account metas and account infos for a CPI
    fn resolve_additional_accounts<T: SplDiscriminate>(
        &self,
        instruction_data: &[u8],
        instruction_accounts: &mut Vec<InstructionAccount<'address>>,
        cpi_accounts: &mut Vec<CpiAccount<'account>>,
        validation_account: &AccountView,
    ) -> Result<(), ProgramError> {
        let data = validation_account.try_borrow()?;
        let state = TlvStateBorrowed::unpack(&data)?;
        let bytes = state.get_first_bytes::<T>()?;
        let extra_account_metas = ListView::<ExtraAccountMeta>::unpack(bytes)?;

        // Return early if there are no extra account metas to resolve.
        if extra_account_metas.is_empty() {
            return Ok(());
        }

        // The first five accounts are always the source, mint, destination, authority,
        // and the validation account. The rest are additional accounts that are
        // resolved from the validation account data.
        let accounts = [
            self.source,
            self.mint,
            self.destination,
            self.authority,
            validation_account,
        ];
        // Create a list of `Ref`s so we can reference account data in the
        // resolution step. The list is built once and each resolved account
        // is appended to it, since rebuilding it for every meta makes
        // cumulative heap usage quadratic under the never-freeing SBF bump
        // allocator.
        let mut account_key_data_refs =
            Vec::with_capacity(accounts.len() + extra_account_metas.len());

        for account in accounts {
            account_key_data_refs.push((account, Some(account.try_borrow()?)));
        }

        for extra_meta in extra_account_metas.iter() {
            let mut meta = extra_meta.resolve(instruction_data, self.program_id, |usize| {
                account_key_data_refs.get(usize).map(|(account, data)| {
                    (account.address(), data.as_ref().map(|data| data.as_ref()))
                })
            })?;
            de_escalate_account_meta(&mut meta, instruction_accounts);

            let account = self
                .additional_accounts
                .iter()
                .find(|&x| *x.as_ref().address() == meta.pubkey)
                .ok_or(AccountResolutionError::IncorrectAccount)?;

            account_key_data_refs.push((account.as_ref(), Some(account.as_ref().try_borrow()?)));

            instruction_accounts.push(InstructionAccount::new(
                account.as_ref().address(),
                meta.is_writable,
                meta.is_signer,
            ));
            cpi_accounts.push(CpiAccount::from(account.as_ref()));
        }

        // Release every resolution borrow before checking writable accounts since
        // duplicate entries can hold multiple borrows of the same account.
        for (_, data) in &mut account_key_data_refs {
            drop(data.take());
        }

        for ((account, _), meta) in account_key_data_refs
            .iter()
            .zip(instruction_accounts.iter())
        {
            if meta.is_writable {
                account.check_borrow_mut()?;
            }
        }

        Ok(())
    }
}

/// De-escalate an account meta if necessary
fn de_escalate_account_meta(account_meta: &mut AccountMeta, account_metas: &[InstructionAccount]) {
    // This is a little tricky to read, but checks if this account is marked as
    // writable in the instruction. If it's read-only, de-escalate it to read-only
    // in the CPI.
    let maybe_highest_privileges = account_metas
        .iter()
        .filter(|&x| x.address == &account_meta.pubkey)
        .map(|x| x.is_writable)
        .reduce(|acc, x| acc || x);

    // If `Some`, then the account was found somewhere in the instruction
    if let Some(is_writable) = maybe_highest_privileges {
        if !is_writable && is_writable != account_meta.is_writable {
            // Existing account is *NOT* writable already, but the CPI
            // wants it to be, so de-escalate to not be writable
            account_meta.is_writable = false;
        }
    }

    // Always mark an account as a non-signer
    account_meta.is_signer = false;
}
