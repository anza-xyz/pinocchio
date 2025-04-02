use pinocchio::{
    account_view::{AccountView, Ref},
    error::ProgramError,
    Address,
};

use crate::ID;

/// Mint data.
#[repr(C)]
pub struct Mint {
    /// Indicates whether the mint authority is present or not.
    mint_authority_flag: [u8; 4],

    /// Optional authority used to mint new tokens. The mint authority may only
    /// be provided during mint creation. If no mint authority is present
    /// then the mint has a fixed supply and no further tokens may be
    /// minted.
    mint_authority: Address,

    /// Total supply of tokens.
    supply: [u8; 8],

    /// Number of base 10 digits to the right of the decimal place.
    decimals: u8,

    /// Is `true` if this structure has been initialized.
    is_initialized: u8,

    /// Indicates whether the freeze authority is present or not.
    freeze_authority_flag: [u8; 4],

    /// Optional authority to freeze token accounts.
    freeze_authority: Address,
}

impl Mint {
    /// The length of the `Mint` account data.
    pub const LEN: usize = core::mem::size_of::<Mint>();

    /// Return a `Mint` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountView`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(account_info: &AccountView) -> Result<Ref<Mint>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account_info.is_owned_by(&ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `Mint` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountView`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data (e.g., there are
    /// no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountView,
    ) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Self::from_bytes_unchecked(
            account_info.borrow_data_unchecked(),
        ))
    }

    /// Return a `Mint` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Mint`, and
    /// it is properly aligned to be interpreted as an instance of `Mint`.
    /// At the moment `Mint` has an alignment of 1 byte.
    /// This method does not perform a length validation.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Mint)
    }

    #[inline(always)]
    pub fn has_mint_authority(&self) -> bool {
        self.mint_authority_flag[0] == 1
    }

    pub fn mint_authority(&self) -> Option<&Address> {
        if self.has_mint_authority() {
            Some(self.mint_authority_unchecked())
        } else {
            None
        }
    }

    /// Return the mint authority.
    ///
    /// This method should be used when the caller knows that the mint will have a mint
    /// authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn mint_authority_unchecked(&self) -> &Address {
        &self.mint_authority
    }

    pub fn supply(&self) -> u64 {
        u64::from_le_bytes(self.supply)
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized == 1
    }

    #[inline(always)]
    pub fn has_freeze_authority(&self) -> bool {
        self.freeze_authority_flag[0] == 1
    }

    pub fn freeze_authority(&self) -> Option<&Address> {
        if self.has_freeze_authority() {
            Some(self.freeze_authority_unchecked())
        } else {
            None
        }
    }

    /// Return the freeze authority.
    ///
    /// This method should be used when the caller knows that the mint will have a freeze
    /// authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn freeze_authority_unchecked(&self) -> &Address {
        &self.freeze_authority
    }
}
