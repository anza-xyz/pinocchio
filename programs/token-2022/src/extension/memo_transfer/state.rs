use {
    crate::ID,
    pinocchio::{
        account_info::{AccountInfo, Ref},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[repr(u8)]
pub enum InstructionDiscriminatorMemoTransfer {
    Enable = 0,
    Disable = 1,
}

/// Instruction data layout:
/// - [0]                        : Extension discriminator (1 byte)
/// - [1]                        : Instruction discriminator (1 byte)
/// dev: Since only the instruction discriminator is used to toggle memo transfer states
/// and no additional parameters are required, `START == END`.
pub mod offset_memo_transfer {
    pub const START: u8 = 2;
    pub const END: u8 = START;
}

/// Models onchain `MemoTransfer` state.
/// Mirrors SPL Token-2022:
/// `pub struct MemoTransfer { pub require_incoming_transfer_memos: PodBool }`
#[repr(C)]
pub struct MemoTransfer {
    /// Indicates whether incoming transfers must include a memo.
    pub require_incoming_transfer_memos: bool,
}

impl MemoTransfer {
    /// The length of the token_account with `MemoTransfer` extension data
    const LEN: u8 = 171;

    /// The byte index where the `require_incoming_transfer_memos` flag is stored
    /// within a Token-2022 account initialized with the `MemoTransfer` extension.
    const REQUIRE_MEMO_FLAG_INDEX: u8 = 170;

    /// Init with given memo requirement flag.
    #[inline]
    pub fn new(require_memo: bool) -> Self {
        Self {
            require_incoming_transfer_memos: require_memo,
        }
    }

    /// Return a `MemoTransfer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<MemoTransfer>, ProgramError> {
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

    /// Return a `MemoTransfer` from the given account info.
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

    /// Return a `MemoTransfer` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `MemoTransfer`
    /// 3. The data is properly aligned (though MemoTransfer has alignment of 1)
    /// 4. The bytes represent valid flag values and pubkey data
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::REQUIRE_MEMO_FLAG_INDEX as usize..].as_ptr() as *const MemoTransfer)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Returns true if memo transfers are enabled.
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        self.require_incoming_transfer_memos
    }

    /// Returns true if memo transfers are disabled.
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        !self.require_incoming_transfer_memos
    }
}
