use {
    super::{
        extension_not_found_error, extension_type_from_u16, mint_from_bytes_unchecked_mut,
        token_account_from_bytes_unchecked_mut, validate_extension_account_type,
        validate_mint_extensions_data, validate_token_extensions_data, ExtensionBaseState,
        ExtensionType, ExtensionValue, Pod, BASE_ACCOUNT_LEN, TLV_HEADER_LEN, TLV_START_INDEX,
    },
    crate::{
        state::{AccountType, Mint, Multisig, TokenAccount},
        ID,
    },
    core::marker::PhantomData,
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
pub(super) fn collect_extension_types_from_tlv(
    tlv_data: &[u8],
    out: &mut [ExtensionType],
) -> Result<usize, ProgramError> {
    let mut count = 0;
    let mut offset = 0;

    while offset + TLV_HEADER_LEN <= tlv_data.len() {
        let ext_type_raw = u16::from_le_bytes([tlv_data[offset], tlv_data[offset + 1]]);
        let length = u16::from_le_bytes([tlv_data[offset + 2], tlv_data[offset + 3]]) as usize;

        if ext_type_raw == 0 {
            break;
        }

        let value_start = offset + TLV_HEADER_LEN;
        let value_end = value_start + length;

        if value_end > tlv_data.len() {
            return Err(ProgramError::InvalidAccountData);
        }
        if count == out.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        out[count] = extension_type_from_u16(ext_type_raw)?;
        count += 1;
        offset = value_end;
    }

    Ok(count)
}

#[inline(always)]
pub(super) fn extension_from_bytes<T: Pod>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    if (bytes.as_ptr() as usize) % core::mem::align_of::<T>() != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

#[inline(always)]
pub(super) fn extension_from_bytes_mut<T: Pod>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    if (bytes.as_ptr() as usize) % core::mem::align_of::<T>() != 0 {
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
        mint_from_bytes_unchecked_mut(data)
    }
}

impl ExtensionBaseState for TokenAccount {
    const BASE_LEN: usize = TokenAccount::BASE_LEN;
    const ACCOUNT_TYPE: AccountType = AccountType::Account;

    #[inline(always)]
    fn validate_extensions_data(data: &[u8]) -> Result<(), ProgramError> {
        validate_token_extensions_data(data)
    }

    #[inline(always)]
    unsafe fn from_bytes_unchecked(data: &[u8]) -> &Self {
        TokenAccount::from_bytes_unchecked(data)
    }

    #[inline(always)]
    unsafe fn from_bytes_unchecked_mut(data: &mut [u8]) -> &mut Self {
        token_account_from_bytes_unchecked_mut(data)
    }
}

#[inline(always)]
fn extension_data_start<B: ExtensionBaseState>(data_len: usize) -> usize {
    if data_len == B::BASE_LEN {
        data_len
    } else {
        TLV_START_INDEX
    }
}

#[inline]
fn validate_state_with_extensions_data<B: ExtensionBaseState>(
    data: &[u8],
) -> Result<(), ProgramError> {
    if data.len() < B::BASE_LEN {
        return Err(ProgramError::InvalidAccountData);
    }
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
pub struct StateWithExtensions<'a, B: ExtensionBaseState> {
    base: &'a B,
    tlv_data: &'a [u8],
}

impl<'a, B: ExtensionBaseState> StateWithExtensions<'a, B> {
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
        account_view: &'a AccountView,
    ) -> Result<Self, ProgramError> {
        if account_view.data_len() < B::BASE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_view.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_view.borrow_unchecked();
        validate_state_with_extensions_data::<B>(data)?;

        let base = B::from_bytes_unchecked(data);
        let tlv_data = &data[extension_data_start::<B>(data.len())..];

        Ok(Self { base, tlv_data })
    }

    #[inline(always)]
    pub fn base(&self) -> &B {
        self.base
    }

    /// Find the value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    pub fn get_extension_bytes(&self, target: ExtensionType) -> Result<&'a [u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv(self.tlv_data, target)
    }

    #[inline]
    pub fn get_extension_types(&self, out: &mut [ExtensionType]) -> Result<usize, ProgramError> {
        collect_extension_types_from_tlv(self.tlv_data, out)
    }

    #[inline]
    pub fn get_extension<V: ExtensionValue>(&self) -> Result<&'a V, ProgramError> {
        let bytes = self.get_extension_bytes(V::TYPE)?;
        extension_from_bytes(bytes)
    }
}

/// A base state with TLV extension data backed by a checked borrow.
///
/// This type holds a [`Ref`] guard that keeps the account data borrow alive.
/// Use this when you need safe, checked borrowing of the account data.
pub struct RefStateWithExtensions<'a, B: ExtensionBaseState> {
    data: Ref<'a, [u8]>,
    _marker: PhantomData<B>,
}

impl<'a, B: ExtensionBaseState> RefStateWithExtensions<'a, B> {
    /// Return a `RefStateWithExtensions` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`,
    /// safe borrowing the account data.
    #[inline]
    pub fn from_account_view(account_view: &'a AccountView) -> Result<Self, ProgramError> {
        if account_view.data_len() < B::BASE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account_view.owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_view.try_borrow()?;
        validate_state_with_extensions_data::<B>(&data)?;

        Ok(Self {
            data,
            _marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn base(&self) -> &B {
        unsafe { B::from_bytes_unchecked(&self.data) }
    }

    #[inline]
    fn tlv_data(&self) -> &[u8] {
        &self.data[extension_data_start::<B>(self.data.len())..]
    }

    /// Find the value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    pub fn get_extension_bytes(&self, target: ExtensionType) -> Result<&[u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv(self.tlv_data(), target)
    }

    #[inline]
    pub fn get_extension_types(&self, out: &mut [ExtensionType]) -> Result<usize, ProgramError> {
        collect_extension_types_from_tlv(self.tlv_data(), out)
    }

    #[inline]
    pub fn get_extension<V: ExtensionValue>(&self) -> Result<&V, ProgramError> {
        let bytes = self.get_extension_bytes(V::TYPE)?;
        extension_from_bytes(bytes)
    }
}

/// A base state with TLV extension data backed by a checked mutable borrow.
pub struct RefMutStateWithExtensions<'a, B: ExtensionBaseState> {
    data: RefMut<'a, [u8]>,
    _marker: PhantomData<B>,
}

impl<'a, B: ExtensionBaseState> RefMutStateWithExtensions<'a, B> {
    /// Return a `RefMutStateWithExtensions` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`,
    /// safe mutable borrowing the account data.
    #[inline]
    pub fn from_account_view(account_view: &'a AccountView) -> Result<Self, ProgramError> {
        if account_view.data_len() < B::BASE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account_view.owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_view.try_borrow_mut()?;
        validate_state_with_extensions_data::<B>(&data)?;

        Ok(Self {
            data,
            _marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn base(&self) -> &B {
        unsafe { B::from_bytes_unchecked(&self.data) }
    }

    #[inline(always)]
    pub fn base_mut(&mut self) -> &mut B {
        unsafe { B::from_bytes_unchecked_mut(&mut self.data) }
    }

    #[inline]
    fn tlv_data(&self) -> &[u8] {
        &self.data[extension_data_start::<B>(self.data.len())..]
    }

    #[inline]
    fn tlv_data_mut(&mut self) -> &mut [u8] {
        let data_len = self.data.len();
        &mut self.data[extension_data_start::<B>(data_len)..]
    }

    /// Find the value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    pub fn get_extension_bytes(&self, target: ExtensionType) -> Result<&[u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv(self.tlv_data(), target)
    }

    /// Find the mutable value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    pub fn get_extension_bytes_mut(
        &mut self,
        target: ExtensionType,
    ) -> Result<&mut [u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv_mut(self.tlv_data_mut(), target)
    }

    #[inline]
    pub fn get_extension_types(&self, out: &mut [ExtensionType]) -> Result<usize, ProgramError> {
        collect_extension_types_from_tlv(self.tlv_data(), out)
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
}

/// A base state with TLV extension data backed by an unchecked mutable borrow.
pub struct StateWithExtensionsMut<'a, B: ExtensionBaseState> {
    base: &'a mut B,
    tlv_data: &'a mut [u8],
}

impl<'a, B: ExtensionBaseState> StateWithExtensionsMut<'a, B> {
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
    pub unsafe fn from_account_view_unchecked(
        account_view: &'a AccountView,
    ) -> Result<Self, ProgramError> {
        if account_view.data_len() < B::BASE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_view.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let data = account_view.borrow_unchecked_mut();
        validate_state_with_extensions_data::<B>(data)?;

        let (base_and_type, tlv_data) = data.split_at_mut(extension_data_start::<B>(data.len()));
        let base = B::from_bytes_unchecked_mut(base_and_type);

        Ok(Self { base, tlv_data })
    }

    #[inline(always)]
    pub fn base(&self) -> &B {
        self.base
    }

    #[inline(always)]
    pub fn base_mut(&mut self) -> &mut B {
        self.base
    }

    /// Find the value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    pub fn get_extension_bytes(&self, target: ExtensionType) -> Result<&[u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv(self.tlv_data, target)
    }

    /// Find the mutable value bytes for a given extension type via
    /// linear TLV walk.
    #[inline]
    pub fn get_extension_bytes_mut(
        &mut self,
        target: ExtensionType,
    ) -> Result<&mut [u8], ProgramError> {
        validate_extension_account_type(target, B::ACCOUNT_TYPE)?;
        get_extension_bytes_from_tlv_mut(self.tlv_data, target)
    }

    #[inline]
    pub fn get_extension_types(&self, out: &mut [ExtensionType]) -> Result<usize, ProgramError> {
        collect_extension_types_from_tlv(self.tlv_data, out)
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
}

#[cfg(test)]
mod tests {
    extern crate std;

    use {
        super::*,
        crate::state::{
            extension::{
                adjust_len_for_multisig, default_account_state::DefaultAccountStateExtension,
                extension_account_type, is_extension_not_found_error,
                permanent_delegate::PermanentDelegateExtension,
                transfer_hook::TransferHookExtension,
                transfer_hook_account::TransferHookAccountExtension, try_calculate_account_len,
                TokenError, ACCOUNT_TYPE_INDEX, EXTENSION_NOT_FOUND_ERROR_CODE,
            },
            AccountState,
        },
        core::{mem::size_of, ptr::copy_nonoverlapping},
        solana_account_view::{RuntimeAccount, NOT_BORROWED},
        solana_address::Address,
        std::{vec, vec::Vec},
    };

    fn push_tlv_entry(buffer: &mut Vec<u8>, extension_type: ExtensionType, value: &[u8]) {
        buffer.extend_from_slice(&(extension_type as u16).to_le_bytes());
        buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
        buffer.extend_from_slice(value);
    }

    fn build_mint_data(tlv_data: &[u8]) -> Vec<u8> {
        let mut data = vec![0u8; TLV_START_INDEX + tlv_data.len()];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;
        data[TLV_START_INDEX..].copy_from_slice(tlv_data);
        data
    }

    fn build_token_data(tlv_data: &[u8]) -> Vec<u8> {
        let mut data = vec![0u8; TLV_START_INDEX + tlv_data.len()];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Account as u8;
        data[TLV_START_INDEX..].copy_from_slice(tlv_data);
        data
    }

    fn build_account_view(owner: &Address, data: &[u8]) -> (Vec<u64>, AccountView) {
        let runtime_len = size_of::<RuntimeAccount>();
        let total_len = runtime_len + data.len();
        let backing_len = total_len.div_ceil(size_of::<u64>());
        let mut backing = vec![0u64; backing_len];
        let raw = backing.as_mut_ptr() as *mut RuntimeAccount;

        unsafe {
            (*raw).borrow_state = NOT_BORROWED;
            (*raw).is_signer = 0;
            (*raw).is_writable = 1;
            (*raw).executable = 0;
            (*raw).resize_delta = 0;
            (*raw).address = Address::new_from_array([42u8; 32]);
            (*raw).owner = owner.clone();
            (*raw).lamports = 1;
            (*raw).data_len = data.len() as u64;

            let data_ptr = (raw as *mut u8).add(runtime_len);
            copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());

            (backing, AccountView::new_unchecked(raw))
        }
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
    fn ref_mint_with_extensions_rejects_wrong_owner() {
        let account_owner = Address::new_from_array([7u8; 32]);
        let data = build_mint_data(&[]);
        let (_backing, account_view) = build_account_view(&account_owner, &data);

        assert!(matches!(
            RefStateWithExtensions::<Mint>::from_account_view(&account_view),
            Err(ProgramError::InvalidAccountOwner)
        ));
    }

    #[test]
    fn ref_mint_with_extensions_rejects_short_data() {
        let data = vec![0u8; BASE_ACCOUNT_LEN];
        let (_backing, account_view) = build_account_view(&ID, &data);

        assert!(matches!(
            RefStateWithExtensions::<Mint>::from_account_view(&account_view),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn ref_mint_with_extensions_accepts_base_only_data() {
        let data = vec![0u8; Mint::BASE_LEN];
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        assert!(matches!(
            mint.get_extension::<DefaultAccountStateExtension>(),
            Err(error) if is_extension_not_found_error(&error)
        ));
    }

    #[test]
    fn ref_token_with_extensions_accepts_base_only_data() {
        let data = vec![0u8; TokenAccount::BASE_LEN];
        let (_backing, account_view) = build_account_view(&ID, &data);

        let token =
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).unwrap();
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
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
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
        let (_backing, account_view) = build_account_view(&ID, &data);

        let token =
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).unwrap();
        assert!(token
            .get_extension::<TransferHookAccountExtension>()
            .unwrap()
            .transferring());
    }

    #[test]
    fn get_extension_mut_returns_not_found_when_absent() {
        let mint_data = vec![0u8; Mint::BASE_LEN];
        let (_mint_backing, mint_view) = build_account_view(&ID, &mint_data);
        let mut mint = RefMutStateWithExtensions::<Mint>::from_account_view(&mint_view).unwrap();
        assert!(is_extension_not_found_error(
            &mint
                .get_extension_mut::<DefaultAccountStateExtension>()
                .unwrap_err()
        ));

        let token_data = vec![0u8; TokenAccount::BASE_LEN];
        let (_token_backing, token_view) = build_account_view(&ID, &token_data);
        let mut token =
            RefMutStateWithExtensions::<TokenAccount>::from_account_view(&token_view).unwrap();
        assert!(is_extension_not_found_error(
            &token
                .get_extension_mut::<TransferHookAccountExtension>()
                .unwrap_err()
        ));
    }

    #[test]
    fn get_extension_propagates_corrupt_tlv_data() {
        let corrupt_mint_data = build_mint_data(&[0u8]);
        let (_mint_backing, mint_view) = build_account_view(&ID, &corrupt_mint_data);
        let mint = RefStateWithExtensions::<Mint>::from_account_view(&mint_view).unwrap();
        assert!(matches!(
            mint.get_extension::<DefaultAccountStateExtension>(),
            Err(ProgramError::InvalidAccountData)
        ));

        let corrupt_token_data = build_token_data(&[0u8]);
        let (_token_backing, token_view) = build_account_view(&ID, &corrupt_token_data);
        let token = RefStateWithExtensions::<TokenAccount>::from_account_view(&token_view).unwrap();
        assert!(matches!(
            token.get_extension::<TransferHookAccountExtension>(),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn ref_mint_with_extensions_rejects_multisig_len_collision() {
        let mut data = vec![0u8; Multisig::LEN];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;
        let (_backing, account_view) = build_account_view(&ID, &data);

        assert!(matches!(
            RefStateWithExtensions::<Mint>::from_account_view(&account_view),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn ref_token_with_extensions_rejects_multisig_len_collision() {
        let mut data = vec![0u8; Multisig::LEN];
        data[ACCOUNT_TYPE_INDEX] = AccountType::Account as u8;
        let (_backing, account_view) = build_account_view(&ID, &data);

        assert!(matches!(
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view),
            Err(ProgramError::InvalidAccountData)
        ));
    }

    #[test]
    fn ref_token_with_extensions_rejects_wrong_account_type() {
        let mut data = build_token_data(&[]);
        data[ACCOUNT_TYPE_INDEX] = AccountType::Mint as u8;
        let (_backing, account_view) = build_account_view(&ID, &data);

        assert!(matches!(
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view),
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
    fn ref_mint_with_extensions_enforces_borrow_rules() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        let read_ref = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        assert!(RefMutStateWithExtensions::<Mint>::from_account_view(&account_view).is_err());
        drop(read_ref);

        let mut write_ref =
            RefMutStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        assert!(RefStateWithExtensions::<Mint>::from_account_view(&account_view).is_err());
        write_ref
            .get_extension_mut::<DefaultAccountStateExtension>()
            .unwrap()
            .set_state(AccountState::Frozen);
        drop(write_ref);

        assert!(RefStateWithExtensions::<Mint>::from_account_view(&account_view).is_ok());
    }

    #[test]
    fn ref_token_with_extensions_enforces_borrow_rules() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHookAccount, &[0u8]);
        let data = build_token_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        let read_ref =
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).unwrap();
        assert!(
            RefMutStateWithExtensions::<TokenAccount>::from_account_view(&account_view).is_err()
        );
        drop(read_ref);

        let mut write_ref =
            RefMutStateWithExtensions::<TokenAccount>::from_account_view(&account_view).unwrap();
        assert!(RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).is_err());
        write_ref
            .get_extension_mut::<TransferHookAccountExtension>()
            .unwrap()
            .set_transferring(true);
        drop(write_ref);

        assert!(RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).is_ok());
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
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mut token = unsafe {
            StateWithExtensionsMut::<TokenAccount>::from_account_view_unchecked(&account_view)
        }
        .unwrap();
        let extension = token
            .get_extension_mut::<TransferHookAccountExtension>()
            .unwrap();
        assert!(!extension.transferring());

        extension.set_transferring(true);
        assert!(extension.transferring());

        extension.set_transferring(false);
        assert!(!extension.transferring());
    }

    #[test]
    fn mint_default_account_state_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(
            &mut tlv_data,
            ExtensionType::DefaultAccountState,
            &[AccountState::Initialized as u8],
        );
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        {
            let mut mint =
                RefMutStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
            mint.get_extension_mut::<DefaultAccountStateExtension>()
                .unwrap()
                .set_state(AccountState::Frozen);
        }

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
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
        let data = build_token_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        {
            let mut token =
                RefMutStateWithExtensions::<TokenAccount>::from_account_view(&account_view)
                    .unwrap();
            token
                .get_extension_mut::<TransferHookAccountExtension>()
                .unwrap()
                .set_transferring(true);
        }

        let token =
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).unwrap();
        assert!(token
            .get_extension::<TransferHookAccountExtension>()
            .unwrap()
            .transferring());
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
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        let mut out = [ExtensionType::Uninitialized; 2];
        let written = mint.get_extension_types(&mut out).unwrap();

        assert_eq!(written, 2);
        assert_eq!(out[0], ExtensionType::PermanentDelegate);
        assert_eq!(out[1], ExtensionType::DefaultAccountState);
    }

    #[test]
    fn get_extension_types_on_base_only_returns_empty() {
        let data = vec![0u8; TokenAccount::BASE_LEN];
        let (_backing, account_view) = build_account_view(&ID, &data);

        let token =
            RefStateWithExtensions::<TokenAccount>::from_account_view(&account_view).unwrap();
        let mut out = [ExtensionType::Uninitialized; 1];
        let written = token.get_extension_types(&mut out).unwrap();

        assert_eq!(written, 0);
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
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        let mut out = [ExtensionType::Uninitialized; 1];
        assert_eq!(
            mint.get_extension_types(&mut out),
            Err(ProgramError::AccountDataTooSmall)
        );
    }

    #[test]
    fn try_calculate_account_len_matches_supported_layouts() {
        assert_eq!(
            try_calculate_account_len::<Mint>(&[]).unwrap(),
            Mint::BASE_LEN
        );
        assert_eq!(
            try_calculate_account_len::<TokenAccount>(&[]).unwrap(),
            TokenAccount::BASE_LEN
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::DefaultAccountState]).unwrap(),
            TLV_START_INDEX + TLV_HEADER_LEN + DefaultAccountStateExtension::LEN
        );
        assert_eq!(
            try_calculate_account_len::<TokenAccount>(&[ExtensionType::TransferHookAccount])
                .unwrap(),
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
    }

    #[test]
    fn try_calculate_account_len_rejects_wrong_or_unsupported_extensions() {
        assert_eq!(
            try_calculate_account_len::<Mint>(&[ExtensionType::TransferHookAccount]),
            Err(ProgramError::InvalidAccountData)
        );
        assert_eq!(
            try_calculate_account_len::<Mint>(&[
                ExtensionType::DefaultAccountState,
                ExtensionType::DefaultAccountState,
            ]),
            Ok(TLV_START_INDEX + TLV_HEADER_LEN + DefaultAccountStateExtension::LEN)
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
        assert_eq!(ExtensionType::DefaultAccountState as u16, 6);
        assert_eq!(ExtensionType::TransferHookAccount as u16, 15);
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
    }

    #[test]
    fn spl2022_supported_extension_sizes_match() {
        assert_eq!(DefaultAccountStateExtension::LEN, 1);
        assert_eq!(TransferHookAccountExtension::LEN, 1);
        assert_eq!(PermanentDelegateExtension::LEN, 32);
        assert_eq!(TransferHookExtension::LEN, 64);
    }

    #[test]
    fn spl2022_supported_extension_alignments_match() {
        assert_eq!(core::mem::align_of::<DefaultAccountStateExtension>(), 1);
        assert_eq!(core::mem::align_of::<TransferHookAccountExtension>(), 1);
        assert_eq!(core::mem::align_of::<PermanentDelegateExtension>(), 1);
        assert_eq!(core::mem::align_of::<TransferHookExtension>(), 1);
    }

    #[test]
    fn permanent_delegate_extension_read_roundtrip() {
        let delegate = [42u8; 32];
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &delegate);
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        let ext = mint.get_extension::<PermanentDelegateExtension>().unwrap();
        assert_eq!(ext.delegate().as_ref(), &delegate);
    }

    #[test]
    fn transfer_hook_extension_read_roundtrip() {
        let mut value = [0u8; 64];
        value[..32].copy_from_slice(&[11u8; 32]);
        value[32..64].copy_from_slice(&[22u8; 32]);
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHook, &value);
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        let ext = mint.get_extension::<TransferHookExtension>().unwrap();
        assert_eq!(ext.authority().as_ref(), &[11u8; 32]);
        assert_eq!(ext.program_id().as_ref(), &[22u8; 32]);
    }

    #[test]
    fn permanent_delegate_extension_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::PermanentDelegate, &[0u8; 32]);
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        let new_delegate = Address::new_from_array([99u8; 32]);
        {
            let mut mint =
                RefMutStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
            mint.get_extension_mut::<PermanentDelegateExtension>()
                .unwrap()
                .set_delegate(&new_delegate);
        }

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        assert_eq!(
            mint.get_extension::<PermanentDelegateExtension>()
                .unwrap()
                .delegate(),
            &new_delegate,
        );
    }

    #[test]
    fn transfer_hook_extension_write_then_read_roundtrip() {
        let mut tlv_data = Vec::new();
        push_tlv_entry(&mut tlv_data, ExtensionType::TransferHook, &[0u8; 64]);
        let data = build_mint_data(&tlv_data);
        let (_backing, account_view) = build_account_view(&ID, &data);

        let new_authority = Address::new_from_array([77u8; 32]);
        let new_program_id = Address::new_from_array([88u8; 32]);
        {
            let mut mint =
                RefMutStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
            let ext = mint.get_extension_mut::<TransferHookExtension>().unwrap();
            ext.set_authority(&new_authority);
            ext.set_program_id(&new_program_id);
        }

        let mint = RefStateWithExtensions::<Mint>::from_account_view(&account_view).unwrap();
        let ext = mint.get_extension::<TransferHookExtension>().unwrap();
        assert_eq!(ext.authority(), &new_authority);
        assert_eq!(ext.program_id(), &new_program_id);
    }
}
