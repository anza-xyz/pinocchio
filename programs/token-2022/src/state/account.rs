use {
    super::AccountState,
    crate::{
        state::{validate_account_type, AccountType},
        ID,
    },
    solana_account_view::{AccountView, Ref},
    solana_address::Address,
    solana_program_error::ProgramError,
};

/// Token account data.
#[repr(C)]
pub struct Account {
    /// The mint associated with this account
    mint: Address,

    /// The owner of this account.
    owner: Address,

    /// The amount of tokens this account holds.
    amount: [u8; 8],

    /// Indicates whether the delegate is present or not.
    delegate_flag: [u8; 4],

    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate.
    delegate: Address,

    /// The account's state.
    state: u8,

    /// Indicates whether this account represents a native token or not.
    is_native: [u8; 4],

    /// When `is_native.is_some()` is `true`, this is a native token, and the
    /// value logs the rent-exempt reserve. An Account is required to be
    /// rent-exempt, so the value is used by the Processor to ensure that
    /// wrapped SOL accounts do not drop below this threshold.
    native_amount: [u8; 8],

    /// The amount delegated.
    delegated_amount: [u8; 8],

    /// Indicates whether the close authority is present or not.
    close_authority_flag: [u8; 4],

    /// Optional authority to close the account.
    close_authority: Address,
}

impl Account {
    pub const BASE_LEN: usize = core::mem::size_of::<Account>();

    /// Return an `Account` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`, safe
    /// borrowing the account data.
    #[inline]
    pub fn from_account_view(account_view: &AccountView) -> Result<Ref<'_, Account>, ProgramError> {
        if !account_view.owned_by(&ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let bytes = account_view.try_borrow()?;
        validate_account_type(&bytes, AccountType::Account, Self::BASE_LEN)?;

        Ok(Ref::map(bytes, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return an `Account` from the given account view.
    ///
    /// This method performs owner and length validation on `AccountView`, but
    /// does not perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data (e.g.,
    /// there are no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_view_unchecked(
        account_view: &AccountView,
    ) -> Result<&Account, ProgramError> {
        if account_view.owner() != &ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        let bytes = account_view.borrow_unchecked();
        validate_account_type(bytes, AccountType::Account, Self::BASE_LEN)?;

        Ok(Self::from_bytes_unchecked(bytes))
    }

    /// Return an `Account` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of
    /// `Account`, and it is properly aligned to be interpreted as an
    /// instance of `Account`. At the moment `Account` has an
    /// alignment of 1 byte. This method does not perform a length
    /// validation.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[..Self::BASE_LEN].as_ptr() as *const Account)
    }

    #[inline(always)]
    pub(crate) unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes[..Self::BASE_LEN].as_mut_ptr() as *mut Account)
    }

    pub fn mint(&self) -> &Address {
        &self.mint
    }

    pub fn owner(&self) -> &Address {
        &self.owner
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline(always)]
    pub fn has_delegate(&self) -> bool {
        self.delegate_flag[0] == 1
    }

    pub fn delegate(&self) -> Option<&Address> {
        if self.has_delegate() {
            Some(self.delegate_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a delegate and want to skip
    /// the `Option` check.
    #[inline(always)]
    pub fn delegate_unchecked(&self) -> &Address {
        &self.delegate
    }

    #[inline(always)]
    pub fn state(&self) -> Result<AccountState, ProgramError> {
        AccountState::try_from(self.state)
    }

    #[inline(always)]
    pub fn is_native(&self) -> bool {
        self.is_native[0] == 1
    }

    pub fn native_amount(&self) -> Option<u64> {
        if self.is_native() {
            Some(self.native_amount_unchecked())
        } else {
            None
        }
    }

    /// Return the native amount.
    ///
    /// This method should be used when the caller knows that the token is
    /// native since it skips the `Option` check.
    #[inline(always)]
    pub fn native_amount_unchecked(&self) -> u64 {
        u64::from_le_bytes(self.native_amount)
    }

    pub fn delegated_amount(&self) -> u64 {
        u64::from_le_bytes(self.delegated_amount)
    }

    #[inline(always)]
    pub fn has_close_authority(&self) -> bool {
        self.close_authority_flag[0] == 1
    }

    pub fn close_authority(&self) -> Option<&Address> {
        if self.has_close_authority() {
            Some(self.close_authority_unchecked())
        } else {
            None
        }
    }

    /// Return the close authority.
    ///
    /// This method should be used when the caller knows that the token will
    /// have a close authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn close_authority_unchecked(&self) -> &Address {
        &self.close_authority
    }

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized as u8
    }

    #[inline(always)]
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen as u8
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use {
        super::*,
        core::{mem::size_of, ptr::copy_nonoverlapping},
        solana_account_view::{RuntimeAccount, NOT_BORROWED},
        solana_address::Address,
        std::{vec, vec::Vec},
    };

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
            (*raw).padding = [0; 4];
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
    fn from_account_view_rejects_wrong_owner() {
        let wrong_owner = Address::new_from_array([7u8; 32]);
        let data = vec![0u8; Account::BASE_LEN];
        let (_backing, account_view) = build_account_view(&wrong_owner, &data);

        assert!(matches!(
            Account::from_account_view(&account_view),
            Err(ProgramError::IncorrectProgramId)
        ));
    }

    #[test]
    fn from_account_view_unchecked_rejects_wrong_owner() {
        let wrong_owner = Address::new_from_array([7u8; 32]);
        let data = vec![0u8; Account::BASE_LEN];
        let (_backing, account_view) = build_account_view(&wrong_owner, &data);

        assert!(matches!(
            unsafe { Account::from_account_view_unchecked(&account_view) },
            Err(ProgramError::IncorrectProgramId)
        ));
    }
}
