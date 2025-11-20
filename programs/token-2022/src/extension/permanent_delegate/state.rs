use {
    crate::ID,
    pinocchio::{
        account_info::{AccountInfo, Ref},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1..33]                    : permanent delegate pubkey (32 bytes)
pub mod offset_permanent_delegate_initialize {
    pub const START: u8 = 1;
    pub const PERMANENT_DELEGATE_PUBKEY: u8 = 32;
    pub const END: u8 = START + PERMANENT_DELEGATE_PUBKEY;
}

/// Permanent delegate extension data for mints.
#[repr(C)]
pub struct PermanentDelegate {
    /// Optional permanent delegate for transferring or burning tokens
    delegate: Pubkey,
}

impl PermanentDelegate {
    /// The length of the mint with `PermanentDelegate` extension data
    const LEN: u8 = 202;
    /// The index where delegate address starts in the mint with `PermanentDelegate` extension data
    const DELEGATE_START: u8 = 170;

    /// The length of the `PermanentDelegate` extension data.
    pub const BASE_LEN: usize = core::mem::size_of::<PermanentDelegate>();

    /// Return a `PermanentDelegate` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<PermanentDelegate>, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        // Check owner
        if account_info.owner() != &ID {
            Err(ProgramError::InvalidAccountOwner)?;
        }

        // Safely borrow and map the data
        let data_ref = account_info
            .try_borrow_data()
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        Ok(Ref::map(data_ref, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `PermanentDelegate` from the given account info.
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
    ) -> Result<&Self, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        // Check owner
        if account_info.owner() != &ID {
            Err(ProgramError::InvalidAccountOwner)?;
        }

        // Get unchecked borrow and convert
        let data = account_info.borrow_data_unchecked();
        Ok(Self::from_bytes_unchecked(data))
    }

    /// Return a `PermanentDelegate` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `PermanentDelegate`
    /// 3. The data is properly aligned (though PermanentDelegate has alignment of 1)
    /// 4. The bytes represent valid flag values and pubkey data
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::DELEGATE_START as usize..].as_ptr() as *const PermanentDelegate)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates a new state
    pub fn new(delegate: Option<&Pubkey>) -> Self {
        Self {
            delegate: delegate.map(|&x| x).unwrap_or_default(),
        }
    }

    #[inline(always)]
    pub fn has_delegate(&self) -> bool {
        self.delegate != Pubkey::default()
    }

    #[inline]
    pub fn delegate(&self) -> Option<&Pubkey> {
        if self.has_delegate() {
            Some(&self.delegate)
        } else {
            None
        }
    }

    /// Return the delegate.
    ///
    /// This method should be used when the caller knows that the permanent delegate will have an
    /// delegate set since it skips the `Option` check.
    #[inline(always)]
    pub fn delegate_unchecked(&self) -> &Pubkey {
        &self.delegate
    }
}
