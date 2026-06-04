use {
    super::{
        adjust_len_for_multisig, default_account_state::DefaultAccountStateExtension,
        permanent_delegate::PermanentDelegateExtension,
        permissioned_burn::PermissionedBurnExtension, transfer_hook::TransferHookExtension,
        validate_extension_account_type, ExtensionBaseState, ExtensionType,
        ImmutableOwnerExtension, NonTransferableAccountExtension, PausableAccountExtension,
        StateWithExtensions, TransferFeeAmountExtension, TransferHookAccountExtension,
        MAX_EXTENSIONS, TLV_HEADER_LEN, TLV_START_INDEX,
    },
    crate::{
        state::{Account, AccountType, Mint},
        ID,
    },
    solana_account_view::AccountView,
    solana_program_error::ProgramError,
};

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
        ExtensionType::PermissionedBurn => Some(PermissionedBurnExtension::LEN),
        ExtensionType::TransferHook => Some(TransferHookExtension::LEN),
        ExtensionType::TransferHookAccount => Some(TransferHookAccountExtension::LEN),
        ExtensionType::ImmutableOwner => Some(ImmutableOwnerExtension::LEN),
        ExtensionType::NonTransferableAccount => Some(NonTransferableAccountExtension::LEN),
        ExtensionType::PausableAccount => Some(PausableAccountExtension::LEN),
        ExtensionType::TransferFeeAmount => Some(TransferFeeAmountExtension::LEN),
        _ => None,
    }
}

/// Returns the account data length needed for the given extension types.
///
/// Only extension types with sizes registered in [`extension_value_len`] are
/// supported.
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
        while j < i {
            if extension_types[j] == extension_type {
                return Err(ProgramError::InvalidInstructionData);
            }
            j += 1;
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

/// Given a Token-2022 extension found on a mint, returns the account extensions
/// that every token account of that mint must have. Returns `None` when the
/// input isn't a recognized mint extension.
#[inline]
pub const fn required_account_extensions_from_mint_extension(
    mint_extension_type: ExtensionType,
) -> Option<&'static [ExtensionType]> {
    match mint_extension_type {
        // Mint extensions that force every account of the mint to also carry specific extensions
        ExtensionType::TransferFeeConfig => Some(&[ExtensionType::TransferFeeAmount]),
        ExtensionType::NonTransferable => Some(&[
            ExtensionType::NonTransferableAccount,
            ExtensionType::ImmutableOwner,
        ]),
        ExtensionType::TransferHook => Some(&[ExtensionType::TransferHookAccount]),
        ExtensionType::Pausable => Some(&[ExtensionType::PausableAccount]),
        // Mint extensions that don't require anything extra on the account side
        ExtensionType::MintCloseAuthority
        | ExtensionType::ConfidentialTransferMint
        | ExtensionType::DefaultAccountState
        | ExtensionType::InterestBearingConfig
        | ExtensionType::PermanentDelegate
        | ExtensionType::ConfidentialTransferFeeConfig
        | ExtensionType::MetadataPointer
        | ExtensionType::TokenMetadata
        | ExtensionType::GroupPointer
        | ExtensionType::TokenGroup
        | ExtensionType::GroupMemberPointer
        | ExtensionType::TokenGroupMember
        | ExtensionType::ConfidentialMintBurn
        | ExtensionType::ScaledUiAmount
        | ExtensionType::PermissionedBurn => Some(&[]),
        // `Uninitialized`, account extensions, or unrecognized
        _ => None,
    }
}

// `SeenExtensions` relies on the largest discriminant staying less than 32.
// If more variants are added, this should be widen to `u64`.
const _: () = assert!(MAX_EXTENSIONS <= u32::BITS as usize);
const _: () = assert!(ExtensionType::PermissionedBurn as usize == MAX_EXTENSIONS - 1);

/// Set of extension types already counted, one bit per type. An extension's
/// discriminant is its bit position: `TransferFeeAmount` (value 2) uses bit 2,
/// `ImmutableOwner` (value 7) uses bit 7, etc.
#[derive(Default)]
struct SeenExtensions(u32);

impl SeenExtensions {
    /// Inserts `extension_type` into the seen struct. Returns `true` if it
    /// wasn't already present.
    #[inline(always)]
    fn insert(&mut self, extension_type: ExtensionType) -> bool {
        let bit = 1u32 << (extension_type as u32);
        let is_new = self.0 & bit == 0;
        self.0 |= bit;
        is_new
    }
}

/// Computes the byte length a Token-2022 account for `mint` needs, counting the
/// extensions every account of that mint requires plus any
/// `additional_account_extensions` (e.g. the `ATA` program adds
/// `ImmutableOwner`).
///
/// Returns `Err` whenever the size can't be computed locally. This can be due
/// to invalid input (wrong owner, a mint-only ext passed as an account ext,
/// non-mint data) or inputs it can't size (an extension it doesn't recognize
/// or track a size for). Callers should defer to the token program's
/// `GetAccountDataSize` if it's not recognized here.
#[inline]
pub fn try_calculate_account_len_from_mint(
    mint: &AccountView,
    additional_account_extensions: &[ExtensionType],
) -> Result<usize, ProgramError> {
    if !mint.owned_by(&ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mint_data = mint.try_borrow()?;

    let mut total_len: usize = TLV_START_INDEX;
    let mut has_extensions = false;

    // Prevents double-counting when the caller and a mint extension both require
    // the same extension.
    let mut seen = SeenExtensions::default();

    // Step 1: count whatever the caller asked us to include
    for &ext in additional_account_extensions {
        if seen.insert(ext) {
            has_extensions = true;
            validate_extension_account_type(ext, AccountType::Account)?;
            let value_len = extension_value_len(ext).ok_or(ProgramError::InvalidInstructionData)?;
            total_len = total_len
                .checked_add(TLV_HEADER_LEN + value_len)
                .ok_or(ProgramError::InvalidInstructionData)?;
        }
    }

    // Step 2: iterate over the mint's extensions and add to count

    // only parse if there is extra data present
    if mint_data.len() != Mint::BASE_LEN {
        let state = StateWithExtensions::<Mint>::from_bytes(&mint_data)?;

        // Find what mint extensions are present
        let mut mint_exts = [ExtensionType::Uninitialized; MAX_EXTENSIONS];
        let mint_ext_count = state.write_extension_types(&mut mint_exts)?;
        for &mint_ext in &mint_exts[..mint_ext_count] {
            let required = required_account_extensions_from_mint_extension(mint_ext)
                .ok_or(ProgramError::InvalidAccountData)?;
            for &ext in required {
                if seen.insert(ext) {
                    has_extensions = true;
                    let value_len =
                        extension_value_len(ext).ok_or(ProgramError::InvalidAccountData)?;
                    total_len = total_len
                        .checked_add(TLV_HEADER_LEN + value_len)
                        .ok_or(ProgramError::InvalidAccountData)?;
                }
            }
        }
    }

    if !has_extensions {
        return Ok(Account::BASE_LEN);
    }

    Ok(adjust_len_for_multisig(total_len))
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use {
        super::{
            super::{
                extension_account_type,
                shared_test_helpers::{build_account_view, build_mint_data, push_tlv_entry},
                ACCOUNT_TYPE_INDEX,
            },
            *,
        },
        alloc::{vec, vec::Vec},
        solana_address::Address,
        test_case::test_case,
    };

    // --- `SeenExtensions` helper ---

    #[test]
    fn insert_returns_true_on_first_call() {
        let mut seen = SeenExtensions::default();
        assert!(seen.insert(ExtensionType::ImmutableOwner));
    }

    #[test]
    fn insert_returns_false_when_already_seen() {
        let mut seen = SeenExtensions::default();
        seen.insert(ExtensionType::ImmutableOwner);
        assert!(!seen.insert(ExtensionType::ImmutableOwner));
    }

    #[test]
    fn duplicate_insert_is_idempotent() {
        let mut seen = SeenExtensions::default();
        seen.insert(ExtensionType::ImmutableOwner);
        let after_first = seen.0;
        seen.insert(ExtensionType::ImmutableOwner);
        assert_eq!(seen.0, after_first);
    }

    #[test]
    fn distinct_extensions_use_distinct_bits() {
        let mut seen = SeenExtensions::default();
        seen.insert(ExtensionType::TransferFeeAmount); // bit 2
        seen.insert(ExtensionType::ImmutableOwner); // bit 7
        seen.insert(ExtensionType::PausableAccount); // bit 27
        assert_eq!(seen.0, (1 << 2) | (1 << 7) | (1 << 27));
    }

    // --- `try_calculate_account_len_from_mint()` ---

    #[test]
    fn rejects_mint_with_wrong_owner() {
        let wrong_owner = Address::new_from_array([7u8; 32]);
        let data = vec![0u8; Mint::BASE_LEN];
        let (_backing, mint) = build_account_view(&wrong_owner, &data);

        assert_eq!(
            try_calculate_account_len_from_mint(&mint, &[ExtensionType::ImmutableOwner]),
            Err(ProgramError::IncorrectProgramId)
        );
    }

    #[test_case(&[], Ok(Account::BASE_LEN))] // no exts, bare account
    #[test_case(&[ExtensionType::ImmutableOwner], Ok(170))] // tracked ext is sized
    #[test_case(&[ExtensionType::ImmutableOwner, ExtensionType::ImmutableOwner], Ok(170))] // duplicates deduped
    #[test_case(&[ExtensionType::TransferFeeConfig], Err(ProgramError::InvalidAccountData))] // mint-only ext rejected
    #[test_case(&[ExtensionType::CpiGuard], Err(ProgramError::InvalidInstructionData))] // untracked ext can't be sized
    fn caller_extensions_against_bare_mint(
        caller_exts: &[ExtensionType],
        expected: Result<usize, ProgramError>,
    ) {
        let data = vec![0u8; Mint::BASE_LEN];
        let (_backing, mint) = build_account_view(&ID, &data);

        assert_eq!(
            try_calculate_account_len_from_mint(&mint, caller_exts),
            expected
        );
    }

    #[test]
    fn rejects_account_typed_data_as_mint() {
        // Token-2022-owned and long enough to parse as extended data, but the
        // account-type byte says `Account`, not `Mint`.
        let mut data = vec![0u8; TLV_START_INDEX];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Account as u8;
        let (_backing, mint) = build_account_view(&ID, &data);

        assert_eq!(
            try_calculate_account_len_from_mint(&mint, &[ExtensionType::ImmutableOwner]),
            Err(ProgramError::InvalidAccountData)
        );
    }

    #[test]
    fn unparsable_mint_tlv_errors() {
        // A valid mint base whose extension TLV won't parse
        let mut tlv = Vec::new();
        tlv.extend_from_slice(&(ExtensionType::TransferFeeConfig as u16).to_le_bytes());
        tlv.extend_from_slice(&64u16.to_le_bytes());
        let data = build_mint_data(&tlv);
        let (_backing, mint) = build_account_view(&ID, &data);

        assert_eq!(
            try_calculate_account_len_from_mint(&mint, &[ExtensionType::ImmutableOwner]),
            Err(ProgramError::InvalidAccountData)
        );
    }

    #[test]
    fn every_mint_extension_maps_to_account_extensions() {
        for discriminant in 0..MAX_EXTENSIONS as u16 {
            let Ok(extension_type) = ExtensionType::try_from(discriminant) else {
                continue;
            };
            if extension_account_type(extension_type) != AccountType::Mint {
                continue;
            }

            let required = required_account_extensions_from_mint_extension(extension_type)
                .unwrap_or_else(|| {
                    panic!("mint extension {extension_type:?} is missing an explicit map arm")
                });

            for &required_extension in required {
                assert_eq!(
                    extension_account_type(required_extension),
                    AccountType::Account,
                    "{extension_type:?} requires {required_extension:?}, not an account extension",
                );
                assert!(
                    extension_value_len(required_extension).is_some(),
                    "{extension_type:?} requires {required_extension:?}, which has no known size",
                );
            }
        }
    }

    #[test_case(ExtensionType::MintCloseAuthority, Account::BASE_LEN)] // mint-only, no account ext
    #[test_case(ExtensionType::TransferFeeConfig, 178)] // adds TransferFeeAmount (8)
    #[test_case(ExtensionType::TransferHook, 171)] // adds TransferHookAccount (1)
    #[test_case(ExtensionType::Pausable, 170)] // adds PausableAccount (0)
    fn single_mint_extension_account_len(mint_ext: ExtensionType, expected: usize) {
        let mut tlv = Vec::new();
        push_tlv_entry(&mut tlv, mint_ext, &[]);
        let data = build_mint_data(&tlv);
        let (_backing, mint) = build_account_view(&ID, &data);

        assert_eq!(
            try_calculate_account_len_from_mint(&mint, &[]),
            Ok(expected)
        );
    }

    #[test]
    fn dedups_immutable_owner_against_non_transferable_mint() {
        // NonTransferable on the mint requires both NonTransferableAccount and
        // ImmutableOwner. Correct dedup produces 166 (TLV_START_INDEX) + 4
        // (ImmutableOwner TLV) + 4 (NonTransferableAccount TLV) = 174.
        let mut tlv = Vec::new();
        push_tlv_entry(&mut tlv, ExtensionType::NonTransferable, &[]);
        let data = build_mint_data(&tlv);
        let (_backing, mint) = build_account_view(&ID, &data);

        assert_eq!(
            try_calculate_account_len_from_mint(&mint, &[ExtensionType::ImmutableOwner]),
            Ok(174)
        );
    }
}
