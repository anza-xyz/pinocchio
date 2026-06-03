use {crate::state::ExtensionType, core::mem::MaybeUninit};

mod amount_to_ui_amount;
mod approve;
mod approve_checked;
mod burn;
mod burn_checked;
mod close_account;
mod create_native_mint;
mod extensions;
mod freeze_account;
mod get_account_data_size;
mod initialize_account;
mod initialize_account_2;
mod initialize_account_3;
mod initialize_immutable_owner;
mod initialize_mint;
mod initialize_mint_2;
mod initialize_multisig;
mod initialize_multisig_2;
mod initialize_non_transferable_mint;
mod mint_to;
mod mint_to_checked;
mod reallocate;
mod revoke;
mod set_authority;
mod sync_native;
mod thaw_account;
mod transfer;
mod transfer_checked;
mod ui_amount_to_amount;
mod unwrap_lamports;
mod withdraw_excess_lamports;

pub use {
    amount_to_ui_amount::*, approve::*, approve_checked::*, burn::*, burn_checked::*,
    close_account::*, create_native_mint::*, extensions::*, freeze_account::*,
    get_account_data_size::*, initialize_account::*, initialize_account_2::*,
    initialize_account_3::*, initialize_immutable_owner::*, initialize_mint::*,
    initialize_mint_2::*, initialize_multisig::*, initialize_multisig_2::*,
    initialize_non_transferable_mint::*, mint_to::*, mint_to_checked::*, reallocate::*, revoke::*,
    set_authority::*, sync_native::*, thaw_account::*, transfer::*, transfer_checked::*,
    ui_amount_to_amount::*, unwrap_lamports::*, withdraw_excess_lamports::*,
};

/// The maximum number of available extensions.
const MAX_EXTENSION_COUNT: usize = 28;
const EXTENSION_TYPES_INSTRUCTION_DATA_LEN: usize = 1 + MAX_EXTENSION_COUNT * 2;

#[inline(always)]
fn write_extension_types_instruction_data(
    instruction_data: &mut [MaybeUninit<u8>; EXTENSION_TYPES_INSTRUCTION_DATA_LEN],
    discriminator: u8,
    extensions: &[ExtensionType],
) {
    debug_assert!(extensions.len() <= MAX_EXTENSION_COUNT);

    instruction_data[0].write(discriminator);

    for (i, extension) in extensions.iter().enumerate() {
        let offset = 1 + i * 2;
        let extension_type = (*extension as u16).to_le_bytes();
        // SAFETY: `offset` and `offset + 1` are within bounds of
        // `instruction_data` because the buffer is exactly
        // `1 + MAX_EXTENSION_COUNT * 2` bytes and callers reject
        // `extensions.len() > MAX_EXTENSION_COUNT`.
        unsafe {
            instruction_data
                .get_unchecked_mut(offset)
                .write(extension_type[0]);
            instruction_data
                .get_unchecked_mut(offset + 1)
                .write(extension_type[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{state::ExtensionType, UNINIT_BYTE},
    };

    #[test]
    fn write_extension_types_instruction_data_with_empty_extensions() {
        let mut instruction_data = [UNINIT_BYTE; EXTENSION_TYPES_INSTRUCTION_DATA_LEN];

        write_extension_types_instruction_data(&mut instruction_data, 21, &[]);

        // SAFETY: the helper initialized the single discriminator byte.
        let bytes =
            unsafe { core::slice::from_raw_parts(instruction_data.as_ptr().cast::<u8>(), 1) };

        assert_eq!(bytes, &[21]);
    }

    #[test]
    fn write_extension_types_instruction_data_at_max_extension_count() {
        const EXTENSIONS: [ExtensionType; MAX_EXTENSION_COUNT] = [
            ExtensionType::TransferFeeConfig,
            ExtensionType::TransferFeeAmount,
            ExtensionType::MintCloseAuthority,
            ExtensionType::ConfidentialTransferMint,
            ExtensionType::ConfidentialTransferAccount,
            ExtensionType::DefaultAccountState,
            ExtensionType::ImmutableOwner,
            ExtensionType::MemoTransfer,
            ExtensionType::NonTransferable,
            ExtensionType::InterestBearingConfig,
            ExtensionType::CpiGuard,
            ExtensionType::PermanentDelegate,
            ExtensionType::NonTransferableAccount,
            ExtensionType::TransferHook,
            ExtensionType::TransferHookAccount,
            ExtensionType::ConfidentialTransferFeeConfig,
            ExtensionType::ConfidentialTransferFeeAmount,
            ExtensionType::MetadataPointer,
            ExtensionType::TokenMetadata,
            ExtensionType::GroupPointer,
            ExtensionType::TokenGroup,
            ExtensionType::GroupMemberPointer,
            ExtensionType::TokenGroupMember,
            ExtensionType::ConfidentialMintBurn,
            ExtensionType::ScaledUiAmount,
            ExtensionType::Pausable,
            ExtensionType::PausableAccount,
            ExtensionType::PermissionedBurn,
        ];

        let mut instruction_data = [UNINIT_BYTE; EXTENSION_TYPES_INSTRUCTION_DATA_LEN];

        write_extension_types_instruction_data(&mut instruction_data, 21, &EXTENSIONS);

        // SAFETY: the helper initialized all `1 + MAX_EXTENSION_COUNT * 2` bytes.
        let bytes = unsafe {
            core::slice::from_raw_parts(
                instruction_data.as_ptr().cast::<u8>(),
                instruction_data.len(),
            )
        };

        #[rustfmt::skip]
        let expected: [u8; EXTENSION_TYPES_INSTRUCTION_DATA_LEN] = [
            21,
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0, 9, 0, 10, 0,
            11, 0, 12, 0, 13, 0, 14, 0, 15, 0, 16, 0, 17, 0, 18, 0, 19, 0, 20, 0,
            21, 0, 22, 0, 23, 0, 24, 0, 25, 0, 26, 0, 27, 0, 28, 0,
        ];

        assert_eq!(bytes, &expected);
    }
}
