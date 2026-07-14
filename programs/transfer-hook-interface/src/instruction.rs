use {
    crate::EXECUTE_DISCRIMINATOR,
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    let len = destination.len().min(source.len());
    // SAFETY: Both pointers have alignment 1. For valid references, the
    // borrow checker guarantees no overlap. `len` is bounded by both
    // slice lengths.
    unsafe {
        core::ptr::copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr() as *mut u8, len);
    }
}

/// An additional account to pass in an Execute CPI, with its
/// signer/writable flags.
pub struct AdditionalAccount<'a> {
    /// The account view.
    pub account: &'a AccountView,
    /// Whether this account is a signer.
    pub is_signer: bool,
    /// Whether this account is writable.
    pub is_writable: bool,
}

/// Invoke the `Execute` instruction on a transfer hook program.
///
/// This is the CPI that Token-2022 (or any caller) uses to invoke the
/// hook program during a transfer. The instruction data is the 8-byte
/// Execute discriminator followed by the transfer amount as a little-
/// endian `u64`.
///
/// ### Accounts:
///   0. `[]` Source token account.
///   1. `[]` Token mint.
///   2. `[]` Destination token account.
///   3. `[]` Source token account owner/delegate.
///   4. `[]` Extra account metas PDA (validation state).
///   5. ..5+M `[]` Additional accounts from the extra account metas.
pub struct Execute<'a, 'b> {
    /// Source token account.
    pub source: &'a AccountView,
    /// Token mint.
    pub mint: &'a AccountView,
    /// Destination token account.
    pub destination: &'a AccountView,
    /// Source account owner/delegate.
    pub authority: &'a AccountView,
    /// Extra account metas PDA (validation state).
    pub extra_account_metas_pda: &'a AccountView,
    /// Additional accounts required by the hook, with their
    /// signer/writable flags from the [`ExtraAccountMeta`] entries.
    pub additional_accounts: &'b [AdditionalAccount<'a>],
    /// Transfer hook program address.
    pub program_id: &'b Address,
    /// Amount of tokens being transferred.
    pub amount: u64,
}

/// Maximum number of total accounts for Execute CPI (5 fixed + up to
/// 30 additional).
const MAX_EXECUTE_ACCOUNTS: usize = 35;

impl Execute<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let additional_len = self.additional_accounts.len();
        let total_accounts = 5 + additional_len;

        // Instruction accounts: 5 fixed + additional.
        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_EXECUTE_ACCOUNTS];

        instruction_accounts[0].write(InstructionAccount::readonly(self.source.address()));
        instruction_accounts[1].write(InstructionAccount::readonly(self.mint.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(self.destination.address()));
        instruction_accounts[3].write(InstructionAccount::readonly(self.authority.address()));
        instruction_accounts[4].write(InstructionAccount::readonly(
            self.extra_account_metas_pda.address(),
        ));

        for (slot, extra) in instruction_accounts[5..]
            .iter_mut()
            .zip(self.additional_accounts.iter())
        {
            slot.write(InstructionAccount::new(
                extra.account.address(),
                extra.is_writable,
                extra.is_signer,
            ));
        }

        // Account views: 5 fixed + additional.
        const UNINIT_INFO: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut accounts = [UNINIT_INFO; MAX_EXECUTE_ACCOUNTS];

        // SAFETY: The allocation is valid to the maximum number of
        // accounts.
        unsafe {
            accounts.get_unchecked_mut(0).write(self.source);
            accounts.get_unchecked_mut(1).write(self.mint);
            accounts.get_unchecked_mut(2).write(self.destination);
            accounts.get_unchecked_mut(3).write(self.authority);
            accounts
                .get_unchecked_mut(4)
                .write(self.extra_account_metas_pda);

            for (slot, extra) in accounts
                .get_unchecked_mut(5..)
                .iter_mut()
                .zip(self.additional_accounts.iter())
            {
                slot.write(extra.account);
            }
        }

        // Instruction data: 8-byte discriminator + 8-byte amount.
        let mut instruction_data = [UNINIT_BYTE; 16];
        write_bytes(&mut instruction_data[..8], &EXECUTE_DISCRIMINATOR);
        write_bytes(&mut instruction_data[8..16], &self.amount.to_le_bytes());

        invoke_signed_with_bounds::<MAX_EXECUTE_ACCOUNTS, _>(
            &InstructionView {
                program_id: self.program_id,
                // SAFETY: `total_accounts` entries are initialized.
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, total_accounts)
                },
                // SAFETY: instruction data is fully initialized.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 16) },
            },
            // SAFETY: `total_accounts` entries are initialized.
            unsafe { from_raw_parts(accounts.as_ptr() as *const &AccountView, total_accounts) },
            signers,
        )
    }
}

/// Derive the extra account metas PDA address for a given mint and
/// program.
///
/// Returns `(address, bump)`.
#[cfg(feature = "find-pda")]
#[inline]
pub fn get_extra_account_metas_address(mint: &Address, program_id: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[crate::EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()],
        program_id,
    )
}
