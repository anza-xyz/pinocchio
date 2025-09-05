use super::AccountState;
use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    extensions::{BaseState, BaseStateVariant, EXTENSION_START_PADDING},
    ID,
};

/// Token account data.
#[repr(C)]
pub struct TokenAccount {
    /// The mint associated with this account
    mint: Pubkey,

    /// The owner of this account.
    owner: Pubkey,

    /// The amount of tokens this account holds.
    amount: [u8; 8],

    /// Indicates whether the delegate is present or not.
    delegate_flag: [u8; 4],

    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate.
    delegate: Pubkey,

    /// The account's state.
    state: u8,

    /// Indicates whether this account represents a native token or not.
    is_native: [u8; 4],

    /// When `is_native.is_some()` is `true`, this is a native token, and the
    /// value logs the rent-exempt reserve. An Account is required to be rent-exempt,
    /// so the value is used by the Processor to ensure that wrapped SOL
    /// accounts do not drop below this threshold.
    native_amount: [u8; 8],

    /// The amount delegated.
    delegated_amount: [u8; 8],

    /// Indicates whether the close authority is present or not.
    close_authority_flag: [u8; 4],

    /// Optional authority to close the account.
    close_authority: Pubkey,
}

impl TokenAccount {
    pub const BASE_LEN: usize = core::mem::size_of::<TokenAccount>();

    /// Return a `TokenAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<TokenAccount>, ProgramError> {
        if account_info.data_len() < Self::BASE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `TokenAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data (e.g., there are
    /// no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&TokenAccount, ProgramError> {
        if account_info.data_len() < Self::BASE_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::from_bytes_unchecked(
            account_info.borrow_data_unchecked(),
        ))
    }

    /// Return a `TokenAccount` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `TokenAccount`, and
    /// it is properly aligned to be interpreted as an instance of `TokenAccount`.
    /// At the moment `TokenAccount` has an alignment of 1 byte.
    /// This method does not perform a length validation.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[..Self::BASE_LEN].as_ptr() as *const TokenAccount)
    }

    pub fn mint(&self) -> &Pubkey {
        &self.mint
    }

    pub fn owner(&self) -> &Pubkey {
        &self.owner
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline(always)]
    pub fn has_delegate(&self) -> bool {
        self.delegate_flag[0] == 1
    }

    pub fn delegate(&self) -> Option<&Pubkey> {
        if self.has_delegate() {
            Some(self.delegate_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a delegate and want to skip the `Option` check.
    #[inline(always)]
    pub fn delegate_unchecked(&self) -> &Pubkey {
        &self.delegate
    }

    #[inline(always)]
    pub fn state(&self) -> AccountState {
        self.state.into()
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
    /// This method should be used when the caller knows that the token is native since it
    /// skips the `Option` check.
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

    pub fn close_authority(&self) -> Option<&Pubkey> {
        if self.has_close_authority() {
            Some(self.close_authority_unchecked())
        } else {
            None
        }
    }

    /// Return the close authority.
    ///
    /// This method should be used when the caller knows that the token will have a close
    /// authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn close_authority_unchecked(&self) -> &Pubkey {
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

impl BaseState for TokenAccount {
    const BASE_STATE: BaseStateVariant = BaseStateVariant::TokenAccount;
    const LEN: usize = TokenAccount::BASE_LEN;
    fn extension_data_start_index() -> Option<usize> {
        Some(TokenAccount::LEN + EXTENSION_START_PADDING)
    }
}
