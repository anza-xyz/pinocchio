use {
    crate::{write_bytes, ID, UNINIT_BYTE},
    core::mem::MaybeUninit,
    pinocchio::{
        account_info::{AccountInfo, Ref},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterestBearingMintInstruction {
    Initialize = 0,
    UpdateRate = 1,
}

#[repr(C, packed)]
pub struct InterestBearingConfig {
    /// Authority that can set the interest rate
    rate_authority: Pubkey,
    /// Initialization timestamp
    initialization_timestamp: i64,
    /// Pre-update average rate
    pre_update_average_rate: i16,
    /// Last update timestamp
    last_update_timestamp: i64,
    /// Current interest rate
    current_rate: i16,
}

impl InterestBearingConfig {
    /// The index where rate_authority address starts in the mint with `InterestBearingConfig` extension data
    pub const RATE_AUTHORITY_START: u8 = 170;

    /// The length of the `InterestBearingConfig` extension data.
    pub const BASE_LEN: usize = core::mem::size_of::<InterestBearingConfig>();

    pub const LEN: usize = Self::BASE_LEN as usize + Self::RATE_AUTHORITY_START as usize;

    /// Return a `InterestBearingConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<InterestBearingConfig>, ProgramError> {
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

    /// Return a `InterestBearingConfig` from the given account info.
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

    /// Return a `InterestBearingConfig` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `InterestBearingConfig`
    /// 3. The data is properly aligned (though InterestBearingConfig has alignment of 1)
    /// 4. The bytes represent valid flag values and pubkey data
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::RATE_AUTHORITY_START as usize..].as_ptr() as *const InterestBearingConfig)
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
    pub fn new(rate_authority: Option<&Pubkey>, rate: i16) -> Self {
        Self {
            rate_authority: rate_authority.map(|&x| x).unwrap_or_default(),
            initialization_timestamp: 0,
            last_update_timestamp: 0,
            pre_update_average_rate: rate,
            current_rate: rate,
        }
    }

    #[inline(always)]
    pub fn has_rate_authority(&self) -> bool {
        self.rate_authority != Pubkey::default()
    }

    #[inline]
    pub fn rate_authority(&self) -> Option<&Pubkey> {
        if self.has_rate_authority() {
            Some(&self.rate_authority)
        } else {
            None
        }
    }

    /// Return the rate authority.
    ///
    /// This method should be used when the caller knows that the interest bearing config will have a
    /// rate authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn rate_authority_unchecked(&self) -> &Pubkey {
        &self.rate_authority
    }

    #[inline(always)]
    pub fn initialization_timestamp(&self) -> i64 {
        self.initialization_timestamp
    }

    #[inline(always)]
    pub fn last_update_timestamp(&self) -> i64 {
        self.last_update_timestamp
    }

    #[inline(always)]
    pub fn pre_update_average_rate(&self) -> i16 {
        self.pre_update_average_rate
    }

    #[inline(always)]
    pub fn current_rate(&self) -> i16 {
        self.current_rate
    }
}

pub fn interest_bearing_mint_initialize_instruction_data(
    rate_authority: Option<&Pubkey>,
    rate: i16,
) -> [MaybeUninit<u8>; 36] {
    // instruction data
    // -  [0]: extension discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..34]: rate_authority pubkey (32 bytes, optional)
    // -  [34..36]: rate (2 bytes, i16)

    let mut data = [UNINIT_BYTE; 36];

    // Set extension discriminator at offset [0]
    write_bytes(
        &mut data,
        &[crate::extension::consts::ExtensionDiscriminator::InterestBearingMint as u8],
    );
    // Set sub-instruction at offset [1]
    write_bytes(
        &mut data[1..2],
        &[InterestBearingMintInstruction::Initialize as u8],
    );
    // Set rate_authority at offset [2..34]
    if let Some(auth) = rate_authority {
        write_bytes(&mut data[2..34], auth);
    } else {
        write_bytes(&mut data[2..34], &[0u8; 32]);
    }
    // Set rate at offset [34..36]
    write_bytes(&mut data[34..36], &rate.to_le_bytes());

    data
}

pub fn interest_bearing_mint_update_rate_instruction_data(rate: i16) -> [MaybeUninit<u8>; 4] {
    // instruction data
    // -  [0]: extension discriminator (1 byte, u8)
    // -  [1]: instruction_type (1 byte, u8)
    // -  [2..4]: rate (2 bytes, i16)

    let mut data = [UNINIT_BYTE; 4];

    // Set extension discriminator at offset [0]
    write_bytes(
        &mut data,
        &[crate::extension::consts::ExtensionDiscriminator::InterestBearingMint as u8],
    );
    // Set sub-instruction at offset [1]
    write_bytes(
        &mut data[1..2],
        &[InterestBearingMintInstruction::UpdateRate as u8],
    );
    // Set rate at offset [2..4]
    write_bytes(&mut data[2..4], &rate.to_le_bytes());

    data
}
