#[cfg(feature = "alloc")]
pub use crate::instructions::batch::{BatchState, IntoBatch};
use {
    crate::Program,
    core::mem::MaybeUninit,
    solana_instruction_view::{cpi::CpiAccount, InstructionAccount},
    solana_program_error::ProgramError,
};

pub mod amount_to_ui_amount;
pub mod approve;
pub mod approve_checked;
pub mod batch;
pub mod burn;
pub mod burn_checked;
pub mod close_account;
pub mod freeze_account;
pub mod get_account_data_size;
pub mod initialize_account;
pub mod initialize_account2;
pub mod initialize_account3;
pub mod initialize_immutable_owner;
pub mod initialize_mint;
pub mod initialize_mint2;
pub mod initialize_multisig;
pub mod initialize_multisig2;
pub mod mint_to;
pub mod mint_to_checked;
pub mod revoke;
pub mod set_authority;
pub mod sync_native;
pub mod thaw_account;
pub mod transfer;
pub mod transfer_checked;
pub mod ui_amount_to_amount;
pub mod unwrap_lamports;
pub mod withdraw_excess_lamports;

/// Constant for an uninitialized byte.
const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

/// Constant for an uninitialized `CpiAccount`.
const UNINIT_CPI_ACCOUNT: MaybeUninit<CpiAccount> = MaybeUninit::<CpiAccount>::uninit();

/// Constant for an uninitialized `InstructionAccount`.
const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
    MaybeUninit::<InstructionAccount>::uninit();

/// Convert an Amount of tokens to a `UiAmount` string, using the given
/// mint.
///
/// Fails on an invalid mint.
///
/// Return data can be fetched using `sol_get_return_data` and deserialized
/// with `String::from_utf8`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type AmountToUiAmount<'account> = amount_to_ui_amount::AmountToUiAmount<'account, Program>;

/// Approves a delegate.  A delegate is given the authority over tokens on
/// behalf of the source account's owner.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The source account.
///   1. `[]` The delegate.
///   2. `[signer]` The source account owner.
///
///   * Multisignature owner
///   0. `[writable]` The source account.
///   1. `[]` The delegate.
///   2. `[]` The source account's multisignature owner.
///   3. `..+M` `[signer]` M signer accounts.
pub type Approve<'account, 'multisig, MultisigSigner> =
    approve::Approve<'account, 'multisig, MultisigSigner, Program>;

/// Approves a delegate.  A delegate is given the authority over tokens on
/// behalf of the source account's owner.
///
/// This instruction differs from Approve in that the token mint and
/// decimals value is checked by the caller.  This may be useful when
/// creating transactions offline or within a hardware wallet.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[]` The delegate.
///   3. `[signer]` The source account owner.
///
///   * Multisignature owner
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[]` The delegate.
///   3. `[]` The source account's multisignature owner.
///   4. `..+M` `[signer]` M signer accounts.
pub type ApproveChecked<'account, 'multisig, MultisigSigner> =
    approve_checked::ApproveChecked<'account, 'multisig, MultisigSigner, Program>;

/// A collection of instructions that can be serialized into a token `Batch`
/// instruction.
pub type Batch<'account, 'state> = batch::Batch<'account, 'state, Program>;

/// Burns tokens by removing them from an account.  `Burn` does not support
/// accounts associated with the native mint, use `CloseAccount` instead.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The account to burn from.
///   1. `[writable]` The token mint.
///   2. `[signer]` The account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The account to burn from.
///   1. `[writable]` The token mint.
///   2. `[]` The account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub type Burn<'account, 'multisig, MultisigSigner> =
    burn::Burn<'account, 'multisig, MultisigSigner, Program>;

/// Burns tokens by removing them from an account.
/// [`BurnChecked`] does not support accounts
/// associated with the native mint, use `CloseAccount` instead.
///
/// This instruction differs from Burn in that the decimals value is checked
/// by the caller. This may be useful when creating transactions offline or
/// within a hardware wallet.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The account to burn from.
///   1. `[writable]` The token mint.
///   2. `[signer]` The account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The account to burn from.
///   1. `[writable]` The token mint.
///   2. `[]` The account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub type BurnChecked<'account, 'multisig, MultisigSigner> =
    burn_checked::BurnChecked<'account, 'multisig, MultisigSigner, Program>;

/// Close an account by transferring all its SOL to the destination account.
/// Non-native accounts may only be closed if its token amount is zero.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The account to close.
///   1. `[writable]` The destination account.
///   2. `[signer]` The account's owner.
///
///   * Multisignature owner
///   0. `[writable]` The account to close.
///   1. `[writable]` The destination account.
///   2. `[]` The account's multisignature owner.
///   3. `..+M` `[signer]` M signer accounts.
pub type CloseAccount<'account, 'multisig, MultisigSigner> =
    close_account::CloseAccount<'account, 'multisig, MultisigSigner, Program>;

/// Freeze an Initialized account using the Mint's `freeze_authority` (if
/// set).
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The account to freeze.
///   1. `[]` The token mint.
///   2. `[signer]` The mint freeze authority.
///
///   * Multisignature owner
///   0. `[writable]` The account to freeze.
///   1. `[]` The token mint.
///   2. `[]` The mint's multisignature freeze authority.
///   3. `..+M` `[signer]` M signer accounts.
pub type FreezeAccount<'account, 'multisig, MultisigSigner> =
    freeze_account::FreezeAccount<'account, 'multisig, MultisigSigner, Program>;

/// Gets the required size of an account for the given mint as a
/// little-endian `u64`.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type GetAccountDataSize<'account> =
    get_account_data_size::GetAccountDataSize<'account, Program>;

/// Initializes a new account to hold tokens.  If this account is associated
/// with the native mint then the token balance of the initialized account
/// will be equal to the amount of SOL in the account. If this account is
/// associated with another mint, that mint must be initialized before this
/// command can succeed.
///
/// The [`InitializeAccount`] instruction requires no
/// signers and MUST be included within the same Transaction as the
/// system program's `CreateAccount` instruction that creates the
/// account being initialized. Otherwise another party can acquire
/// ownership of the uninitialized account.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multisignature.
///   3. `[]` Rent sysvar.
pub type InitializeAccount<'account> = initialize_account::InitializeAccount<'account, Program>;

/// Like [`InitializeAccount`], but the owner pubkey is
/// passed via instruction data rather than the accounts list. This
/// variant may be preferable when using Cross Program Invocation from
/// an instruction that does not need the owner's `AccountInfo`
/// otherwise.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` Rent sysvar.
pub type InitializeAccount2<'account> = initialize_account2::InitializeAccount2<'account, Program>;

/// Like [`InitializeAccount2`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub type InitializeAccount3<'account, 'address> =
    initialize_account3::InitializeAccount3<'account, 'address, Program>;

/// Initialize the Immutable Owner extension for the given token account
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeAccount`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
pub type InitializeImmutableOwner<'account> =
    initialize_immutable_owner::InitializeImmutableOwner<'account, Program>;

/// Initializes a new mint and optionally deposits all the newly minted
/// tokens in an account.
///
/// The `InitializeMint` instruction requires no signers and MUST be
/// included within the same Transaction as the system program's
/// `CreateAccount` instruction that creates the account being initialized.
/// Otherwise another party can acquire ownership of the uninitialized
/// account.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
///   1. `[]` Rent sysvar.
pub type InitializeMint<'account, 'address> =
    initialize_mint::InitializeMint<'account, 'address, Program>;

/// Like [`InitializeMint`], but does not require the Rent
/// sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub type InitializeMint2<'account, 'address> =
    initialize_mint2::InitializeMint2<'account, 'address, Program>;

/// Initializes a multisignature account with N provided signers.
///
/// Multisignature accounts can used in place of any single owner/delegate
/// accounts in any token instruction that require an owner/delegate to be
/// present.  The variant field represents the number of signers (M)
/// required to validate this multisignature account.
///
/// The [`InitializeMultisig`] instruction requires no
/// signers and MUST be included within the same Transaction as the
/// system program's `CreateAccount` instruction that creates the
/// account being initialized. Otherwise another party can acquire
/// ownership of the uninitialized account.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The multisignature account to initialize.
///   1. `[]` Rent sysvar.
///   2. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <=
///      11`.
pub type InitializeMultisig<'account, 'multisig, MultisigSigner> =
    initialize_multisig::InitializeMultisig<'account, 'multisig, MultisigSigner, Program>;

/// Like [`InitializeMultisig`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The multisignature account to initialize.
///   1. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <=
///      11`.
pub type InitializeMultisig2<'account, 'multisig, MultisigSigner> =
    initialize_multisig2::InitializeMultisig2<'account, 'multisig, MultisigSigner, Program>;

/// Mints new tokens to an account.  The native mint does not support
/// minting.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[writable]` The account to mint tokens to.
///   2. `[signer]` The mint's minting authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[writable]` The account to mint tokens to.
///   2. `[]` The mint's multisignature mint-tokens authority.
///   3. `..+M` `[signer]` M signer accounts.
pub type MintTo<'account, 'multisig, MultisigSigner> =
    mint_to::MintTo<'account, 'multisig, MultisigSigner, Program>;

/// Mints new tokens to an account.  The native mint does not support
/// minting.
///
/// This instruction differs from [`MintTo`] in that the
/// decimals value is checked by the caller.  This may be useful when
/// creating transactions offline or within a hardware wallet.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[writable]` The account to mint tokens to.
///   2. `[signer]` The mint's minting authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[writable]` The account to mint tokens to.
///   2. `[]` The mint's multisignature mint-tokens authority.
///   3. `..+M` `[signer]` M signer accounts.
pub type MintToChecked<'account, 'multisig, MultisigSigner> =
    mint_to_checked::MintToChecked<'account, 'multisig, MultisigSigner, Program>;

/// Revokes the delegate's authority.
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The source account.
///   1. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[]` The source account's multisignature owner/delegate.
///   2. `..+M` `[signer]` M signer accounts.
pub type Revoke<'account, 'multisig, MultisigSigner> =
    revoke::Revoke<'account, 'multisig, MultisigSigner, Program>;

/// Sets a new authority of a mint or account.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint or account to change the authority of.
///   1. `[signer]` The current authority of the mint or account.
///
///   * Multisignature authority
///   0. `[writable]` The mint or account to change the authority of.
///   1. `[]` The mint's or account's current multisignature authority.
///   2. `..+M` `[signer]` M signer accounts.
pub type SetAuthority<'account, 'address, 'multisig, MultisigSigner> =
    set_authority::SetAuthority<'account, 'address, 'multisig, MultisigSigner, Program>;

/// Given a wrapped / native token account (a token account containing SOL)
/// updates its amount field based on the account's underlying `lamports`.
/// This is useful if a non-wrapped SOL account uses
/// `system_instruction::transfer` to move lamports to a wrapped token
/// account, and needs to have its token `amount` field updated.
///
/// Accounts expected by this instruction:
///
///   * Using runtime Rent sysvar
///   0. `[writable]`  The native token account to sync with its underlying
///      lamports.
///
///   * Using Rent sysvar account
///   0. `[writable]`  The native token account to sync with its underlying
///      lamports.
///   1. `[]` Rent sysvar.
pub type SyncNative<'account> = sync_native::SyncNative<'account, Program>;

/// Thaw a Frozen account using the Mint's `freeze_authority` (if set).
///
/// Accounts expected by this instruction:
///
///   * Single owner
///   0. `[writable]` The account to thaw.
///   1. `[]` The token mint.
///   2. `[signer]` The mint freeze authority.
///
///   * Multisignature owner
///   0. `[writable]` The account to thaw.
///   1. `[]` The token mint.
///   2. `[]` The mint's multisignature freeze authority.
///   3. `..+M` `[signer]` M signer accounts.
pub type ThawAccount<'account, 'multisig, MultisigSigner> =
    thaw_account::ThawAccount<'account, 'multisig, MultisigSigner, Program>;

/// Transfers tokens from one account to another either directly or via a
/// delegate.  If this account is associated with the native mint then equal
/// amounts of SOL and Tokens will be transferred to the destination
/// account.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[]` The source account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub type Transfer<'account, 'multisig, MultisigSigner> =
    transfer::Transfer<'account, 'multisig, MultisigSigner, Program>;

/// Transfers tokens from one account to another either directly or via a
/// delegate.  If this account is associated with the native mint then equal
/// amounts of SOL and Tokens will be transferred to the destination
/// account.
///
/// This instruction differs from [`Transfer`] in that the token mint and
/// decimals value is checked by the caller.  This may be useful when
/// creating transactions offline or within a hardware wallet.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[writable]` The destination account.
///   3. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[writable]` The destination account.
///   3. `[]` The source account's multisignature owner/delegate.
///   4. `..+M` `[signer]` M signer accounts.
pub type TransferChecked<'account, 'multisig, MultisigSigner> =
    transfer_checked::TransferChecked<'account, 'multisig, MultisigSigner, Program>;

/// Convert a `UiAmount` of tokens to a little-endian `u64` raw Amount,
/// using the given mint. In this version of the program, the mint can
/// only specify the number of decimals.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type UiAmountToAmount<'account, 'amount> =
    ui_amount_to_amount::UiAmountToAmount<'account, 'amount, Program>;

/// Transfer lamports from a native SOL account to a destination account.
///
/// This is useful to unwrap lamports from a wrapped SOL account.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[]` The source account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub type UnwrapLamports<'account, 'multisig, MultisigSigner> =
    unwrap_lamports::UnwrapLamports<'account, 'multisig, MultisigSigner, Program>;

/// This instruction is to be used to rescue SOL sent to any `TokenProgram`
/// owned account by sending them to any other account, leaving behind only
/// lamports for rent exemption.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[writable]` The destination account.
///   2. `[]` The source account's multisignature owner/delegate.
///   3. `..+M` `[signer]` M signer accounts.
pub type WithdrawExcessLamports<'account, 'multisig, MultisigSigner> =
    withdraw_excess_lamports::WithdrawExcessLamports<'account, 'multisig, MultisigSigner, Program>;

#[cold]
fn account_borrow_failed_error() -> ProgramError {
    ProgramError::AccountBorrowFailed
}

#[cold]
fn invalid_argument_error() -> ProgramError {
    ProgramError::InvalidArgument
}

/// A helper function to write bytes from a source slice to a destination slice of `MaybeUninit<u8>`.
#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    let len = destination.len().min(source.len());
    // SAFETY:
    // - Both pointers have alignment 1.
    // - For valid (non-UB) references, the borrow checker guarantees no overlap.
    // - `len` is bounded by both slice lengths.
    unsafe {
        core::ptr::copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr() as *mut u8, len);
    }
}

/// A trait for instructions that can be used in a CPI context.
pub trait CpiWriter {
    /// Writes the `AccountView`s required by this instruction into the provided
    /// slice.
    ///
    /// Returns the number of accounts written.
    fn write_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<CpiAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi;

    /// Writes the `InstructionAccount`s required by this instruction into the
    /// provided slice.
    ///
    /// Returns the number of accounts written.
    fn write_instruction_accounts<'source, 'cpi>(
        &'source self,
        accounts: &mut [MaybeUninit<InstructionAccount<'cpi>>],
    ) -> Result<usize, ProgramError>
    where
        Self: 'cpi;

    /// Writes the instruction data for this instruction into the provided
    /// slice.
    ///
    /// Returns the number of bytes written.
    fn write_instruction_data(&self, data: &mut [MaybeUninit<u8>]) -> Result<usize, ProgramError>;
}
