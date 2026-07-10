use {
    crate::{state::ExtensionType, Token2022Program},
    core::mem::MaybeUninit,
    solana_program_error::ProgramError,
};

mod create_native_mint;
mod extensions;
pub mod get_account_data_size;
mod initialize_non_transferable_mint;
mod reallocate;

pub use {
    create_native_mint::*,
    extensions::*,
    initialize_non_transferable_mint::*,
    pinocchio_token::instructions::{
        batch::IntoBatch, initialize_multisig::MAX_MULTISIG_SIGNERS, set_authority::AuthorityType,
        unwrap_lamports::Amount,
    },
    reallocate::*,
};

/// The maximum number of available extensions.
const MAX_EXTENSION_COUNT: usize = 28;

/// The length of the instruction data for instructions that take a list of
/// extension types.
const EXTENSION_TYPES_INSTRUCTION_DATA_LEN: usize = 1 + MAX_EXTENSION_COUNT * 2;

/// Convert an Amount of tokens to a `UiAmount` string, using the given
/// mint.
///
/// Fails on an invalid mint.
///
/// Return data can be fetched using `sol_get_return_data` and deserialized
/// with `String::from_utf8`.
///
/// WARNING: For mints using the interest-bearing or scaled-ui-amount
/// extensions, this instruction uses standard floating-point arithmetic to
/// convert values, which is not guaranteed to give consistent behavior.
///
/// In particular, conversions will not always work in reverse. For example,
/// if you pass amount `A` to `AmountToUiAmount` and receive `B`, and pass
/// the result `B` to `UiAmountToAmount`, you will not always get back `A`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type AmountToUiAmount<'account> =
    pinocchio_token::instructions::amount_to_ui_amount::AmountToUiAmount<
        'account,
        Token2022Program,
    >;

/// Approves a delegate. A delegate is given the authority over tokens on
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
    pinocchio_token::instructions::approve::Approve<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Approves a delegate. A delegate is given the authority over tokens on
/// behalf of the source account's owner.
///
/// This instruction differs from Approve in that the token mint and
/// decimals value is checked by the caller. This may be useful when
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
    pinocchio_token::instructions::approve_checked::ApproveChecked<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// A collection of instructions that can be serialized into a token `Batch`
/// instruction.
pub type Batch<'account, 'state> =
    pinocchio_token::instructions::batch::Batch<'account, 'state, Token2022Program>;

#[cfg(feature = "alloc")]
/// A state object that contains the buffers for a `Batch` instruction.
pub type BatchState<'account> =
    pinocchio_token::instructions::batch::BatchState<'account, Token2022Program>;

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
pub type Burn<'account, 'multisig, MultisigSigner> = pinocchio_token::instructions::burn::Burn<
    'account,
    'multisig,
    MultisigSigner,
    Token2022Program,
>;

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
    pinocchio_token::instructions::burn_checked::BurnChecked<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::close_account::CloseAccount<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::freeze_account::FreezeAccount<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Gets the required size of an account for the given mint as a
/// little-endian `u64`.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type GetAccountDataSize<'account, 'extensions> =
    get_account_data_size::GetAccountDataSize<'account, 'extensions, Token2022Program>;

/// Initializes a new account to hold tokens. If this account is associated
/// with the native mint then the token balance of the initialized account
/// will be equal to the amount of SOL in the account. If this account is
/// associated with another mint, that mint must be initialized before this
/// command can succeed.
///
/// The `InitializeAccount` instruction requires no
/// signers and MUST be included within the same Transaction as the
/// system program's `CreateAccount` instruction that creates the
/// account being initialized. Otherwise another party can acquire
/// ownership of the uninitialized account.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multisignature.
///   3. `[]` Rent sysvar.
pub type InitializeAccount<'account> =
    pinocchio_token::instructions::initialize_account::InitializeAccount<
        'account,
        Token2022Program,
    >;

/// Like [`InitializeAccount`], but the owner pubkey is
/// passed via instruction data rather than the accounts list. This
/// variant may be preferable when using Cross Program Invocation from
/// an instruction that does not need the owner's `AccountInfo`
/// otherwise.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` Rent sysvar.
pub type InitializeAccount2<'account> =
    pinocchio_token::instructions::initialize_account2::InitializeAccount2<
        'account,
        Token2022Program,
    >;

/// Like [`InitializeAccount2`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub type InitializeAccount3<'account, 'address> =
    pinocchio_token::instructions::initialize_account3::InitializeAccount3<
        'account,
        'address,
        Token2022Program,
    >;

/// Initialize the Immutable Owner extension for the given token account
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeAccount`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]`  The account to initialize.
pub type InitializeImmutableOwner<'account> =
    pinocchio_token::instructions::initialize_immutable_owner::InitializeImmutableOwner<
        'account,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::initialize_mint::InitializeMint<
        'account,
        'address,
        Token2022Program,
    >;

/// Like [`InitializeMint`], but does not require the Rent
/// sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub type InitializeMint2<'account, 'address> =
    pinocchio_token::instructions::initialize_mint2::InitializeMint2<
        'account,
        'address,
        Token2022Program,
    >;

/// Initializes a multisignature account with N provided signers.
///
/// Multisignature accounts can used in place of any single owner/delegate
/// accounts in any token instruction that require an owner/delegate to be
/// present. The variant field represents the number of signers (M)
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
    pinocchio_token::instructions::initialize_multisig::InitializeMultisig<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Like [`InitializeMultisig`], but does not require the
/// Rent sysvar to be provided
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The multisignature account to initialize.
///   1. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <=
///      11`.
pub type InitializeMultisig2<'account, 'multisig, MultisigSigner> =
    pinocchio_token::instructions::initialize_multisig2::InitializeMultisig2<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Mints new tokens to an account. The native mint does not support
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
    pinocchio_token::instructions::mint_to::MintTo<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Mints new tokens to an account. The native mint does not support
/// minting.
///
/// This instruction differs from [`MintTo`] in that the
/// decimals value is checked by the caller. This may be useful when
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
    pinocchio_token::instructions::mint_to_checked::MintToChecked<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::revoke::Revoke<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::set_authority::SetAuthority<
        'account,
        'address,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Given a wrapped / native token account (a token account containing SOL)
/// updates its amount field based on the account's underlying `lamports`.
/// This is useful if a non-wrapped SOL account uses
/// `system_instruction::transfer` to move lamports to a wrapped token
/// account, and needs to have its token `amount` field updated.
///
/// Accounts expected by this instruction:
///
///   * Using runtime Rent sysvar
///   0. `[writable]` The native token account to sync with its underlying
///      lamports.
///
///   * Using Rent sysvar account
///   0. `[writable]` The native token account to sync with its underlying
///      lamports.
///   1. `[]` Rent sysvar.
pub type SyncNative<'account> =
    pinocchio_token::instructions::sync_native::SyncNative<'account, Token2022Program>;

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
    pinocchio_token::instructions::thaw_account::ThawAccount<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Transfers tokens from one account to another either directly or via a
/// delegate. If this account is associated with the native mint then equal
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
    pinocchio_token::instructions::transfer::Transfer<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Transfers tokens from one account to another either directly or via a
/// delegate. If this account is associated with the native mint then equal
/// amounts of SOL and Tokens will be transferred to the destination
/// account.
///
/// This instruction differs from [`Transfer`] in that the token mint and
/// decimals value is checked by the caller. This may be useful when
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
    pinocchio_token::instructions::transfer_checked::TransferChecked<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

/// Convert a `UiAmount` of tokens to a little-endian `u64` raw Amount,
/// using the given mint.
///
/// Return data can be fetched using `sol_get_return_data` and deserializing
/// the return data as a little-endian `u64`.
///
/// WARNING: For mints using the interest-bearing or scaled-ui-amount
/// extensions, this instruction uses standard floating-point arithmetic to
/// convert values, which is not guaranteed to give consistent behavior.
///
/// In particular, conversions will not always work in reverse. For example,
/// if you pass amount `A` to `UiAmountToAmount` and receive `B`, and pass
/// the result `B` to `AmountToUiAmount`, you will not always get back `A`.
///
/// Accounts expected by this instruction:
///
///   0. `[]` The mint to calculate for.
pub type UiAmountToAmount<'account, 'amount> =
    pinocchio_token::instructions::ui_amount_to_amount::UiAmountToAmount<
        'account,
        'amount,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::unwrap_lamports::UnwrapLamports<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

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
    pinocchio_token::instructions::withdraw_excess_lamports::WithdrawExcessLamports<
        'account,
        'multisig,
        MultisigSigner,
        Token2022Program,
    >;

#[cold]
fn invalid_argument_error() -> ProgramError {
    ProgramError::InvalidArgument
}

/// Write the extensions to the instruction data buffer.
///
/// # Safety
///
/// The caller must ensure that `instruction_data` is at least `1 +
/// extensions.len() * 2` bytes long, and that `extensions.len() <=
/// MAX_EXTENSION_COUNT`.
#[inline(always)]
unsafe fn write_extension_types_instruction_data(
    instruction_data: &mut [MaybeUninit<u8>],
    discriminator: u8,
    extensions: &[ExtensionType],
) {
    debug_assert!(extensions.len() <= MAX_EXTENSION_COUNT);

    instruction_data[0].write(discriminator);

    for (i, extension) in extensions.iter().enumerate() {
        let offset = 1 + i * 2;
        let extension_type = (*extension as u16).to_le_bytes();
        // SAFETY: The caller has ensured that `instruction_data` is large enough
        // to hold all extension types.
        unsafe {
            instruction_data
                .get_unchecked_mut(offset)
                .write(extension_type[0]);
            instruction_data
                .get_unchecked_mut(offset + 1)
                .write(extension_type[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{state::ExtensionType, UNINIT_BYTE},
    };

    #[test]
    fn write_extension_types_instruction_data_with_empty_extensions() {
        let mut instruction_data = [UNINIT_BYTE; EXTENSION_TYPES_INSTRUCTION_DATA_LEN];

        // SAFETY: `instruction_data` is initialized to be
        // `EXTENSION_TYPES_INSTRUCTION_DATA_LEN` bytes long.
        unsafe { write_extension_types_instruction_data(&mut instruction_data, 21, &[]) };

        // SAFETY: the helper initialized the single discriminator byte.
        let bytes =
            unsafe { core::slice::from_raw_parts(instruction_data.as_ptr().cast::<u8>(), 1) };

        assert_eq!(bytes, &[21]);
    }

    #[test]
    fn write_extension_types_instruction_data_at_max_extension_count() {
        const EXTENSIONS: [ExtensionType; MAX_EXTENSION_COUNT] = [
            ExtensionType::TransferFeeConfig,
            ExtensionType::TransferFeeAmount,
            ExtensionType::MintCloseAuthority,
            ExtensionType::ConfidentialTransferMint,
            ExtensionType::ConfidentialTransferAccount,
            ExtensionType::DefaultAccountState,
            ExtensionType::ImmutableOwner,
            ExtensionType::MemoTransfer,
            ExtensionType::NonTransferable,
            ExtensionType::InterestBearingConfig,
            ExtensionType::CpiGuard,
            ExtensionType::PermanentDelegate,
            ExtensionType::NonTransferableAccount,
            ExtensionType::TransferHook,
            ExtensionType::TransferHookAccount,
            ExtensionType::ConfidentialTransferFeeConfig,
            ExtensionType::ConfidentialTransferFeeAmount,
            ExtensionType::MetadataPointer,
            ExtensionType::TokenMetadata,
            ExtensionType::GroupPointer,
            ExtensionType::TokenGroup,
            ExtensionType::GroupMemberPointer,
            ExtensionType::TokenGroupMember,
            ExtensionType::ConfidentialMintBurn,
            ExtensionType::ScaledUiAmount,
            ExtensionType::Pausable,
            ExtensionType::PausableAccount,
            ExtensionType::PermissionedBurn,
        ];

        let mut instruction_data = [UNINIT_BYTE; EXTENSION_TYPES_INSTRUCTION_DATA_LEN];

        // SAFETY: `instruction_data` is initialized to be
        // `EXTENSION_TYPES_INSTRUCTION_DATA_LEN` bytes long and
        // `EXTENSIONS.len() <= MAX_EXTENSION_COUNT`.
        unsafe { write_extension_types_instruction_data(&mut instruction_data, 21, &EXTENSIONS) };

        // SAFETY: the helper initialized all `1 + MAX_EXTENSION_COUNT * 2` bytes.
        let bytes = unsafe {
            core::slice::from_raw_parts(
                instruction_data.as_ptr().cast::<u8>(),
                instruction_data.len(),
            )
        };

        #[rustfmt::skip]
        let expected: [u8; EXTENSION_TYPES_INSTRUCTION_DATA_LEN] = [
            21,
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 0, 10, 0,
            11, 0, 12, 0, 13, 0, 14, 0, 15, 0, 16, 0, 17, 0, 18, 0, 19, 0, 20, 0,
            21, 0, 22, 0, 23, 0, 24, 0, 25, 0, 26, 0, 27, 0, 28, 0,
        ];

        assert_eq!(bytes, &expected);
    }
}
