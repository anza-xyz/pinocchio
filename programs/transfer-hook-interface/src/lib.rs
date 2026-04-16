#![no_std]

pub mod instruction;
pub mod state;

use solana_address::Address;

/// Namespace for all programs implementing transfer-hook.
pub const NAMESPACE: &str = "spl-transfer-hook-interface";

/// Seed for the extra account metas PDA.
pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

/// Discriminator for the `Execute` instruction.
///
/// Computed as `SHA256("spl-transfer-hook-interface:execute")[..8]`.
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

/// Discriminator for the `InitializeExtraAccountMetaList` instruction.
///
/// Computed as
/// `SHA256("spl-transfer-hook-interface:initialize-extra-account-metas")[..8]`.
pub const INITIALIZE_EXTRA_ACCOUNT_META_LIST_DISCRIMINATOR: [u8; 8] =
    [43, 34, 13, 49, 167, 88, 235, 235];

/// Discriminator for the `UpdateExtraAccountMetaList` instruction.
///
/// Computed as
/// `SHA256("spl-transfer-hook-interface:update-extra-account-metas")[..8]`.
pub const UPDATE_EXTRA_ACCOUNT_META_LIST_DISCRIMINATOR: [u8; 8] =
    [157, 105, 42, 146, 102, 85, 241, 174];

/// Collect the seeds used to derive the extra account metas PDA.
#[inline(always)]
pub fn collect_extra_account_metas_seeds(mint: &Address) -> [&[u8]; 2] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

/// Collect the signer seeds for the extra account metas PDA.
#[inline(always)]
pub fn collect_extra_account_metas_signer_seeds<'a>(
    mint: &'a Address,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref(), bump_seed]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_discriminator() {
        let hash = <sha2::Sha256 as sha2::Digest>::digest(b"spl-transfer-hook-interface:execute");
        assert_eq!(EXECUTE_DISCRIMINATOR, hash[..8]);
    }

    #[test]
    fn initialize_discriminator() {
        let hash = <sha2::Sha256 as sha2::Digest>::digest(
            b"spl-transfer-hook-interface:initialize-extra-account-metas",
        );
        assert_eq!(INITIALIZE_EXTRA_ACCOUNT_META_LIST_DISCRIMINATOR, hash[..8]);
    }

    #[test]
    fn update_discriminator() {
        let hash = <sha2::Sha256 as sha2::Digest>::digest(
            b"spl-transfer-hook-interface:update-extra-account-metas",
        );
        assert_eq!(UPDATE_EXTRA_ACCOUNT_META_LIST_DISCRIMINATOR, hash[..8]);
    }
}
