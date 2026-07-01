use crate::definitions::TokenProgram;
use solana_address::Address;

#[cfg(feature = "alloc")]
pub use crate::definitions::{BatchState, IntoBatch};

/// Represent the SPL Token program.
pub struct Program;

impl TokenProgram for Program {
    #[inline(always)]
    fn id() -> Address {
        crate::ID
    }
}

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
pub type AmountToUiAmount<'account> = crate::definitions::AmountToUiAmount<'account, Program>;

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
    crate::definitions::Approve<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::ApproveChecked<'account, 'multisig, MultisigSigner, Program>;

/// A collection of instructions that can be serialized into a token `Batch`
/// instruction.
pub type Batch<'account, 'state> = crate::definitions::Batch<'account, 'state, Program>;

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
    crate::definitions::Burn<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::BurnChecked<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::CloseAccount<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::FreezeAccount<'account, 'multisig, MultisigSigner, Program>;

/// Gets the required size of an account for the given mint as a
/// little-endian `u64`.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type GetAccountDataSize<'account> = crate::definitions::GetAccountDataSize<'account, Program>;

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
pub type InitializeAccount<'account> = crate::definitions::InitializeAccount<'account, Program>;

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
pub type InitializeAccount2<'account> = crate::definitions::InitializeAccount2<'account, Program>;

/// Like [`InitializeAccount2`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub type InitializeAccount3<'account, 'address> =
    crate::definitions::InitializeAccount3<'account, 'address, Program>;

/// Initialize the Immutable Owner extension for the given token account
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeAccount`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
pub type InitializeImmutableOwner<'account> =
    crate::definitions::InitializeImmutableOwner<'account, Program>;

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
    crate::definitions::InitializeMint<'account, 'address, Program>;

/// Like [`InitializeMint`], but does not require the Rent
/// sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub type InitializeMint2<'account, 'address> =
    crate::definitions::InitializeMint2<'account, 'address, Program>;

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
    crate::definitions::InitializeMultisig<'account, 'multisig, MultisigSigner, Program>;

/// Like [`InitializeMultisig`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The multisignature account to initialize.
///   1. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <=
///      11`.
pub type InitializeMultisig2<'account, 'multisig, MultisigSigner> =
    crate::definitions::InitializeMultisig2<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::MintTo<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::MintToChecked<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::Revoke<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::SetAuthority<'account, 'address, 'multisig, MultisigSigner, Program>;

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
pub type SyncNative<'account> = crate::definitions::SyncNative<'account, Program>;

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
    crate::definitions::ThawAccount<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::Transfer<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::TransferChecked<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::UiAmountToAmount<'account, 'amount, Program>;

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
    crate::definitions::UnwrapLamports<'account, 'multisig, MultisigSigner, Program>;

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
    crate::definitions::WithdrawExcessLamports<'account, 'multisig, MultisigSigner, Program>;
