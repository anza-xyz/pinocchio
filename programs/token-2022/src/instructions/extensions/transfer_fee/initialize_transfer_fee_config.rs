use {
    crate::{instructions::ExtensionDiscriminator, write_bytes, UNINIT_BYTE},
    core::slice::from_raw_parts,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

/// Initialize the transfer fee on a new mint.
///
/// Fails if the mint has already been initialized, so must be called before
/// `InitializeMint`.
///
/// The mint must have exactly enough space allocated for the base mint (82
/// bytes), plus 83 bytes of padding, 1 byte reserved for the account type,
/// then space required for this extension, plus any others.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub struct InitializeTransferFeeConfig<'a, 'b> {
    /// The mint to initialize.
    pub mint: &'a AccountView,

    /// Address that may update the fees.
    pub transfer_fee_config_authority: Option<&'b Address>,

    /// Withdraw instructions must be signed by this address.
    pub withdraw_withheld_authority: Option<&'b Address>,

    /// Amount of transfer collected as fees, expressed as basis points of
    /// the transfer amount.
    pub transfer_fee_basis_points: u16,

    /// Maximum fee assessed on transfers.
    pub maximum_fee: u64,

    /// Token program.
    pub token_program: &'b Address,
}

impl InitializeTransferFeeConfig<'_, '_> {
    /// Instruction discriminator.
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Instruction data.

        let mut instruction_data = [UNINIT_BYTE; 78];
        // At the start, data length is 2 bytes for the instruction and extension
        // discriminators, plus 1 byte for the optional transfer fee authority.
        let mut data_len = 2 + 1;

        instruction_data[0].write(ExtensionDiscriminator::TransferFee as u8);

        instruction_data[1].write(Self::DISCRIMINATOR);

        if let Some(authority) = self.transfer_fee_config_authority {
            instruction_data[2].write(1);
            write_bytes(&mut instruction_data[3..35], authority.as_ref());
            // Add 32 bytes for the authority.
            data_len += size_of::<Address>();
        } else {
            instruction_data[2].write(0);
        }
        if let Some(authority) = self.withdraw_withheld_authority {
            // SAFETY: `instruction_data` is allocated to the maximum expected length.
            unsafe {
                instruction_data.get_unchecked_mut(data_len).write(1);
                write_bytes(
                    instruction_data.get_unchecked_mut(data_len + 1..),
                    authority.as_ref(),
                );
            }
            data_len += 1 + size_of::<Address>();
        } else {
            // SAFETY: `instruction_data` is allocated to the maximum expected length.
            unsafe { instruction_data.get_unchecked_mut(data_len).write(0) };
            data_len += 1;
        }

        // SAFETY: `instruction_data` is allocated to the maximum expected length.
        unsafe {
            write_bytes(
                instruction_data.get_unchecked_mut(data_len..),
                &self.transfer_fee_basis_points.to_le_bytes(),
            );
        }
        data_len += 2;

        // SAFETY: `instruction_data` is allocated to the maximum expected length.
        unsafe {
            write_bytes(
                instruction_data.get_unchecked_mut(data_len..),
                &self.maximum_fee.to_le_bytes(),
            );
        }
        data_len += 8;

        invoke(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[InstructionAccount::writable(self.mint.address())],
                // SAFETY: instruction data is initialized to `data_len` bytes.
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, data_len) },
            },
            &[self.mint],
        )
    }
}
