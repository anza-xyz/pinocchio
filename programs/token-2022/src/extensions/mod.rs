use crate::from_bytes;
use pinocchio::program_error::ProgramError;

/// Number of padding bytes at the start of the extension area
/// before the first extension begins.
pub const EXTENSION_START_PADDING: usize = 1;

/// Size (in bytes) of the `extension length` field in the serialized account data.
pub const EXTENSION_LEN_BYTES_LEN: usize = 2;

/// Size (in bytes) of the `extension type` field in the serialized account data.
pub const EXTENSION_TYPE_BYTES_LEN: usize = 2;

/// Size (in bytes) of the extension header (type and length).
pub const EXTENSION_HEADER_LEN: usize = EXTENSION_TYPE_BYTES_LEN + EXTENSION_LEN_BYTES_LEN;

#[repr(u16)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ExtensionType {
    /// Used as padding if the account size would otherwise be 355, same as a
    /// multisig
    Uninitialized,
    /// Includes transfer fee rate info and accompanying authorities to withdraw
    /// and set the fee
    TransferFeeConfig,
    /// Includes withheld transfer fees
    TransferFeeAmount,
    /// Includes an optional mint close authority
    MintCloseAuthority,
    /// Auditor configuration for confidential transfers
    ConfidentialTransferMint,
    /// State for confidential transfers
    ConfidentialTransferAccount,
    /// Specifies the default Account::state for new Accounts
    DefaultAccountState,
    /// Indicates that the Account owner authority cannot be changed
    ImmutableOwner,
    /// Require inbound transfers to have memo
    MemoTransfer,
    /// Indicates that the tokens from this mint can't be transferred
    NonTransferable,
    /// Tokens accrue interest over time,
    InterestBearingConfig,
    /// Locks privileged token operations from happening via CPI
    CpiGuard,
    /// Includes an optional permanent delegate
    PermanentDelegate,
    /// Indicates that the tokens in this account belong to a non-transferable
    /// mint
    NonTransferableAccount,
    /// Mint requires a CPI to a program implementing the "transfer hook"
    /// interface
    TransferHook,
    /// Indicates that the tokens in this account belong to a mint with a
    /// transfer hook
    TransferHookAccount,
    /// Includes encrypted withheld fees and the encryption public that they are
    /// encrypted under
    ConfidentialTransferFeeConfig,
    /// Includes confidential withheld transfer fees
    ConfidentialTransferFeeAmount,
    /// Mint contains a pointer to another account (or the same account) that
    /// holds metadata
    MetadataPointer,
    /// Mint contains token-metadata
    TokenMetadata,
    /// Mint contains a pointer to another account (or the same account) that
    /// holds group configurations
    GroupPointer,
    /// Mint contains token group configurations
    TokenGroup,
    /// Mint contains a pointer to another account (or the same account) that
    /// holds group member configurations
    GroupMemberPointer,
    /// Mint contains token group member configurations
    TokenGroupMember,
    /// Mint allowing the minting and burning of confidential tokens
    ConfidentialMintBurn,
    /// Tokens whose UI amount is scaled by a given amount
    ScaledUiAmount,
    /// Tokens where minting / burning / transferring can be paused
    Pausable,
    /// Indicates that the account belongs to a pa-usable mint
    PausableAccount,
}

impl ExtensionType {
    #[inline(always)]
    fn from_bytes(val: [u8; 2]) -> Option<Self> {
        match u16::from_le_bytes(val) {
            0 => Some(Self::Uninitialized),
            1 => Some(Self::TransferFeeConfig),
            2 => Some(Self::TransferFeeAmount),
            3 => Some(Self::MintCloseAuthority),
            4 => Some(Self::ConfidentialTransferMint),
            5 => Some(Self::ConfidentialTransferAccount),
            6 => Some(Self::DefaultAccountState),
            7 => Some(Self::ImmutableOwner),
            8 => Some(Self::MemoTransfer),
            9 => Some(Self::NonTransferable),
            10 => Some(Self::InterestBearingConfig),
            11 => Some(Self::CpiGuard),
            12 => Some(Self::PermanentDelegate),
            13 => Some(Self::NonTransferableAccount),
            14 => Some(Self::TransferHook),
            15 => Some(Self::TransferHookAccount),
            16 => Some(Self::ConfidentialTransferFeeConfig),
            17 => Some(Self::ConfidentialTransferFeeAmount),
            18 => Some(Self::MetadataPointer),
            19 => Some(Self::TokenMetadata),
            20 => Some(Self::GroupPointer),
            21 => Some(Self::TokenGroup),
            22 => Some(Self::GroupMemberPointer),
            23 => Some(Self::TokenGroupMember),
            24 => Some(Self::ConfidentialMintBurn),
            25 => Some(Self::ScaledUiAmount),
            26 => Some(Self::Pausable),
            27 => Some(Self::PausableAccount),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; 2] {
        u16::to_le_bytes(*self as u16)
    }
}

/// Represents the type of "base state" for the account
/// (either a Mint account or a Token-Account).
#[derive(PartialEq, Eq)]
pub enum BaseStateVariant {
    Mint,
    TokenAccount,
}

/// Trait that must be implemented by Token-2022 state
/// which support extensions (Mint, Token-Account)
pub trait BaseState {
    /// Which base state variant this type represents.
    const BASE_STATE: BaseStateVariant;

    /// Size in bytes of the base account data (without extensions).
    const LEN: usize;

    /// Where extension data starts (offset into account data).
    fn extension_data_start_index() -> Option<usize>;
}

/// Describes whether an extension has a fixed or variable size.
pub enum ExtensionLen {
    FixedSize(u16),
    VariableSize,
}

/// Trait implemented by all Token-2022 extensions.
/// Defines extension type, size, and which base state it belongs to.
pub trait Extension {
    const TYPE: ExtensionType;
    const LEN: ExtensionLen;
    const BASE_STATE: BaseStateVariant;
}

/// Reads an extension of type `E` from the provided account data bytes
/// corresponding to base state B.
///
/// Safety: Uses `from_bytes` to cast extension bytes into a reference to Extension E.
#[inline]
pub fn get_extension_from_acc_data<E: Extension, B: BaseState>(
    acc_data_bytes: &[u8],
) -> Result<Option<&E>, ProgramError> {
    // Ensure extension is only searched in its matching base state.
    if B::BASE_STATE != E::BASE_STATE {
        return Ok(None);
    }

    // Get where extension data begins.
    let ext_data_start_index = match B::extension_data_start_index() {
        Some(idx) => idx,
        None => return Ok(None),
    };

    // Slice out the extension area from account data.
    let ext_bytes = acc_data_bytes
        .get(ext_data_start_index..)
        .ok_or(ProgramError::AccountDataTooSmall)?;

    let mut start = 0;
    let end = ext_bytes.len();

    // Walk through each extension in the serialized list.
    while start < end {
        let ext_type_idx = start;
        let ext_len_idx = ext_type_idx + EXTENSION_TYPE_BYTES_LEN;
        let ext_data_idx = ext_len_idx + EXTENSION_LEN_BYTES_LEN;

        // Parse extension type.
        let ext_type = ext_bytes[ext_type_idx..(ext_type_idx + EXTENSION_TYPE_BYTES_LEN)]
            .try_into()
            .map(ExtensionType::from_bytes)
            .map_err(|_| ProgramError::InvalidAccountData)?
            .ok_or(ProgramError::InvalidAccountData)?;

        // Parse extension length.
        let ext_len = ext_bytes[ext_len_idx..(ext_len_idx + EXTENSION_LEN_BYTES_LEN)]
            .try_into()
            .map(u16::from_le_bytes)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        // Match ext len and check if this is the extension we’re looking for, then return it.
        match E::LEN {
            ExtensionLen::FixedSize(size) => {
                if ext_type == E::TYPE && ext_len == size {
                    return Ok(Some(unsafe {
                        from_bytes(&ext_bytes[ext_data_idx..ext_data_idx + size as usize])
                    }));
                }
            }
            ExtensionLen::VariableSize => {
                if ext_type == E::TYPE {
                    return Ok(Some(unsafe {
                        from_bytes(&ext_bytes[ext_data_idx..ext_data_idx + ext_len as usize])
                    }));
                }
            }
        }

        // Advance to next extension in the serialized account data bytes
        start += EXTENSION_HEADER_LEN + ext_len as usize;
    }

    Ok(None)
}
