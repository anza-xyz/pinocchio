//! Instruction types.

use alloc::{vec, vec::Vec};
#[cfg(feature = "wincode")]
use solana_address::Address;
#[cfg(feature = "wincode")]
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::{ProgramError, ProgramResult};

use crate::{error::TokenError, ID};

/// Instructions supported by the token program.
#[repr(u8)]
#[cfg_attr(feature = "wincode", derive(wincode::SchemaRead, wincode::SchemaWrite))]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
pub enum TokenInstruction {
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
    InitializeMint {
        /// Number of base 10 digits to the right of the decimal place.
        decimals: u8,
        /// The authority/multisignature to mint tokens.
        mint_authority: Address,
        /// The freeze authority/multisignature of the mint.
        freeze_authority: Option<Address>,
    },

    /// Initializes a new account to hold tokens.  If this account is associated
    /// with the native mint then the token balance of the initialized account
    /// will be equal to the amount of SOL in the account. If this account is
    /// associated with another mint, that mint must be initialized before this
    /// command can succeed.
    ///
    /// The [`TokenInstruction::InitializeAccount`] instruction requires no
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
    InitializeAccount,

    /// Initializes a multisignature account with N provided signers.
    ///
    /// Multisignature accounts can used in place of any single owner/delegate
    /// accounts in any token instruction that require an owner/delegate to be
    /// present.  The variant field represents the number of signers (M)
    /// required to validate this multisignature account.
    ///
    /// The [`TokenInstruction::InitializeMultisig`] instruction requires no
    /// signers and MUST be included within the same Transaction as the
    /// system program's `CreateAccount` instruction that creates the
    /// account being initialized. Otherwise another party can acquire
    /// ownership of the uninitialized account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The multisignature account to initialize.
    ///   1. `[]` Rent sysvar.
    ///   2. `..+N` `[signer]` The signer accounts, must equal to N where `1 <=
    ///      N <= 11`.
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u8` The number of signers (M) required to validate this
    ///     multisignature account.
    InitializeMultisig,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens to transfer.
    Transfer,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens the delegate is approved for.
    Approve,

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
    Revoke,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `AuthorityType` The type of authority to update.
    ///   - `Option<Pubkey>` The new authority.
    SetAuthority,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of new tokens to mint.
    MintTo,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens to burn.
    Burn,

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
    CloseAccount,

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
    FreezeAccount,

    /// Thaw a Frozen account using the Mint's `freeze_authority` (if set).
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
    ThawAccount,

    /// Transfers tokens from one account to another either directly or via a
    /// delegate.  If this account is associated with the native mint then equal
    /// amounts of SOL and Tokens will be transferred to the destination
    /// account.
    ///
    /// This instruction differs from Transfer in that the token mint and
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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens to transfer.
    ///   - `u8` Expected number of base 10 digits to the right of the decimal
    ///     place.
    TransferChecked,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens the delegate is approved for.
    ///   - `u8` Expected number of base 10 digits to the right of the decimal
    ///     place.
    ApproveChecked,

    /// Mints new tokens to an account.  The native mint does not support
    /// minting.
    ///
    /// This instruction differs from [`TokenInstruction::MintTo`] in that the
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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of new tokens to mint.
    ///   - `u8` Expected number of base 10 digits to the right of the decimal
    ///     place.
    MintToChecked,

    /// Burns tokens by removing them from an account.
    /// [`TokenInstruction::BurnChecked`] does not support accounts
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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens to burn.
    ///   - `u8` Expected number of base 10 digits to the right of the decimal
    ///     place.
    BurnChecked,

    /// Like [`TokenInstruction::InitializeAccount`], but the owner pubkey is
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
    ///
    /// Data expected by this instruction:
    ///
    ///  - `Pubkey` The new account's owner/multisignature.
    InitializeAccount2,

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
    SyncNative,

    /// Like [`TokenInstruction::InitializeAccount2`], but does not require the
    /// Rent sysvar to be provided
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]`  The account to initialize.
    ///   1. `[]` The mint this account will be associated with.
    ///
    /// Data expected by this instruction:
    ///
    /// - `Pubkey` The new account's owner/multisignature.
    InitializeAccount3,

    /// Like [`TokenInstruction::InitializeMultisig`], but does not require the
    /// Rent sysvar to be provided
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The multisignature account to initialize.
    ///   1. `..+N` `[signer]` The signer accounts, must equal to N where `1 <=
    ///      N <= 11`.
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u8` The number of signers (M) required to validate this
    ///     multisignature account.
    InitializeMultisig2,

    /// Like [`TokenInstruction::InitializeMint`], but does not require the Rent
    /// sysvar to be provided
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The mint to initialize.
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u8` The number of base 10 digits to the right of the decimal place.
    ///   - `Pubkey` The authority/multisignature to mint tokens.
    ///   - `Option<Pubkey>` The freeze authority/multisignature of the mint.
    InitializeMint2,

    /// Gets the required size of an account for the given mint as a
    /// little-endian `u64`.
    ///
    /// Return data can be fetched using `sol_get_return_data` and deserializing
    /// the return data as a little-endian `u64`.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The mint to calculate for.
    GetAccountDataSize,

    /// Initialize the Immutable Owner extension for the given token account
    ///
    /// Fails if the account has already been initialized, so must be called
    /// before [`TokenInstruction::InitializeAccount`].
    ///
    /// No-ops in this version of the program, but is included for compatibility
    /// with the Associated Token Account program.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]`  The account to initialize.
    InitializeImmutableOwner,

    /// Convert an Amount of tokens to a `UiAmount` `string`, using the given
    /// mint. In this version of the program, the mint can only specify the
    /// number of decimals.
    ///
    /// Fails on an invalid mint.
    ///
    /// Return data can be fetched using `sol_get_return_data` and deserialized
    /// with `String::from_utf8`.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The mint to calculate for
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The amount of tokens to reformat.
    AmountToUiAmount,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `&str` The `ui_amount` of tokens to reformat.
    UiAmountToAmount,

    /// This instruction is to be used to rescue SOL sent to any `TokenProgram`
    /// owned account by sending them to any other account, leaving behind only
    /// lamports for rent exemption.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source Account owned by the token program
    ///   1. `[writable]` Destination account
    ///   2. `[signer]` Authority
    ///   3. `..+M` `[signer]` M signer accounts.
    WithdrawExcessLamports = 38,

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
    ///
    /// Data expected by this instruction:
    ///
    ///   - `Option<u64>` The amount of lamports to transfer. When an amount is
    ///     not specified, the entire balance of the source account will be
    ///     transferred.
    UnwrapLamports = 45,

    /// Executes a batch of instructions. The instructions to be executed are
    /// specified in sequence on the instruction data. Each instruction
    /// provides:
    ///   - `u8`: number of accounts
    ///   - `u8`: instruction data length (includes the discriminator)
    ///   - `u8`: instruction discriminator
    ///   - `[u8]`: instruction data
    ///
    /// Accounts follow a similar pattern, where accounts for each instruction
    /// are specified in sequence. Therefore, the number of accounts
    /// expected by this instruction is variable, i.e., it depends on the
    /// instructions provided.
    ///
    /// Both the number of accounts and instruction data length are used to
    /// identify the slice of accounts and instruction data for each
    /// instruction.
    ///
    /// Note that it is not sound to have a `batch` instruction that contains
    /// other `batch` instruction; an error will be raised when this is
    /// detected.
    Batch = 255,
    // Any new variants also need to be added to program-2022 `TokenInstruction`, so that the
    // latter remains a superset of this instruction set. New variants also need to be added to
    // token/js/src/instructions/types.ts to maintain @solana/spl-token compatibility
}

/// Specifies the authority type for `SetAuthority` instructions
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(strum_macros::FromRepr, strum_macros::EnumIter))]
pub enum AuthorityType {
    /// Authority to mint new tokens
    MintTokens,
    /// Authority to freeze any account associated with the Mint
    FreezeAccount,
    /// Owner of a given token account
    AccountOwner,
    /// Authority to close a token account
    CloseAccount,
}

impl TryFrom<u8> for AuthorityType {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // SAFETY: `value` is guaranteed to be in the range of the enum variants.
            0..=3 => Ok(unsafe { core::mem::transmute::<u8, AuthorityType>(value) }),
            _ => Err(TokenError::InvalidInstruction.into()),
        }
    }
}

/// Create an instruction using wincode.
#[cfg(feature = "wincode")]
#[inline(always)]
fn create_instruction<T>(program_id: Address, data: &T, accounts: Vec<AccountMeta>) -> Instruction
where
    T: wincode::Serialize<Src = T>,
{
    Instruction::new_with_wincode(program_id, data, accounts)
}

/// Checks that the supplied program ID is the correct one for SPL-token
fn check_program_account(program_id: &Address) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

/// Creates a `InitializeMint` instruction.
pub fn initialize_mint(
    token_program_id: &Address,
    mint_pubkey: &Address,
    mint_authority_pubkey: &Address,
    freeze_authority_pubkey: Option<&Address>,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    let freeze_authority = freeze_authority_pubkey.cloned().into();
    let data = TokenInstruction::InitializeMint {
        mint_authority: *mint_authority_pubkey,
        freeze_authority,
        decimals,
    };

    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(create_instruction(*token_program_id, &data, accounts))
}

#[cfg(test)]
mod tests {
    use {
        super::{AuthorityType, TokenInstruction},
        strum::IntoEnumIterator,
    };

    #[test]
    fn test_token_instruction_from_u8_exhaustive() {
        for variant in TokenInstruction::iter() {
            let variant_u8 = variant.clone() as u8;
            assert_eq!(
                TokenInstruction::from_repr(variant_u8),
                Some(TokenInstruction::try_from(variant_u8).unwrap())
            );
            assert_eq!(TokenInstruction::try_from(variant_u8).unwrap(), variant);
        }
    }

    #[test]
    fn test_authority_type_from_u8_exhaustive() {
        for variant in AuthorityType::iter() {
            let variant_u8 = variant.clone() as u8;
            assert_eq!(
                AuthorityType::from_repr(variant_u8),
                Some(AuthorityType::try_from(variant_u8).unwrap())
            );
            assert_eq!(AuthorityType::try_from(variant_u8).unwrap(), variant);
        }
    }
}
