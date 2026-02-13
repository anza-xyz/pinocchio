pub mod default_account_state;
pub mod permanent_delegate;
mod state;
pub mod transfer_hook;
pub mod transfer_hook_account;

use {
    super::{AccountType, Mint, Multisig, TokenAccount},
    solana_program_error::ProgramError,
};
pub use {
    default_account_state::DefaultAccountStateExtension,
    permanent_delegate::PermanentDelegateExtension,
    state::{
        RefMutStateWithExtensions, RefStateWithExtensions, StateWithExtensions,
        StateWithExtensionsMut,
    },
    transfer_hook::TransferHookExtension,
    transfer_hook_account::TransferHookAccountExtension,
};

/// Maximum number of distinct extension types (excluding `Uninitialized`).
///
/// Useful for pre-allocating the output buffer when calling
/// `collect_extension_types_from_tlv` or the wrapper `get_extension_types`
/// methods.
pub const MAX_EXTENSIONS: usize = 28;

const BASE_ACCOUNT_LEN: usize = 165;
const MINT_PADDING_LEN: usize = BASE_ACCOUNT_LEN - Mint::BASE_LEN;
const ZERO_MINT_PADDING: [u8; MINT_PADDING_LEN] = [0u8; MINT_PADDING_LEN];
const ACCOUNT_TYPE_INDEX: usize = BASE_ACCOUNT_LEN;
const TLV_START_INDEX: usize = ACCOUNT_TYPE_INDEX + 1;
const TLV_HEADER_LEN: usize = 4;

/// Token-2022 error discriminants used by this state module.
///
/// Keep these values aligned with SPL Token-2022's `TokenError`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenError {
    /// Extension not found in account data.
    ExtensionNotFound = 48,
}

impl From<TokenError> for ProgramError {
    #[inline(always)]
    fn from(error: TokenError) -> Self {
        ProgramError::Custom(error as u32)
    }
}

/// SPL Token-2022 `TokenError::ExtensionNotFound` discriminant.
pub const EXTENSION_NOT_FOUND_ERROR_CODE: u32 = TokenError::ExtensionNotFound as u32;

mod sealed {
    pub trait SealedPod {}
}

/// Marker trait for plain extension payload values that are safe to
/// reinterpret from bytes.
///
/// # Safety
///
/// Implementors must be plain data with no padding and no invalid bit-patterns.
pub unsafe trait Pod: sealed::SealedPod + Copy + 'static {}

#[inline(always)]
pub const fn extension_not_found_error() -> ProgramError {
    ProgramError::Custom(TokenError::ExtensionNotFound as u32)
}

#[inline(always)]
pub fn is_extension_not_found_error(error: &ProgramError) -> bool {
    matches!(
        error,
        ProgramError::Custom(code) if *code == EXTENSION_NOT_FOUND_ERROR_CODE
    )
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ExtensionType {
    Uninitialized = 0,
    TransferFeeConfig = 1,
    TransferFeeAmount = 2,
    MintCloseAuthority = 3,
    ConfidentialTransferMint = 4,
    ConfidentialTransferAccount = 5,
    DefaultAccountState = 6,
    ImmutableOwner = 7,
    MemoTransfer = 8,
    NonTransferable = 9,
    InterestBearingConfig = 10,
    CpiGuard = 11,
    PermanentDelegate = 12,
    NonTransferableAccount = 13,
    TransferHook = 14,
    TransferHookAccount = 15,
    ConfidentialTransferFeeConfig = 16,
    ConfidentialTransferFeeAmount = 17,
    MetadataPointer = 18,
    TokenMetadata = 19,
    GroupPointer = 20,
    TokenGroup = 21,
    GroupMemberPointer = 22,
    TokenGroupMember = 23,
    ConfidentialMintBurn = 24,
    ScaledUiAmount = 25,
    Pausable = 26,
    PausableAccount = 27,
}

/// Marker for typed extension values that can be decoded from TLV entries.
pub trait ExtensionValue: Pod {
    const TYPE: ExtensionType;
}

/// Trait for supported token-2022 base account types that can host TLV
/// extensions.
pub trait ExtensionBaseState: Sized {
    const BASE_LEN: usize;
    const ACCOUNT_TYPE: AccountType;

    fn validate_extensions_data(data: &[u8]) -> Result<(), ProgramError>;

    /// # Safety
    ///
    /// The caller must ensure the provided bytes contain a valid base state.
    unsafe fn from_bytes_unchecked(data: &[u8]) -> &Self;

    /// # Safety
    ///
    /// The caller must ensure the provided bytes contain a valid mutable base
    /// state.
    unsafe fn from_bytes_unchecked_mut(data: &mut [u8]) -> &mut Self;
}

#[inline(always)]
const fn extension_account_type(extension_type: ExtensionType) -> AccountType {
    match extension_type {
        ExtensionType::Uninitialized => AccountType::Uninitialized,
        ExtensionType::TransferFeeConfig
        | ExtensionType::MintCloseAuthority
        | ExtensionType::ConfidentialTransferMint
        | ExtensionType::DefaultAccountState
        | ExtensionType::NonTransferable
        | ExtensionType::InterestBearingConfig
        | ExtensionType::PermanentDelegate
        | ExtensionType::TransferHook
        | ExtensionType::ConfidentialTransferFeeConfig
        | ExtensionType::MetadataPointer
        | ExtensionType::TokenMetadata
        | ExtensionType::GroupPointer
        | ExtensionType::TokenGroup
        | ExtensionType::GroupMemberPointer
        | ExtensionType::TokenGroupMember
        | ExtensionType::ConfidentialMintBurn
        | ExtensionType::ScaledUiAmount
        | ExtensionType::Pausable => AccountType::Mint,
        ExtensionType::TransferFeeAmount
        | ExtensionType::ConfidentialTransferAccount
        | ExtensionType::ImmutableOwner
        | ExtensionType::MemoTransfer
        | ExtensionType::CpiGuard
        | ExtensionType::NonTransferableAccount
        | ExtensionType::TransferHookAccount
        | ExtensionType::ConfidentialTransferFeeAmount
        | ExtensionType::PausableAccount => AccountType::Account,
    }
}

#[inline(always)]
fn extension_type_from_u16(extension_type: u16) -> Result<ExtensionType, ProgramError> {
    match extension_type {
        0 => Ok(ExtensionType::Uninitialized),
        1 => Ok(ExtensionType::TransferFeeConfig),
        2 => Ok(ExtensionType::TransferFeeAmount),
        3 => Ok(ExtensionType::MintCloseAuthority),
        4 => Ok(ExtensionType::ConfidentialTransferMint),
        5 => Ok(ExtensionType::ConfidentialTransferAccount),
        6 => Ok(ExtensionType::DefaultAccountState),
        7 => Ok(ExtensionType::ImmutableOwner),
        8 => Ok(ExtensionType::MemoTransfer),
        9 => Ok(ExtensionType::NonTransferable),
        10 => Ok(ExtensionType::InterestBearingConfig),
        11 => Ok(ExtensionType::CpiGuard),
        12 => Ok(ExtensionType::PermanentDelegate),
        13 => Ok(ExtensionType::NonTransferableAccount),
        14 => Ok(ExtensionType::TransferHook),
        15 => Ok(ExtensionType::TransferHookAccount),
        16 => Ok(ExtensionType::ConfidentialTransferFeeConfig),
        17 => Ok(ExtensionType::ConfidentialTransferFeeAmount),
        18 => Ok(ExtensionType::MetadataPointer),
        19 => Ok(ExtensionType::TokenMetadata),
        20 => Ok(ExtensionType::GroupPointer),
        21 => Ok(ExtensionType::TokenGroup),
        22 => Ok(ExtensionType::GroupMemberPointer),
        23 => Ok(ExtensionType::TokenGroupMember),
        24 => Ok(ExtensionType::ConfidentialMintBurn),
        25 => Ok(ExtensionType::ScaledUiAmount),
        26 => Ok(ExtensionType::Pausable),
        27 => Ok(ExtensionType::PausableAccount),
        _ => Err(ProgramError::InvalidAccountData),
    }
}

#[inline(always)]
const fn adjust_len_for_multisig(account_len: usize) -> usize {
    if account_len == Multisig::LEN {
        account_len.saturating_add(core::mem::size_of::<ExtensionType>())
    } else {
        account_len
    }
}

#[inline(always)]
fn validate_extension_account_type(
    extension_type: ExtensionType,
    expected: AccountType,
) -> Result<(), ProgramError> {
    if extension_account_type(extension_type) != expected {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

#[inline]
fn validate_mint_extensions_data(data: &[u8]) -> Result<(), ProgramError> {
    if data.len() <= ACCOUNT_TYPE_INDEX {
        return Err(ProgramError::InvalidAccountData);
    }

    if data[Mint::BASE_LEN..BASE_ACCOUNT_LEN] != ZERO_MINT_PADDING {
        return Err(ProgramError::InvalidAccountData);
    }

    if data[ACCOUNT_TYPE_INDEX] != AccountType::Mint as u8 {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

#[inline]
fn validate_token_extensions_data(data: &[u8]) -> Result<(), ProgramError> {
    if data.len() <= ACCOUNT_TYPE_INDEX {
        return Err(ProgramError::InvalidAccountData);
    }

    if data[ACCOUNT_TYPE_INDEX] != AccountType::Account as u8 {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

#[inline(always)]
unsafe fn mint_from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Mint {
    &mut *(bytes[..Mint::BASE_LEN].as_mut_ptr() as *mut Mint)
}

#[inline(always)]
unsafe fn token_account_from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut TokenAccount {
    &mut *(bytes[..TokenAccount::BASE_LEN].as_mut_ptr() as *mut TokenAccount)
}

/// Returns the fixed byte length of the given extension's value payload,
/// or `None` if the extension type is not yet supported.
///
/// Only a subset of extension types have known sizes registered here.
/// Unsupported types will cause [`try_calculate_account_len`] to fail.
#[inline(always)]
pub const fn extension_value_len(extension_type: ExtensionType) -> Option<usize> {
    match extension_type {
        ExtensionType::DefaultAccountState => Some(DefaultAccountStateExtension::LEN),
        ExtensionType::PermanentDelegate => Some(PermanentDelegateExtension::LEN),
        ExtensionType::TransferHook => Some(TransferHookExtension::LEN),
        ExtensionType::TransferHookAccount => Some(TransferHookAccountExtension::LEN),
        _ => None,
    }
}

#[inline]
pub fn try_calculate_account_len<B: ExtensionBaseState>(
    extension_types: &[ExtensionType],
) -> Result<usize, ProgramError> {
    if extension_types.is_empty() {
        return Ok(B::BASE_LEN);
    }

    let mut total_len = TLV_START_INDEX;
    let mut i = 0;

    while i < extension_types.len() {
        let extension_type = extension_types[i];
        validate_extension_account_type(extension_type, B::ACCOUNT_TYPE)?;

        let mut j = 0;
        let mut duplicate = false;
        while j < i {
            if extension_types[j] == extension_type {
                duplicate = true;
                break;
            }
            j += 1;
        }
        if duplicate {
            i += 1;
            continue;
        }

        let value_len =
            extension_value_len(extension_type).ok_or(ProgramError::InvalidInstructionData)?;

        total_len = total_len
            .checked_add(TLV_HEADER_LEN + value_len)
            .ok_or(ProgramError::InvalidInstructionData)?;
        i += 1;
    }

    Ok(adjust_len_for_multisig(total_len))
}
