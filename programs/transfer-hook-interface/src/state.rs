use solana_program_error::ProgramError;

/// An extra account meta entry as stored in the extra-account-metas PDA.
///
/// This is a 35-byte `#[repr(C)]` type that is binary-compatible with the
/// `ExtraAccountMeta` type in `spl-tlv-account-resolution`.
///
/// ## Layout
///
/// | Offset | Size | Field              |
/// |--------|------|--------------------|
/// | 0      | 1    | `discriminator`    |
/// | 1      | 32   | `address_config`   |
/// | 33     | 1    | `is_signer`        |
/// | 34     | 1    | `is_writable`      |
///
/// ## Discriminator values
///
/// - `0` — Standard account meta. `address_config` is a 32-byte pubkey.
/// - `1` — PDA derived from the hook program. `address_config` contains packed
///   [`Seed`] entries.
/// - `128..=255` — PDA derived from an external program at index `discriminator
///   - 128`. `address_config` contains packed [`Seed`] entries.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtraAccountMeta {
    /// 0 = standard pubkey, 1 = hook-program PDA, 128+ = external PDA.
    pub discriminator: u8,
    /// Either a raw 32-byte pubkey (disc 0) or packed seed config (disc
    /// 1 / 128+).
    pub address_config: [u8; 32],
    /// Whether this account must sign the transaction.
    pub is_signer: u8,
    /// Whether this account is writable.
    pub is_writable: u8,
}

/// The byte size of a single [`ExtraAccountMeta`].
pub const EXTRA_ACCOUNT_META_SIZE: usize = 35;

const _: () = assert!(core::mem::size_of::<ExtraAccountMeta>() == EXTRA_ACCOUNT_META_SIZE);

impl ExtraAccountMeta {
    /// Create an entry for a fixed account (discriminator 0).
    #[inline]
    pub const fn new_with_pubkey(pubkey: &[u8; 32], is_signer: bool, is_writable: bool) -> Self {
        Self {
            discriminator: 0,
            address_config: *pubkey,
            is_signer: is_signer as u8,
            is_writable: is_writable as u8,
        }
    }

    /// Create an entry for a PDA derived from the hook program
    /// (discriminator 1).
    ///
    /// `seeds` are packed into the 32-byte `address_config` field using
    /// [`Seed::pack_into_address_config`].
    #[inline]
    pub fn new_with_seeds(
        seeds: &[Seed],
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            discriminator: 1,
            address_config: Seed::pack_into_address_config(seeds)?,
            is_signer: is_signer as u8,
            is_writable: is_writable as u8,
        })
    }

    /// Create an entry for a PDA derived from an external program
    /// (discriminator 128+).
    ///
    /// `program_index` is the index of the program account in the
    /// *entire* accounts list (fixed + extra).
    #[inline]
    pub fn new_external_pda_with_seeds(
        program_index: u8,
        seeds: &[Seed],
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            discriminator: program_index
                .checked_add(128)
                .ok_or(ProgramError::InvalidArgument)?,
            address_config: Seed::pack_into_address_config(seeds)?,
            is_signer: is_signer as u8,
            is_writable: is_writable as u8,
        })
    }
}

/// A seed component used to derive PDA-based extra account metas.
///
/// Seeds are packed into a 32-byte `address_config` field using a
/// compact TLV encoding. Each variant has a 1-byte type discriminator
/// followed by its parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seed {
    /// A hard-coded literal byte string.
    ///
    /// Encoding: `[1, length, ...bytes]`
    Literal { bytes: [u8; 32], length: u8 },

    /// A slice of the instruction data.
    ///
    /// Encoding: `[2, index, length]`
    InstructionData { index: u8, length: u8 },

    /// The public key of an account at the given index in the accounts
    /// list.
    ///
    /// Encoding: `[3, index]`
    AccountKey { index: u8 },

    /// A slice of an account's data.
    ///
    /// Encoding: `[4, account_index, data_index, length]`
    AccountData {
        account_index: u8,
        data_index: u8,
        length: u8,
    },
}

impl Seed {
    /// Byte size of this seed when packed (discriminator + payload).
    #[inline]
    pub const fn tlv_size(&self) -> usize {
        match self {
            Self::Literal { length, .. } => 2 + *length as usize,
            Self::InstructionData { .. } => 3,
            Self::AccountKey { .. } => 2,
            Self::AccountData { .. } => 4,
        }
    }

    /// Pack this seed into `dst`. The caller must ensure `dst.len() >=
    /// self.tlv_size()`.
    pub fn pack(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        match self {
            Self::Literal { bytes, length } => {
                let len = *length as usize;
                if dst.len() < 2 + len {
                    return Err(ProgramError::InvalidArgument);
                }
                dst[0] = 1;
                dst[1] = *length;
                dst[2..2 + len].copy_from_slice(&bytes[..len]);
            }
            Self::InstructionData { index, length } => {
                if dst.len() < 3 {
                    return Err(ProgramError::InvalidArgument);
                }
                dst[0] = 2;
                dst[1] = *index;
                dst[2] = *length;
            }
            Self::AccountKey { index } => {
                if dst.len() < 2 {
                    return Err(ProgramError::InvalidArgument);
                }
                dst[0] = 3;
                dst[1] = *index;
            }
            Self::AccountData {
                account_index,
                data_index,
                length,
            } => {
                if dst.len() < 4 {
                    return Err(ProgramError::InvalidArgument);
                }
                dst[0] = 4;
                dst[1] = *account_index;
                dst[2] = *data_index;
                dst[3] = *length;
            }
        }
        Ok(())
    }

    /// Pack multiple seeds into a 32-byte `address_config` field.
    pub fn pack_into_address_config(seeds: &[Self]) -> Result<[u8; 32], ProgramError> {
        let mut packed = [0u8; 32];
        let mut offset: usize = 0;
        for seed in seeds {
            let size = seed.tlv_size();
            let end = offset + size;
            if end > 32 {
                return Err(ProgramError::InvalidArgument);
            }
            seed.pack(&mut packed[offset..end])?;
            offset = end;
        }
        Ok(packed)
    }
}

/// Helpers for reading and writing the TLV-encoded extra-account-metas
/// PDA data.
///
/// The on-chain format stored in the extra-account-metas PDA is:
///
/// ```text
/// [8-byte type discriminator][4-byte length]
///   [4-byte count][35 × count ExtraAccountMeta entries]
/// ```
///
/// The 8-byte type discriminator identifies which instruction the extra
/// metas are for. For Transfer Hook `Execute`, this is
/// [`EXECUTE_DISCRIMINATOR`](crate::EXECUTE_DISCRIMINATOR).
/// The 4-byte length is the byte size of the value section
/// (count + entries). Inside the value, a 4-byte little-endian count
/// precedes the array of 35-byte entries.
pub struct ExtraAccountMetaList;

impl ExtraAccountMetaList {
    /// TLV header size: 8-byte type discriminator + 4-byte length.
    const TLV_HEADER_SIZE: usize = 12;

    /// PodSlice header size: 4-byte count.
    const POD_SLICE_HEADER_SIZE: usize = 4;

    /// Compute the total account data size needed to store `num_items`
    /// extra account metas in the PDA.
    #[inline]
    pub const fn size_of(num_items: usize) -> usize {
        Self::TLV_HEADER_SIZE + Self::POD_SLICE_HEADER_SIZE + (EXTRA_ACCOUNT_META_SIZE * num_items)
    }

    /// Initialize the extra-account-metas PDA data buffer.
    ///
    /// Writes the full TLV envelope: 8-byte type discriminator,
    /// 4-byte value length, 4-byte count, and each
    /// [`ExtraAccountMeta`] entry.
    ///
    /// `type_discriminator` identifies which instruction these metas
    /// are for — typically
    /// [`EXECUTE_DISCRIMINATOR`](crate::EXECUTE_DISCRIMINATOR).
    ///
    /// `buf` must be at least [`Self::size_of(metas.len())`] bytes.
    pub fn init(
        buf: &mut [u8],
        type_discriminator: &[u8; 8],
        metas: &[ExtraAccountMeta],
    ) -> Result<(), ProgramError> {
        let expected = Self::size_of(metas.len());
        if buf.len() < expected {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let value_len = Self::POD_SLICE_HEADER_SIZE + EXTRA_ACCOUNT_META_SIZE * metas.len();

        // TLV header: 8-byte discriminator + 4-byte value length.
        buf[0..8].copy_from_slice(type_discriminator);
        buf[8..12].copy_from_slice(&(value_len as u32).to_le_bytes());

        // PodSlice count.
        buf[12..16].copy_from_slice(&(metas.len() as u32).to_le_bytes());

        // Entries.
        let mut offset = 16;
        for meta in metas {
            buf[offset] = meta.discriminator;
            buf[offset + 1..offset + 33].copy_from_slice(&meta.address_config);
            buf[offset + 33] = meta.is_signer;
            buf[offset + 34] = meta.is_writable;
            offset += EXTRA_ACCOUNT_META_SIZE;
        }

        Ok(())
    }

    /// Read the count of extra account metas from an initialized buffer.
    ///
    /// Validates that the TLV type discriminator matches
    /// `type_discriminator`.
    ///
    /// Returns `None` if the buffer is too short or the discriminator
    /// does not match.
    #[inline]
    pub fn count(buf: &[u8], type_discriminator: &[u8; 8]) -> Option<u32> {
        if buf.len() < 16 {
            return None;
        }
        if buf[0..8] != *type_discriminator {
            return None;
        }
        Some(u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]))
    }

    /// Get a reference to the `i`-th [`ExtraAccountMeta`] in an
    /// initialized buffer.
    ///
    /// Returns `None` if `index` is out of bounds.
    #[inline]
    pub fn get<'a>(
        buf: &'a [u8],
        type_discriminator: &[u8; 8],
        index: usize,
    ) -> Option<&'a ExtraAccountMeta> {
        let count = Self::count(buf, type_discriminator)? as usize;
        if index >= count {
            return None;
        }
        let offset = 16 + index * EXTRA_ACCOUNT_META_SIZE;
        let end = offset + EXTRA_ACCOUNT_META_SIZE;
        if buf.len() < end {
            return None;
        }
        // SAFETY: ExtraAccountMeta is #[repr(C)] with alignment 1 (all
        // fields are u8 or [u8; N]), and the slice bounds are verified
        // above.
        Some(unsafe { &*(buf[offset..end].as_ptr() as *const ExtraAccountMeta) })
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use {super::*, crate::EXECUTE_DISCRIMINATOR, alloc::vec};

    #[test]
    fn seed_pack_literal() {
        let mut lit_bytes = [0u8; 32];
        lit_bytes[..5].copy_from_slice(b"hello");
        let seed = Seed::Literal {
            bytes: lit_bytes,
            length: 5,
        };
        assert_eq!(seed.tlv_size(), 7);
        let config = Seed::pack_into_address_config(&[seed]).unwrap();
        assert_eq!(config[0], 1); // discriminator
        assert_eq!(config[1], 5); // length
        assert_eq!(&config[2..7], b"hello");
    }

    #[test]
    fn seed_pack_account_key() {
        let seed = Seed::AccountKey { index: 2 };
        assert_eq!(seed.tlv_size(), 2);
        let config = Seed::pack_into_address_config(&[seed]).unwrap();
        assert_eq!(config[0], 3);
        assert_eq!(config[1], 2);
    }

    #[test]
    fn seed_pack_multiple() {
        let seeds = [
            Seed::AccountKey { index: 0 },
            Seed::AccountKey { index: 2 },
            Seed::InstructionData {
                index: 8,
                length: 8,
            },
        ];
        let config = Seed::pack_into_address_config(&seeds).unwrap();
        // AccountKey(0): [3, 0]
        assert_eq!(config[0], 3);
        assert_eq!(config[1], 0);
        // AccountKey(2): [3, 2]
        assert_eq!(config[2], 3);
        assert_eq!(config[3], 2);
        // InstructionData(8, 8): [2, 8, 8]
        assert_eq!(config[4], 2);
        assert_eq!(config[5], 8);
        assert_eq!(config[6], 8);
    }

    #[test]
    fn seed_pack_overflow() {
        // 32-byte literal fills the entire config — adding another seed
        // should fail.
        let mut lit_bytes = [0u8; 32];
        lit_bytes[..30].copy_from_slice(&[0xAA; 30]);
        let seeds = [
            Seed::Literal {
                bytes: lit_bytes,
                length: 30,
            },
            Seed::AccountKey { index: 0 },
        ];
        assert!(Seed::pack_into_address_config(&seeds).is_err());
    }

    #[test]
    fn extra_account_meta_fixed() {
        let pubkey = [42u8; 32];
        let meta = ExtraAccountMeta::new_with_pubkey(&pubkey, true, false);
        assert_eq!(meta.discriminator, 0);
        assert_eq!(meta.address_config, pubkey);
        assert_eq!(meta.is_signer, 1);
        assert_eq!(meta.is_writable, 0);
    }

    #[test]
    fn extra_account_meta_pda() {
        let seeds = [Seed::AccountKey { index: 0 }, Seed::AccountKey { index: 2 }];
        let meta = ExtraAccountMeta::new_with_seeds(&seeds, false, true).unwrap();
        assert_eq!(meta.discriminator, 1);
        assert_eq!(meta.is_signer, 0);
        assert_eq!(meta.is_writable, 1);
        assert_eq!(meta.address_config[0], 3); // AccountKey disc
        assert_eq!(meta.address_config[1], 0); // index 0
        assert_eq!(meta.address_config[2], 3); // AccountKey disc
        assert_eq!(meta.address_config[3], 2); // index 2
    }

    #[test]
    fn extra_account_meta_external_pda() {
        let seeds = [Seed::AccountKey { index: 1 }];
        let meta = ExtraAccountMeta::new_external_pda_with_seeds(5, &seeds, false, true).unwrap();
        assert_eq!(meta.discriminator, 133); // 128 + 5
    }

    #[test]
    fn list_size_of() {
        // 12 (TLV header) + 4 (count) + 35*2 = 86
        assert_eq!(ExtraAccountMetaList::size_of(2), 86);
        // 12 + 4 + 0 = 16
        assert_eq!(ExtraAccountMetaList::size_of(0), 16);
    }

    #[test]
    fn list_init_and_read() {
        let pubkey_a = [1u8; 32];
        let pubkey_b = [2u8; 32];
        let metas = [
            ExtraAccountMeta::new_with_pubkey(&pubkey_a, true, false),
            ExtraAccountMeta::new_with_pubkey(&pubkey_b, false, true),
        ];

        let size = ExtraAccountMetaList::size_of(metas.len());
        assert_eq!(size, 12 + 4 + 35 * 2); // 86

        let mut buf = vec![0u8; size];
        ExtraAccountMetaList::init(&mut buf, &EXECUTE_DISCRIMINATOR, &metas).unwrap();

        // Verify TLV header.
        assert_eq!(&buf[0..8], &EXECUTE_DISCRIMINATOR);
        let value_len = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        assert_eq!(value_len as usize, 4 + 35 * 2);

        assert_eq!(
            ExtraAccountMetaList::count(&buf, &EXECUTE_DISCRIMINATOR),
            Some(2)
        );

        let entry0 = ExtraAccountMetaList::get(&buf, &EXECUTE_DISCRIMINATOR, 0).unwrap();
        assert_eq!(entry0.discriminator, 0);
        assert_eq!(entry0.address_config, pubkey_a);
        assert_eq!(entry0.is_signer, 1);
        assert_eq!(entry0.is_writable, 0);

        let entry1 = ExtraAccountMetaList::get(&buf, &EXECUTE_DISCRIMINATOR, 1).unwrap();
        assert_eq!(entry1.address_config, pubkey_b);
        assert_eq!(entry1.is_signer, 0);
        assert_eq!(entry1.is_writable, 1);

        assert!(ExtraAccountMetaList::get(&buf, &EXECUTE_DISCRIMINATOR, 2).is_none());
    }

    #[test]
    fn list_wrong_discriminator() {
        let metas = [ExtraAccountMeta::new_with_pubkey(&[1u8; 32], false, false)];
        let size = ExtraAccountMetaList::size_of(metas.len());
        let mut buf = vec![0u8; size];
        ExtraAccountMetaList::init(&mut buf, &EXECUTE_DISCRIMINATOR, &metas).unwrap();

        // Reading with a different discriminator should return None.
        let wrong_disc = [0u8; 8];
        assert!(ExtraAccountMetaList::count(&buf, &wrong_disc).is_none());
    }

    #[test]
    fn list_buffer_too_small() {
        let metas = [ExtraAccountMeta::new_with_pubkey(&[0u8; 32], false, false)];
        let mut buf = [0u8; 10]; // way too small
        assert!(ExtraAccountMetaList::init(&mut buf, &EXECUTE_DISCRIMINATOR, &metas).is_err());
    }
}
