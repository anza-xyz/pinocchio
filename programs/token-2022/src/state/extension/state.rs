use {
    super::{
        extension_not_found_error, validate_extension_account_type, validate_mint_extensions_data,
        validate_token_extensions_data, ExtensionBaseState, ExtensionType, ExtensionValue,
        BASE_ACCOUNT_LEN, TLV_HEADER_LEN, TLV_START_INDEX,
    },
    crate::{
        state::{Account, AccountType, Mint, Multisig},
        ID,
    },
    solana_account_view::{AccountView, Ref, RefMut},
    solana_program_error::ProgramError,
};

/// Find the value bytes for a given extension type via linear TLV walk.
#[inline]
pub(super) fn get_extension_bytes_from_tlv(
    tlv_data: &[u8],
    target: ExtensionType,
) -> Result<&[u8], ProgramError> {
    let target_val = target as u16;
    let mut offset = 0;

    while offset + TLV_HEADER_LEN <= tlv_data.len() {
        let ext_type = u16::from_le_bytes([tlv_data[offset], tlv_data[offset + 1]]);
        let length = u16::from_le_bytes([tlv_data[offset + 2], tlv_data[offset + 3]]) as usize;

        if ext_type == 0 {
            return Err(extension_not_found_error());
        }

        let value_start = offset + TLV_HEADER_LEN;
        let value_end = value_start + length;

        if value_end > tlv_data.len() {
            return Err(ProgramError::InvalidAccountData);
        }

        if ext_type == target_val {
            return Ok(&tlv_data[value_start..value_end]);
        }

        offset = value_end;
    }

    if offset == tlv_data.len() {
        Err(extension_not_found_error())
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}

/// Find the mutable value bytes for a given extension type via linear TLV walk.
#[inline]
pub(super) fn get_extension_bytes_from_tlv_mut(
    tlv_data: &mut [u8],
    target: ExtensionType,
) -> Result<&mut [u8], ProgramError> {
    let target_val = target as u16;
    let mut offset = 0;

    while offset + TLV_HEADER_LEN <= tlv_data.len() {
        let ext_type = u16::from_le_bytes([tlv_data[offset], tlv_data[offset + 1]]);
        let length = u16::from_le_bytes([tlv_data[offset + 2], tlv_data[offset + 3]]) as usize;

        if ext_type == 0 {
            return Err(extension_not_found_error());
        }

        let value_start = offset + TLV_HEADER_LEN;
        let value_end = value_start + length;

        if value_end > tlv_data.len() {
            return Err(ProgramError::InvalidAccountData);
        }

        if ext_type == target_val {
            return Ok(&mut tlv_data[value_start..value_end]);
        }

        offset = value_end;
    }

    if offset == tlv_data.len() {
        Err(extension_not_found_error())
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}

/// Collect extension types from TLV data in encounter order.
///
/// Returns the number of written entries on success.
#[inline]
pub(super) fn write_extension_types_from_tlv(
    tlv_data: &[u8],
    out: &mut [ExtensionType],
) -> Result<usize, ProgramError> {
    let mut count = 0;
    let mut offset = 0;

    while offset < tlv_data.len() {
        if offset + core::mem::size_of::<u16>() > tlv_data.len() {
            return Ok(count);
        }

        let ext_type_raw = u16::from_le_bytes([tlv_data[offset], tlv_data[offset + 1]]);

        if ext_type_raw == 0 {
            return Ok(count);
        }

        if offset + TLV_HEADER_LEN > tlv_data.len() {
            return Err(ProgramError::InvalidAccountData);
        }

        let length = u16::from_le_bytes([tlv_data[offset + 2], tlv_data[offset + 3]]) as usize;

        let value_start = offset + TLV_HEADER_LEN;
        let value_end = value_start + length;

        if value_end > tlv_data.len() {
            return Err(ProgramError::InvalidAccountData);
        }
        if count == out.len() {
            return Err(ProgramError::InvalidArgument);
        }

        out[count] = ExtensionType::try_from(ext_type_raw)?;
        count += 1;
        offset = value_end;
    }

    Ok(count)
}

#[inline(always)]
pub(super) fn extension_from_bytes<T: ExtensionValue>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    if !(bytes.as_ptr() as usize).is_multiple_of(core::mem::align_of::<T>()) {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

#[inline(always)]
pub(super) fn extension_from_bytes_mut<T: ExtensionValue>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    if bytes.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    if !(bytes.as_ptr() as usize).is_multiple_of(core::mem::align_of::<T>()) {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut T) })
}

impl ExtensionBaseState for Mint {
    const BASE_LEN: usize = Mint::BASE_LEN;
    const ACCOUNT_TYPE: AccountType = AccountType::Mint;

    #[inline(always)]
    fn validate_extensions_data(data: &[u8]) -> Result<(), ProgramError> {
        validate_mint_extensions_data(data)
    }

    #[inline(always)]
    unsafe fn from_bytes_unchecked(data: &[u8]) -> &Self {
        Mint::from_bytes_unchecked(data)
    }

    #[inline(always)]
    unsafe fn from_bytes_unchecked_mut(data: &mut [u8]) -> &mut Self {
        Mint::from_bytes_unchecked_mut(data)
    }
}

impl ExtensionBaseState for Account {
    const BASE_LEN: usize = Account::BASE_LEN;
    const ACCOUNT_TYPE: AccountType = AccountType::Account;

    #[inline(always)]
    fn validate_extensions_data(data: &[u8]) -> Result<(), ProgramError> {
        validate_token_extensions_data(data)
    }

    #[inline(always)]
    unsafe fn from_bytes_unchecked(data: &[u8]) -> &Self {
        Account::from_bytes_unchecked(data)
    }

    #[inline(always)]
    unsafe fn from_bytes_unchecked_mut(data: &mut [u8]) -> &mut Self {
        Account::from_bytes_unchecked_mut(data)
    }
}

#[inline(always)]
fn extension_data_start<B: ExtensionBaseState>(tail_len: usize) -> usize {
    core::cmp::min(tail_len, TLV_START_INDEX - B::BASE_LEN)
}

#[inline]
fn validate_state_with_extensions_data<B: ExtensionBaseState>(
    data: &[u8],
) -> Result<(), ProgramError> {
    if data.len() == Multisig::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    if data.len() == B::BASE_LEN {
        return Ok(());
    }
    if data.len() <= BASE_ACCOUNT_LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    B::validate_extensions_data(data)
}

/// A base state with TLV extension data.
#[repr(C)]
pub struct StateWithExtensions<B: ExtensionBaseState> {
    pub base: B,
    data: [u8],
}

impl<B: ExtensionBaseState> StateWithExtensions<B> {
    /// Return a `StateWithExtensions` from the given byte slice.
    ///
    /// This method validates the data layout but does **not** check the
    /// account owner. Callers needing owner validation should check
    /// ownership before borrowing.
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        validate_state_with_extensions_data::<B>(data)?;
        Ok(unsafe { Self::from_bytes_unchecked(data) })
    }

    /// Return a `StateWithExtensions` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView` and
    /// checks the account data borrow state.
    #[inline]
    pub fn from_account_view(account_view: &AccountView) -> Result<Ref<'_, Self>, ProgramError> {
        if !account_view.owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ref::try_map(account_view.try_borrow()?, Self::from_bytes).map_err(|(_, error)| error)
    }

    /// Return a `StateWithExtensions` from the given account view.
    ///
    /// This method performs owner and length validation on
    /// `AccountView`, but does not perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account
    /// data (e.g., there are no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_view_unchecked(
        account_view: &AccountView,
    ) -> Result<&Self, ProgramError> {
        if account_view.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Self::from_bytes(account_view.borrow_unchecked())
    }

    /// # Safety
    ///
    /// `data` must have passed `validate_state_with_extensions_data::<B>`.
    #[inline(always)]
    unsafe fn from_bytes_unchecked(data: &[u8]) -> &Self {
        debug_assert!(data.len() >= B::BASE_LEN);
        let ptr = core::ptr::slice_from_raw_parts(data.as_ptr(), data.len() - B::BASE_LEN);
        &*(ptr as *const Self)
    }

    #[inline(always)]
    fn tlv_data(&self) -> &[u8] {
        let data_start = extension_data_start::<B>(self.data.len());
        &self.data[data_start..]
    }

    /// Find the value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    fn get_extension_bytes(&self, target: ExtensionType) -> Result<&[u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv(self.tlv_data(), target)
    }

    #[inline]
    pub fn write_extension_types(&self, out: &mut [ExtensionType]) -> Result<usize, ProgramError> {
        write_extension_types_from_tlv(self.tlv_data(), out)
    }

    #[inline]
    pub fn get_extension<V: ExtensionValue>(&self) -> Result<&V, ProgramError> {
        let bytes = self.get_extension_bytes(V::TYPE)?;
        extension_from_bytes(bytes)
    }
}

/// A base state with TLV extension data backed by an unchecked mutable borrow.
#[repr(C)]
pub struct StateWithExtensionsMut<B: ExtensionBaseState> {
    pub base: B,
    data: [u8],
}

impl<B: ExtensionBaseState> StateWithExtensionsMut<B> {
    /// Return a `StateWithExtensionsMut` from the given mutable byte slice.
    ///
    /// This method validates the data layout but does **not** check the
    /// account owner. Callers needing owner validation should check
    /// ownership before borrowing.
    #[inline]
    pub fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        validate_state_with_extensions_data::<B>(data)?;
        Ok(unsafe { Self::from_bytes_mut_unchecked(data) })
    }

    /// Return a `StateWithExtensionsMut` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView` and
    /// checks the account data borrow state.
    #[inline]
    pub fn from_account_view_mut(
        account_view: &mut AccountView,
    ) -> Result<RefMut<'_, Self>, ProgramError> {
        if !account_view.owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        RefMut::try_map(account_view.try_borrow_mut()?, Self::from_bytes_mut)
            .map_err(|(_, error)| error)
    }

    /// Return a `StateWithExtensionsMut` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`,
    /// but does not perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to mutably borrow the account
    /// data (e.g., there are no active borrows of the account data).
    #[inline]
    pub unsafe fn from_account_view_unchecked_mut(
        account_view: &mut AccountView,
    ) -> Result<&mut Self, ProgramError> {
        if account_view.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Self::from_bytes_mut(account_view.borrow_unchecked_mut())
    }

    /// Find the value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    fn get_extension_bytes(&self, target: ExtensionType) -> Result<&[u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv(self.tlv_data(), target)
    }

    /// Find the mutable value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    fn get_extension_bytes_mut(
        &mut self,
        target: ExtensionType,
    ) -> Result<&mut [u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv_mut(self.tlv_data_mut(), target)
    }

    #[inline]
    pub fn write_extension_types(&self, out: &mut [ExtensionType]) -> Result<usize, ProgramError> {
        write_extension_types_from_tlv(self.tlv_data(), out)
    }

    #[inline]
    pub fn get_extension<V: ExtensionValue>(&self) -> Result<&V, ProgramError> {
        let bytes = self.get_extension_bytes(V::TYPE)?;
        extension_from_bytes(bytes)
    }

    #[inline]
    pub fn get_extension_mut<V: ExtensionValue>(&mut self) -> Result<&mut V, ProgramError> {
        let bytes = self.get_extension_bytes_mut(V::TYPE)?;
        extension_from_bytes_mut(bytes)
    }

    /// # Safety
    ///
    /// `data` must have passed `validate_state_with_extensions_data::<B>`.
    #[inline(always)]
    unsafe fn from_bytes_mut_unchecked(data: &mut [u8]) -> &mut Self {
        debug_assert!(data.len() >= B::BASE_LEN);
        let ptr = core::ptr::slice_from_raw_parts_mut(data.as_mut_ptr(), data.len() - B::BASE_LEN);
        &mut *(ptr as *mut Self)
    }

    #[inline(always)]
    fn tlv_data(&self) -> &[u8] {
        let data_start = extension_data_start::<B>(self.data.len());
        &self.data[data_start..]
    }

    #[inline]
    fn tlv_data_mut(&mut self) -> &mut [u8] {
        let data_start = extension_data_start::<B>(self.data.len());
        &mut self.data[data_start..]
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use {
        super::*,
        crate::state::{
            extension::{
                adjust_len_for_multisig,
                default_account_state::DefaultAccountStateExtension,
                extension_account_type,
                immutable_owner::ImmutableOwnerExtension,
                is_extension_not_found_error,
                non_transferable_account::NonTransferableAccountExtension,
                pausable_account::PausableAccountExtension,
                permanent_delegate::PermanentDelegateExtension,
                permissioned_burn::PermissionedBurnExtension,
                shared_test_helpers::{build_account_view, build_mint_data, push_tlv_entry},
                transfer_fee_amount::TransferFeeAmountExtension,
                transfer_hook::TransferHookExtension,
                transfer_hook_account::TransferHookAccountExtension,
                try_calculate_account_len, TokenError, ACCOUNT_TYPE_INDEX,
                EXTENSION_NOT_FOUND_ERROR_CODE,
            },
            AccountState,
        },
        core::mem::size_of,
        solana_address::Address,
        std::{vec, vec::Vec},
    };

    fn build_token_data(tlv_data: &[u8]) -> Vec<u8> {
        let mut data = vec![0u8; TLV_START_INDEX + tlv_data.len()];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Account as u8;
        data[TLV_START_INDEX..].copy_from_slice(tlv_data);
        data
    }

    #[test]
    fn validate_mint_extensions_data_rejects_non_zero_padding() {
        let mut data = [0u8; TLV_START_INDEX];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;
        data[Mint::BASE_LEN] = 1;

        assert_eq!(
            validate_mint_extensions_data(&data),
            Err(ProgramError::InvalidAccountData)
        );
    }

    #[test]
    fn validate_token_extensions_data_requires_account_type() {
        let mut data = [0u8; TLV_START_INDEX];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;

        assert_eq!(
            validate_token_extensions_data(&data),
            Err(ProgramError::InvalidAccountData)
        );

        data[ACCOUNT_TYPE_INDEX] = AccountType::Account as u8;

        assert_eq!(validate_token_extensions_data(&data), Ok(()));
    }

    #[test]
    fn get_extension_bytes_from_tlv_rejects_overflow() {
        let mut tlv = [0u8; 6];
        tlv[..2].copy_from_slice(&(ExtensionType::DefaultAccountState as u16).to_le_bytes());
        tlv[2..4].copy_from_slice(&5u16.to_le_bytes());

        assert_eq!(
            get_extension_bytes_from_tlv(&tlv, ExtensionType::DefaultAccountState),
            Err(ProgramError::InvalidAccountData)
        );
    }

    #[test]
    fn get_extension_bytes_from_tlv_rejects_trailing_partial_header() {
        let tlv = [0u8; 1];

        assert_eq!(
            get_extension_bytes_from_tlv(&tlv, ExtensionType::DefaultAccountState),
            Err(ProgramError::InvalidAccountData)
        );
    }

    #[test]
    fn get_extension_bytes_from_tlv_mut_updates_value() {
        let mut tlv = [0u8; 5];
        tlv[..2].copy_from_slice(&(ExtensionType::DefaultAccountState as u16).to_le_bytes());
        tlv[2..4].copy_from_slice(&1u16.to_le_bytes());
        tlv[4] = AccountState::Initialized as u8;

        let bytes =
            get_extension_bytes_from_tlv_mut(&mut tlv, ExtensionType::DefaultAccountState).unwrap();
        bytes[0] = AccountState::Frozen as u8;

        assert_eq!(tlv[4], AccountState::Frozen as u8);
    }

    #[test]
    fn extension_account_type_check_rejects_mismatches() {
        assert_eq!(
            validate_extension_account_type(ExtensionType::TransferHookAccount, AccountType::Mint),
            Err(ProgramError::InvalidAccountData)
        );

        assert_eq!(
            validate_extension_account_type(ExtensionType::TransferHook, AccountType::Mint),
            Ok(())
        );
    }

    #[test]
    fn extension_from_bytes_requires_exact_fixed_length() {
        let oversized = [AccountState::Initialized as u8, AccountState::Frozen as u8];

        assert!(matches!(
            extension_from_bytes::<DefaultAccountStateExtension>(&oversized),
            Err(ProgramError::InvalidAccountData)
        ));

        let exact = [AccountState::Frozen as u8];
        let extension = extension_from_bytes::<DefaultAccountStateExtension>(&exact).unwrap();
        assert_eq!(extension.state().unwrap(), AccountState::Frozen);
    }

    #[test]
    fn default_account_state_extension_rejects_invalid_state() {
        let invalid = [3u8];
        let extension = extension_from_bytes::<DefaultAccountStateExtension>(&invalid).unwrap();
        assert_eq!(extension.state(), Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn mint_with_extensions_rejects_wrong_owner() {
        let account_owner = Address::new_from_array([7u8; 32]);
        let data = build_mint_data(&[]);
        let (_backing, account_view) = build_account_view(&account_owner, &data);

        assert!(matches!(
            unsafe { StateWithExtensions::<Mint>::from_account_view_unchecked(&account_view) },
            Err(ProgramError::InvalidAccountOwner)
        ));
    }

    #[test]
    fn mint_with_extensions_rejects_short_data() {
        let data = vec![0u8; BASE_ACCOUNT_LEN];

        assert!(matches!(
            StateWithExtensions::<Mint>::from_bytes(&data),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn mint_with_extensions_accepts_base_only_data() {
        let data = vec![0u8; Mint::BASE_LEN];

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert!(matches!(
            mint.get_extension::<DefaultAccountStateExtension>(),
            Err(error) if is_extension_not_found_error(&error)
        ));
    }

    #[test]
    fn token_with_extensions_accepts_base_only_data() {
        let data = vec![0u8; Account::BASE_LEN];

        let token = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        assert!(matches!(
            token.get_extension::<TransferHookAccountExtension>(),
            Err(error) if is_extension_not_found_error(&error)
        ));
    }

    #[test]
    fn get_extension_returns_present_default_account_state() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let data = build_mint_data(&tlv_data);

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert_eq!(
            mint.get_extension::<DefaultAccountStateExtension>()
                .unwrap()
                .state()
                .unwrap(),
            AccountState::Initialized
        );
    }

    #[test]
    fn get_extension_returns_present_transfer_hook_account() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[1u8]);
        let data = build_token_data(&tlv_data);

        let token = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        assert!(bool::from(
            token
                .get_extension::<TransferHookAccountExtension>()
                .unwrap()
                .transferring
        ));
    }

    #[test]
    fn get_extension_mut_returns_not_found_when_absent() {
        let mut mint_data = vec![0u8; Mint::BASE_LEN];
        let mint = StateWithExtensionsMut::<Mint>::from_bytes_mut(&mut mint_data).unwrap();
        assert!(is_extension_not_found_error(
            &mint
                .get_extension_mut::<DefaultAccountStateExtension>()
                .unwrap_err()
        ));

        let mut token_data = vec![0u8; Account::BASE_LEN];
        let token = StateWithExtensionsMut::<Account>::from_bytes_mut(&mut token_data).unwrap();
        assert!(is_extension_not_found_error(
            &token
                .get_extension_mut::<TransferHookAccountExtension>()
                .unwrap_err()
        ));
    }

    #[test]
    fn get_extension_propagates_corrupt_tlv_data() {
        let corrupt_mint_data = build_mint_data(&[0u8]);
        let mint = StateWithExtensions::<Mint>::from_bytes(&corrupt_mint_data).unwrap();
        assert!(matches!(
            mint.get_extension::<DefaultAccountStateExtension>(),
            Err(ProgramError::InvalidAccountData)
        ));

        let corrupt_token_data = build_token_data(&[0u8]);
        let token = StateWithExtensions::<Account>::from_bytes(&corrupt_token_data).unwrap();
        assert!(matches!(
            token.get_extension::<TransferHookAccountExtension>(),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn mint_with_extensions_rejects_multisig_len_collision() {
        let mut data = vec![0u8; Multisig::LEN];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;

        assert!(matches!(
            StateWithExtensions::<Mint>::from_bytes(&data),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn token_with_extensions_rejects_multisig_len_collision() {
        let mut data = vec![0u8; Multisig::LEN];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Account as u8;

        assert!(matches!(
            StateWithExtensions::<Account>::from_bytes(&data),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn token_with_extensions_rejects_wrong_account_type() {
        let mut data = build_token_data(&[]);
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;

        assert!(matches!(
            StateWithExtensions::<Account>::from_bytes(&data),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn mint_with_extensions_unchecked_rejects_token_account_type() {
        let data = build_token_data(&[]);
        let (_backing, account_view) = build_account_view(&ID, &data);

        assert!(matches!(
            unsafe { StateWithExtensions::<Mint>::from_account_view_unchecked(&account_view) },
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn mint_with_extensions_from_bytes_write_then_read() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let mut data = build_mint_data(&tlv_data);

        {
            let mint = StateWithExtensionsMut::<Mint>::from_bytes_mut(&mut data).unwrap();
            mint.get_extension_mut::<DefaultAccountStateExtension>()
                .unwrap()
                .set_state(AccountState::Frozen);
        }

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert_eq!(
            mint.get_extension::<DefaultAccountStateExtension>()
                .unwrap()
                .state()
                .unwrap(),
            AccountState::Frozen,
        );
    }

    #[test]
    fn token_with_extensions_from_bytes_write_then_read() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[0u8]);
        let mut data = build_token_data(&tlv_data);

        {
            let token = StateWithExtensionsMut::<Account>::from_bytes_mut(&mut data).unwrap();
            token
                .get_extension_mut::<TransferHookAccountExtension>()
                .unwrap()
                .transferring = true.into();
        }

        let token = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        assert!(bool::from(
            token
                .get_extension::<TransferHookAccountExtension>()
                .unwrap()
                .transferring
        ));
    }

    #[test]
    fn mint_with_extensions_from_account_view_enforces_borrow_rules() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);
        let mut account_view_mut = account_view.clone();

        let mint = StateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        assert!(
            StateWithExtensionsMut::<Mint>::from_account_view_mut(&mut account_view_mut).is_err()
        );
        drop(mint);

        let mut mint =
            StateWithExtensionsMut::<Mint>::from_account_view_mut(&mut account_view_mut).unwrap();
        let account_view_read = account_view.clone();
        assert!(StateWithExtensions::<Mint>::from_account_view(&account_view_read).is_err());
        mint.get_extension_mut::<DefaultAccountStateExtension>()
            .unwrap()
            .set_state(AccountState::Frozen);
        drop(mint);

        let mint = StateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        assert_eq!(
            mint.get_extension::<DefaultAccountStateExtension>()
                .unwrap()
                .state()
                .unwrap(),
            AccountState::Frozen,
        );
    }

    #[test]
    fn token_with_extensions_from_account_view_enforces_borrow_rules() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[0u8]);
        let data = build_token_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);
        let mut account_view_mut = account_view.clone();

        let token = StateWithExtensions::<Account>::from_account_view(&account_view).unwrap();
        assert!(
            StateWithExtensionsMut::<Account>::from_account_view_mut(&mut account_view_mut)
                .is_err()
        );
        drop(token);

        let mut token =
            StateWithExtensionsMut::<Account>::from_account_view_mut(&mut account_view_mut)
                .unwrap();
        let account_view_read = account_view.clone();
        assert!(StateWithExtensions::<Account>::from_account_view(&account_view_read).is_err());
        token
            .get_extension_mut::<TransferHookAccountExtension>()
            .unwrap()
            .transferring = true.into();
        drop(token);

        let token = StateWithExtensions::<Account>::from_account_view(&account_view).unwrap();
        assert!(bool::from(
            token
                .get_extension::<TransferHookAccountExtension>()
                .unwrap()
                .transferring
        ));
    }

    #[test]
    fn get_extension_bytes_from_tlv_finds_middle_entry() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &[9u8; 32]);
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Frozen as u8],
        );
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHook, &[5u8; 64]);

        let bytes =
            get_extension_bytes_from_tlv(&tlv_data, ExtensionType::DefaultAccountState).unwrap();
        assert_eq!(bytes, [AccountState::Frozen as u8]);
    }

    #[test]
    fn get_extension_bytes_from_tlv_stops_at_uninitialized_entry() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        push_tlv_entry(&mut tlv_data, ExtensionType::Uninitialized, &[]);
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[1u8]);

        assert!(matches!(
            get_extension_bytes_from_tlv(&tlv_data, ExtensionType::TransferHookAccount),
            Err(error) if is_extension_not_found_error(&error)
        ));
    }

    #[test]
    fn get_extension_bytes_from_tlv_returns_first_duplicate_entry() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Frozen as u8],
        );

        let bytes =
            get_extension_bytes_from_tlv(&tlv_data, ExtensionType::DefaultAccountState).unwrap();
        assert_eq!(bytes, [AccountState::Initialized as u8]);
    }

    #[test]
    fn transfer_hook_account_extension_bool_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[0u8]);
        let data = build_token_data(&tlv_data);
        let (_backing, mut account_view) = build_account_view(&ID, &data);

        let token = unsafe {
            StateWithExtensionsMut::<Account>::from_account_view_unchecked_mut(&mut account_view)
        }
        .unwrap();
        let extension = token
            .get_extension_mut::<TransferHookAccountExtension>()
            .unwrap();
        assert!(!bool::from(extension.transferring));

        extension.transferring = true.into();
        assert!(bool::from(extension.transferring));

        extension.transferring = false.into();
        assert!(!bool::from(extension.transferring));
    }

    #[test]
    fn mint_default_account_state_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let mut data = build_mint_data(&tlv_data);

        {
            let mint = StateWithExtensionsMut::<Mint>::from_bytes_mut(&mut data).unwrap();
            mint.get_extension_mut::<DefaultAccountStateExtension>()
                .unwrap()
                .set_state(AccountState::Frozen);
        }

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert_eq!(
            mint.get_extension::<DefaultAccountStateExtension>()
                .unwrap()
                .state()
                .unwrap(),
            AccountState::Frozen,
        );
    }

    #[test]
    fn token_transfer_hook_account_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[0u8]);
        let mut data = build_token_data(&tlv_data);

        {
            let token = StateWithExtensionsMut::<Account>::from_bytes_mut(&mut data).unwrap();
            token
                .get_extension_mut::<TransferHookAccountExtension>()
                .unwrap()
                .transferring = true.into();
        }

        let token = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        assert!(bool::from(
            token
                .get_extension::<TransferHookAccountExtension>()
                .unwrap()
                .transferring
        ));
    }

    #[test]
    fn get_extension_types_lists_entries_in_order() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &[7u8; 32]);
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let data = build_mint_data(&tlv_data);

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let mut out = [ExtensionType::Uninitialized; 2];
        let written = mint.write_extension_types(&mut out).unwrap();

        assert_eq!(written, 2);
        assert_eq!(out[0], ExtensionType::PermanentDelegate);
        assert_eq!(out[1], ExtensionType::DefaultAccountState);
    }

    #[test]
    fn get_extension_types_on_base_only_returns_empty() {
        let data = vec![0u8; Account::BASE_LEN];

        let token = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        let mut out = [ExtensionType::Uninitialized; 1];
        let written = token.write_extension_types(&mut out).unwrap();

        assert_eq!(written, 0);
    }

    #[test]
    fn get_extension_types_allows_single_trailing_byte() {
        let data = build_mint_data(&[1u8]);

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let mut out = [ExtensionType::Uninitialized; 1];
        let written = mint.write_extension_types(&mut out).unwrap();

        assert_eq!(written, 0);
    }

    #[test]
    fn get_extension_types_rejects_partial_nonzero_header() {
        let mut data = build_mint_data(&[ExtensionType::DefaultAccountState as u8, 0u8]);
        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let mut out = [ExtensionType::Uninitialized; 1];
        assert_eq!(
            mint.write_extension_types(&mut out),
            Err(ProgramError::InvalidAccountData)
        );

        data = build_mint_data(&[ExtensionType::DefaultAccountState as u8, 0u8, 0u8]);
        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert_eq!(
            mint.write_extension_types(&mut out),
            Err(ProgramError::InvalidAccountData)
        );
    }

    #[test]
    fn get_extension_types_requires_sufficient_output_capacity() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &[7u8; 32]);
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let data = build_mint_data(&tlv_data);

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let mut out = [ExtensionType::Uninitialized; 1];
        assert_eq!(
            mint.write_extension_types(&mut out),
            Err(ProgramError::InvalidArgument)
        );
    }

    #[test]
    fn try_calculate_account_len_matches_supported_layouts() {
        assert_eq!(
            try_calculate_account_len::<Mint>(&[]).unwrap(),
            Mint::BASE_LEN
        );
        assert_eq!(
            try_calculate_account_len::<Account>(&[]).unwrap(),
            Account::BASE_LEN
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::DefaultAccountState]).unwrap(),
            TLV_START_INDEX + TLV_HEADER_LEN + DefaultAccountStateExtension::LEN
        );
        assert_eq!(
            try_calculate_account_len::<Account>(&[ExtensionType::TransferHookAccount]).unwrap(),
            TLV_START_INDEX + TLV_HEADER_LEN + TransferHookAccountExtension::LEN
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::PermanentDelegate]).unwrap(),
            TLV_START_INDEX + TLV_HEADER_LEN + PermanentDelegateExtension::LEN
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap(),
            TLV_START_INDEX + TLV_HEADER_LEN + TransferHookExtension::LEN
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::PermissionedBurn]).unwrap(),
            TLV_START_INDEX + TLV_HEADER_LEN + size_of::<Address>()
        );
    }

    #[test]
    fn try_calculate_account_len_rejects_wrong_or_unsupported_extensions() {
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::TransferHookAccount]),
            Err(ProgramError::InvalidAccountData)
        );
        assert_eq!(
            try_calculate_account_len::<Account>(&[ExtensionType::PermissionedBurn]),
            Err(ProgramError::InvalidAccountData)
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[
                ExtensionType::DefaultAccountState,
                ExtensionType::DefaultAccountState,
            ]),
            Err(ProgramError::InvalidInstructionData)
        );
    }

    #[test]
    fn adjust_len_for_multisig_matches_spl_behavior() {
        assert_eq!(
            adjust_len_for_multisig(Multisig::LEN),
            Multisig::LEN + core::mem::size_of::<ExtensionType>()
        );
        assert_eq!(adjust_len_for_multisig(Mint::BASE_LEN), Mint::BASE_LEN);
    }

    #[test]
    fn spl2022_supported_extension_discriminants_match() {
        assert_eq!(ExtensionType::TransferFeeAmount as u16, 2);
        assert_eq!(ExtensionType::DefaultAccountState as u16, 6);
        assert_eq!(ExtensionType::ImmutableOwner as u16, 7);
        assert_eq!(ExtensionType::NonTransferableAccount as u16, 13);
        assert_eq!(ExtensionType::TransferHookAccount as u16, 15);
        assert_eq!(ExtensionType::PausableAccount as u16, 27);
        assert_eq!(ExtensionType::PermissionedBurn as u16, 28);
    }

    #[test]
    fn spl2022_extension_not_found_error_discriminant_matches() {
        assert_eq!(TokenError::ExtensionNotFound as u32, 48);
        assert_eq!(EXTENSION_NOT_FOUND_ERROR_CODE, 48);
        assert!(is_extension_not_found_error(&extension_not_found_error()));
    }

    #[test]
    fn spl2022_supported_extension_account_types_match() {
        assert_eq!(
            extension_account_type(ExtensionType::DefaultAccountState),
            AccountType::Mint
        );
        assert_eq!(
            extension_account_type(ExtensionType::TransferHookAccount),
            AccountType::Account
        );
        assert_eq!(
            extension_account_type(ExtensionType::PermissionedBurn),
            AccountType::Mint
        );
        assert_eq!(
            extension_account_type(ExtensionType::ImmutableOwner),
            AccountType::Account
        );
        assert_eq!(
            extension_account_type(ExtensionType::NonTransferableAccount),
            AccountType::Account
        );
        assert_eq!(
            extension_account_type(ExtensionType::PausableAccount),
            AccountType::Account
        );
        assert_eq!(
            extension_account_type(ExtensionType::TransferFeeAmount),
            AccountType::Account
        );
    }

    #[test]
    fn spl2022_supported_extension_sizes_match() {
        assert_eq!(DefaultAccountStateExtension::LEN, 1);
        assert_eq!(TransferHookAccountExtension::LEN, 1);
        assert_eq!(PermanentDelegateExtension::LEN, 32);
        assert_eq!(PermissionedBurnExtension::LEN, 32);
        assert_eq!(TransferHookExtension::LEN, 64);
        assert_eq!(ImmutableOwnerExtension::LEN, 0);
        assert_eq!(NonTransferableAccountExtension::LEN, 0);
        assert_eq!(PausableAccountExtension::LEN, 0);
        assert_eq!(TransferFeeAmountExtension::LEN, 8);
    }

    #[test]
    fn spl2022_supported_extension_alignments_match() {
        assert_eq!(core::mem::align_of::<DefaultAccountStateExtension>(), 1);
        assert_eq!(core::mem::align_of::<TransferHookAccountExtension>(), 1);
        assert_eq!(core::mem::align_of::<PermanentDelegateExtension>(), 1);
        assert_eq!(core::mem::align_of::<PermissionedBurnExtension>(), 1);
        assert_eq!(core::mem::align_of::<TransferHookExtension>(), 1);
        assert_eq!(core::mem::align_of::<ImmutableOwnerExtension>(), 1);
        assert_eq!(core::mem::align_of::<NonTransferableAccountExtension>(), 1);
        assert_eq!(core::mem::align_of::<PausableAccountExtension>(), 1);
        assert_eq!(core::mem::align_of::<TransferFeeAmountExtension>(), 1);
    }

    #[test]
    fn permanent_delegate_extension_read_roundtrip() {
        let delegate = [42u8; 32];
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &delegate);
        let data = build_mint_data(&tlv_data);

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let ext = mint.get_extension::<PermanentDelegateExtension>().unwrap();
        assert_eq!(ext.delegate.as_ref().unwrap().as_ref(), &delegate);
    }

    #[test]
    fn transfer_hook_extension_read_roundtrip() {
        let mut value = [0u8; 64];
        value[..32].copy_from_slice(&[11u8; 32]);
        value[32..64].copy_from_slice(&[22u8; 32]);
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHook, &value);
        let data = build_mint_data(&tlv_data);

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let ext = mint.get_extension::<TransferHookExtension>().unwrap();
        assert_eq!(ext.authority.as_ref().unwrap().as_ref(), &[11u8; 32]);
        assert_eq!(ext.program_id.as_ref().unwrap().as_ref(), &[22u8; 32]);
    }

    #[test]
    fn permanent_delegate_extension_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &[0u8; 32]);
        let mut data = build_mint_data(&tlv_data);

        let new_delegate = Address::new_from_array([99u8; 32]);
        {
            let mint = StateWithExtensionsMut::<Mint>::from_bytes_mut(&mut data).unwrap();
            mint.get_extension_mut::<PermanentDelegateExtension>()
                .unwrap()
                .delegate = new_delegate.clone().into();
        }

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert_eq!(
            mint.get_extension::<PermanentDelegateExtension>()
                .unwrap()
                .delegate
                .as_ref()
                .unwrap(),
            &new_delegate,
        );
    }

    #[test]
    fn permissioned_burn_extension_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermissionedBurn, &[0u8; 32]);
        let mut data = build_mint_data(&tlv_data);

        let new_authority = Address::new_from_array([55u8; 32]);
        {
            let mint = StateWithExtensionsMut::<Mint>::from_bytes_mut(&mut data).unwrap();
            mint.get_extension_mut::<PermissionedBurnExtension>()
                .unwrap()
                .authority = new_authority.clone().into();
        }

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        assert_eq!(
            mint.get_extension::<PermissionedBurnExtension>()
                .unwrap()
                .authority
                .as_ref()
                .unwrap(),
            &new_authority,
        );
    }

    #[test]
    fn transfer_hook_extension_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHook, &[0u8; 64]);
        let mut data = build_mint_data(&tlv_data);

        let new_authority = Address::new_from_array([77u8; 32]);
        let new_program_id = Address::new_from_array([88u8; 32]);
        {
            let mint = StateWithExtensionsMut::<Mint>::from_bytes_mut(&mut data).unwrap();
            let ext = mint.get_extension_mut::<TransferHookExtension>().unwrap();
            ext.authority = new_authority.clone().into();
            ext.program_id = new_program_id.clone().into();
        }

        let mint = StateWithExtensions::<Mint>::from_bytes(&data).unwrap();
        let ext = mint.get_extension::<TransferHookExtension>().unwrap();
        assert_eq!(ext.authority.as_ref().unwrap(), &new_authority);
        assert_eq!(ext.program_id.as_ref().unwrap(), &new_program_id);
    }

    #[test]
    fn transfer_fee_amount_extension_read_roundtrip() {
        let amount: u64 = 1_234_567_890_123;
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::TransferFeeAmount,
            &amount.to_le_bytes(),
        );
        let data = build_token_data(&tlv_data);

        let account = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        let ext = account
            .get_extension::<TransferFeeAmountExtension>()
            .unwrap();
        assert_eq!(u64::from(ext.withheld_amount), amount);
    }

    #[test]
    fn transfer_fee_amount_extension_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferFeeAmount, &[0u8; 8]);
        let mut data = build_token_data(&tlv_data);

        let new_amount: u64 = 987_654_321;
        let account = StateWithExtensionsMut::<Account>::from_bytes_mut(&mut data).unwrap();
        account
            .get_extension_mut::<TransferFeeAmountExtension>()
            .unwrap()
            .withheld_amount = new_amount.into();

        let account = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        assert_eq!(
            u64::from(
                account
                    .get_extension::<TransferFeeAmountExtension>()
                    .unwrap()
                    .withheld_amount
            ),
            new_amount,
        );
    }

    #[test]
    fn immutable_owner_extension_present() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::ImmutableOwner, &[]);
        let data = build_token_data(&tlv_data);

        let account = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        account.get_extension::<ImmutableOwnerExtension>().unwrap();
    }

    #[test]
    fn non_transferable_account_extension_present() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::NonTransferableAccount, &[]);
        let data = build_token_data(&tlv_data);

        let account = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        account
            .get_extension::<NonTransferableAccountExtension>()
            .unwrap();
    }

    #[test]
    fn pausable_account_extension_present() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PausableAccount, &[]);
        let data = build_token_data(&tlv_data);

        let account = StateWithExtensions::<Account>::from_bytes(&data).unwrap();
        account.get_extension::<PausableAccountExtension>().unwrap();
    }
}
