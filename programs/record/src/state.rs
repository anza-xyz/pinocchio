use solana_address::Address;

/// Header type for recorded account data
#[repr(C)]
pub struct RecordData {
    /// Struct version, allows for upgrades to the program
    version: u8,

    /// The account allowed to update the data
    authority: Address,
}

impl RecordData {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u8 = 1;

    /// Start of writable account data, after version and authority
    pub const WRITABLE_START_INDEX: usize = 33;

    /// Return a `RecordData` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `RecordData`, and
    /// it is properly aligned to be interpreted as an instance of `RecordData`.
    /// At the moment `RecordData` has an alignment of 1 byte.
    /// This method does not perform a length validation.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const RecordData)
    }

    #[inline(always)]
    pub const fn authority(&self) -> &Address {
        &self.authority
    }

    #[inline(always)]
    pub const fn version(&self) -> u8 {
        self.version
    }
}
