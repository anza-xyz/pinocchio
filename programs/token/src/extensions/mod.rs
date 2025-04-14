use crate::{
    from_bytes_ref,
    state::{Mint, TokenAccount},
};
// pub mod confidential_transfer;
pub mod confidential_transfer;
pub mod cpi_guard;
pub mod default_account_state;
pub mod group_member_pointer;
pub mod group_pointer;
pub mod immutable_owner;
pub mod interest_bearing_mint;
pub mod memo_transfer;
pub mod metadata;
pub mod metadata_pointer;
pub mod mint_close_authority;
pub mod non_transferable;
pub mod pausable;
pub mod permanent_delegate;
pub mod token_group;
pub mod transfer_fee;

pub const ELGAMAL_PUBKEY_LEN: usize = 32;

pub const POD_AE_CIPHERTEXT_LEN: usize = 36;
pub const POD_ELGAMAL_CIPHERTEXT_LEN: usize = 64;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElagamalPubkey(pub [u8; ELGAMAL_PUBKEY_LEN]);

pub const EXTENSIONS_PADDING: usize = 83;

pub const EXTENSION_START_OFFSET: usize = 1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Indicates that the account belongs to a pausable mint
    PausableAccount,
}

impl ExtensionType {
    fn from_bytes(val: [u8; 2]) -> Option<Self> {
        let val = u16::from_le_bytes(val);
        let ext = match val {
            0 => ExtensionType::Uninitialized,
            1 => ExtensionType::TransferFeeConfig,
            2 => ExtensionType::TransferFeeAmount,
            3 => ExtensionType::MintCloseAuthority,
            4 => ExtensionType::ConfidentialTransferMint,
            5 => ExtensionType::ConfidentialTransferAccount,
            6 => ExtensionType::DefaultAccountState,
            7 => ExtensionType::ImmutableOwner,
            8 => ExtensionType::MemoTransfer,
            9 => ExtensionType::NonTransferable,
            10 => ExtensionType::InterestBearingConfig,
            11 => ExtensionType::CpiGuard,
            12 => ExtensionType::PermanentDelegate,
            13 => ExtensionType::NonTransferableAccount,
            14 => ExtensionType::TransferHook,
            15 => ExtensionType::TransferHookAccount,
            16 => ExtensionType::ConfidentialTransferFeeConfig,
            17 => ExtensionType::ConfidentialTransferFeeAmount,
            18 => ExtensionType::MetadataPointer,
            19 => ExtensionType::TokenMetadata,
            20 => ExtensionType::GroupPointer,
            21 => ExtensionType::TokenGroup,
            22 => ExtensionType::GroupMemberPointer,
            23 => ExtensionType::TokenGroupMember,
            24 => ExtensionType::ConfidentialMintBurn,
            25 => ExtensionType::ScaledUiAmount,
            26 => ExtensionType::Pausable,
            27 => ExtensionType::PausableAccount,
            _ => return None,
        };
        Some(ext)
    }
}

pub const EXTENSION_LENGTH_LEN: usize = 2;
pub const EXTENSION_TYPE_LEN: usize = 2;

pub enum BaseState {
    Mint,
    TokenAccount,
}

pub trait Extension {
    const TYPE: ExtensionType;
    const LEN: usize;
    const BASE_STATE: BaseState;
}

pub fn get_extension_from_bytes<T: Extension + Clone + Copy>(acc_data_bytes: &[u8]) -> Option<&T> {
    let ext_bytes = match T::BASE_STATE {
        BaseState::Mint => {
            &acc_data_bytes[Mint::LEN + EXTENSIONS_PADDING + EXTENSION_START_OFFSET..]
        }
        BaseState::TokenAccount => &acc_data_bytes[TokenAccount::LEN + EXTENSION_START_OFFSET..],
    };
    let mut start = 0;
    let end = ext_bytes.len();
    while start < end {
        let ext_type_idx = start;
        let ext_len_idx = ext_type_idx + 2;
        let ext_data_idx = ext_len_idx + EXTENSION_LENGTH_LEN;

        let ext_type: [u8; 2] = ext_bytes[ext_type_idx..ext_type_idx + EXTENSION_TYPE_LEN]
            .try_into()
            .ok()?;
        let ext_type = ExtensionType::from_bytes(ext_type)?;
        let ext_len: [u8; 2] = ext_bytes[ext_len_idx..ext_len_idx + EXTENSION_LENGTH_LEN]
            .try_into()
            .ok()?;

        let ext_len = u16::from_le_bytes(ext_len);

        if ext_type == T::TYPE && ext_len as usize == T::LEN {
            return Some(unsafe {
                from_bytes_ref(&ext_bytes[ext_data_idx..ext_data_idx + T::LEN])
            });
        }

        start = start + EXTENSION_TYPE_LEN + EXTENSION_LENGTH_LEN + ext_len as usize;
    }
    None
}

pub fn get_extension_data_bytes_for_variable_pack<T: Extension + Clone>(
    acc_data_bytes: &[u8],
) -> Option<&[u8]> {
    let ext_bytes = match T::BASE_STATE {
        BaseState::Mint => {
            &acc_data_bytes[Mint::LEN + EXTENSIONS_PADDING + EXTENSION_START_OFFSET..]
        }
        BaseState::TokenAccount => &acc_data_bytes[TokenAccount::LEN + EXTENSION_START_OFFSET..],
    };
    let mut start = 0;
    let end = ext_bytes.len();
    while start < end {
        let ext_type_idx = start;
        let ext_len_idx = ext_type_idx + 2;
        let ext_data_idx = ext_len_idx + EXTENSION_LENGTH_LEN;

        let ext_type: [u8; 2] = ext_bytes[ext_type_idx..ext_type_idx + EXTENSION_TYPE_LEN]
            .try_into()
            .ok()?;

        let ext_type = ExtensionType::from_bytes(ext_type)?;
        let ext_len: [u8; 2] = ext_bytes[ext_len_idx..ext_len_idx + EXTENSION_LENGTH_LEN]
            .try_into()
            .ok()?;

        let ext_len = u16::from_le_bytes(ext_len);

        if ext_type == T::TYPE {
            return Some(&ext_bytes[ext_data_idx..ext_data_idx + ext_len as usize]);
        }

        start = start + EXTENSION_TYPE_LEN + EXTENSION_LENGTH_LEN + ext_len as usize;
    }
    None
}
#[cfg(test)]
mod tests {
    use crate::extensions::{
        confidential_transfer::{ConfidentialTransferFeeConfig, ConfidentialTransferMint},
        get_extension_from_bytes,
        group_member_pointer::GroupMemberPointer,
        group_pointer::GroupPointer,
        metadata_pointer::MetadataPointer,
        mint_close_authority::MintCloseAuthority,
        permanent_delegate::PermanentDelegate,
        token_group::TokenGroup,
        transfer_fee::TransferFeeConfig,
    };

    pub const TEST_MINT_WITH_EXTENSIONS_SLICE: &[u8] = &[
        1, 0, 0, 0, 221, 76, 72, 108, 144, 248, 182, 240, 7, 195, 4, 239, 36, 129, 248, 5, 24, 107,
        232, 253, 95, 82, 172, 209, 2, 92, 183, 155, 159, 103, 255, 33, 133, 204, 6, 44, 35, 140,
        0, 0, 6, 1, 1, 0, 0, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173,
        49, 41, 63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        /*                  MintCloseAuthority Extension                                      */
        3, 0, 32, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41, 63,
        207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43,
        /*                  PermanentDelegate Extension                                      */
        12, 0, 32, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41,
        63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43,
        /*                  TransferFeeConfig Extension                                      */
        1, 0, 108, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41,
        63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 23, 133, 50, 97, 239, 106,
        184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41, 63, 207, 7, 207, 18, 10, 181, 185, 161,
        87, 6, 84, 141, 192, 43, 0, 0, 0, 0, 0, 0, 0, 0, 93, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 93, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        /*                  ConfidentialTransferMint Extension                                      */
        4, 0, 65, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41, 63,
        207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        /*                  ConfidentialTransferFeeConfig Extension                                      */
        16, 0, 129, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41,
        63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 28, 55, 230, 67, 59, 115,
        4, 221, 130, 115, 122, 228, 13, 155, 139, 243, 196, 159, 91, 14, 108, 73, 168, 213, 51, 40,
        179, 229, 6, 144, 28, 87, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        /*                  TransferHook Extension                                      */
        14, 0, 64, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41,
        63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        /*                  MetadataPointer Extension                                      */
        18, 0, 64, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41,
        63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 23, 146, 72, 59, 108, 138,
        42, 135, 183, 71, 29, 129, 79, 149, 145, 249, 57, 92, 132, 10, 156, 227, 217, 244, 213,
        186, 125, 58, 75, 138, 116, 158,
        /*                  TokenMetadata Extension                                      */
        19, 0, 174, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173, 49, 41,
        63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 23, 146, 72, 59, 108, 138,
        42, 135, 183, 71, 29, 129, 79, 149, 145, 249, 57, 92, 132, 10, 156, 227, 217, 244, 213,
        186, 125, 58, 75, 138, 116, 158, 10, 0, 0, 0, 80, 97, 121, 80, 97, 108, 32, 85, 83, 68, 5,
        0, 0, 0, 80, 89, 85, 83, 68, 79, 0, 0, 0, 104, 116, 116, 112, 115, 58, 47, 47, 116, 111,
        107, 101, 110, 45, 109, 101, 116, 97, 100, 97, 116, 97, 46, 112, 97, 120, 111, 115, 46, 99,
        111, 109, 47, 112, 121, 117, 115, 100, 95, 109, 101, 116, 97, 100, 97, 116, 97, 47, 112,
        114, 111, 100, 47, 115, 111, 108, 97, 110, 97, 47, 112, 121, 117, 115, 100, 95, 109, 101,
        116, 97, 100, 97, 116, 97, 46, 106, 115, 111, 110, 0, 0, 0, 0,
        /*                  GroupPointer Extension                                      */
        20, 0, 64, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2,
        /*                  TokenGroup Extension                                      */
        21, 0, 80, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
    ];
    pub const TEST_GROUP_MEMBER_MINT_SLICE: &[u8] = &[
        1, 0, 0, 0, 221, 76, 72, 108, 144, 248, 182, 240, 7, 195, 4, 239, 36, 129, 248, 5, 24, 107,
        232, 253, 95, 82, 172, 209, 2, 92, 183, 155, 159, 103, 255, 33, 133, 204, 6, 44, 35, 140,
        0, 0, 6, 1, 1, 0, 0, 0, 23, 133, 50, 97, 239, 106, 184, 83, 42, 103, 240, 83, 134, 90, 173,
        49, 41, 63, 207, 7, 207, 18, 10, 181, 185, 161, 87, 6, 84, 141, 192, 43, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        /*                  GroupMemberPointer Extension                                      */
        22, 0, 64, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2,
    ];

    #[test]
    fn test_transfer_fee_config() {
        let transfer_fee =
            get_extension_from_bytes::<TransferFeeConfig>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(transfer_fee.is_some());
    }

    #[test]
    fn test_metadata_pointer() {
        let metadata_pointer =
            get_extension_from_bytes::<MetadataPointer>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(metadata_pointer.is_some());
    }

    #[test]
    fn test_mint_close_authority() {
        let mint_close_authority =
            get_extension_from_bytes::<MintCloseAuthority>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(mint_close_authority.is_some());
    }

    #[test]
    fn test_permanent_delegate() {
        let permanent_delegate =
            get_extension_from_bytes::<PermanentDelegate>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(permanent_delegate.is_some());
    }

    #[test]
    fn test_group_pointer() {
        let group_pointer =
            get_extension_from_bytes::<GroupPointer>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(group_pointer.is_some());
        let gp = group_pointer.unwrap();
        assert!(gp.authority.eq(&[1u8; 32]));
        assert!(gp.group_address.eq(&[2u8; 32]));
    }

    #[test]
    fn test_token_group() {
        let token_group = get_extension_from_bytes::<TokenGroup>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(token_group.is_some());
        let tg = token_group.unwrap();
        assert!(tg.update_authority.eq(&[1u8; 32]));
        assert!(tg.mint.eq(&[2u8; 32]));
        assert_eq!(u64::from_le_bytes(tg.size), 1);
        assert_eq!(u64::from_le_bytes(tg.max_size), 2);
    }

    #[test]
    fn test_group_member_pointer() {
        let group_member_pointer =
            get_extension_from_bytes::<GroupMemberPointer>(&TEST_GROUP_MEMBER_MINT_SLICE);
        assert!(group_member_pointer.is_some());
        let gmp = group_member_pointer.unwrap();
        assert!(gmp.authority.eq(&[1u8; 32]));
        assert!(gmp.member_address.eq(&[2u8; 32]));
    }

    #[test]
    fn test_confidential_transfer_mint() {
        let confidential_transfer_mint =
            get_extension_from_bytes::<ConfidentialTransferMint>(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(confidential_transfer_mint.is_some());
    }

    #[test]
    fn test_confidential_transfer_fee_config() {
        let confidential_transfer_fee_config = get_extension_from_bytes::<
            ConfidentialTransferFeeConfig,
        >(&TEST_MINT_WITH_EXTENSIONS_SLICE);
        assert!(confidential_transfer_fee_config.is_some());
    }

    #[test]
    fn test_token_metadata() {
        use crate::extensions::get_extension_data_bytes_for_variable_pack;
        use crate::extensions::metadata::TokenMetadata;

        let token_metadata = get_extension_data_bytes_for_variable_pack::<TokenMetadata>(
            &TEST_MINT_WITH_EXTENSIONS_SLICE,
        )
        .unwrap();

        let token_metadata = TokenMetadata::from_bytes(token_metadata);

        assert!(token_metadata.is_ok());

        let token_metadata = token_metadata.unwrap();

        assert_eq!(token_metadata.symbol, "PYUSD");
    }
}
