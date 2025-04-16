use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{self, AccountMeta, Signer},
    pubkey::Pubkey,
    sysvars::clock::UnixTimestamp,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScaledUiAmountConfig {
    /// Authority that can set the scaling amount and authority
    pub authority: Pubkey,
    /// Amount to multiply raw amounts by, outside of the decimal
    pub multiplier: [u8; 8],
    /// Unix timestamp at which `new_multiplier` comes into effective
    pub new_multiplier_effective_timestamp: UnixTimestamp,
    /// Next multiplier, once `new_multiplier_effective_timestamp` is reached
    pub new_multiplier: [u8; 8],
}

impl super::Extension for ScaledUiAmountConfig {
    const TYPE: super::ExtensionType = super::ExtensionType::ScaledUiAmount;
    const LEN: usize = Self::LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl ScaledUiAmountConfig {
    /// The length of the `ScaledUiAmountConfig` account data.
    pub const LEN: usize = core::mem::size_of::<ScaledUiAmountConfig>();

    /// Return a `ScaledUiAmountConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<&ScaledUiAmountConfig, pinocchio::program_error::ProgramError> {
        super::get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(pinocchio::program_error::ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct Initialize<'a> {
    /// The mint to initialize
    pub mint: &'a AccountInfo,
    /// The public key for the account that can update the multiplier
    pub authority: Option<Pubkey>,
    /// The initial multiplier
    pub multiplier: f64,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, seeds: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction Layout
        // - [0] : instruction discriminator
        // - [1] : extension instruction discriminator
        // - [2..34] : authority
        // - [34..42] : multiplier

        let mut instruction_data = [UNINIT_BYTE; 42];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[43]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[0]);
        // Set authority as Pubkey at offset [2..34]
        if let Some(authority) = self.authority {
            write_bytes(&mut instruction_data[2..34], authority.as_ref());
        } else {
            write_bytes(&mut instruction_data[2..34], &Pubkey::default());
        }
        // Set multiplier as f64 at offset [34..42]
        write_bytes(
            &mut instruction_data[34..42],
            &self.multiplier.to_le_bytes(),
        );
        let instruction = instruction::Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 42) },
        };

        invoke_signed(&instruction, &[self.mint], seeds)?;

        Ok(())
    }
}

pub struct UpdateMultiplier<'a> {
    /// The mint to update multiplier
    pub mint: &'a AccountInfo,
    /// The multiplier authority
    pub authority: &'a AccountInfo,
    /// The new multiplier
    pub multiplier: [u8; 8],
    /// Timestamp at which the new multiplier will take effect
    pub effective_timestamp: UnixTimestamp,
}

impl UpdateMultiplier<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, seeds: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction Layout
        // - [0] : instruction discriminator
        // - [1] : extension instruction discriminator
        // - [2..10] : multiplier
        // - [10..18] : effective timestamp

        let mut instruction_data = [UNINIT_BYTE; 18];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data[0..1], &[43]);
        // Set extension discriminator as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[1]);
        // Set multiplier as f64 at offset [2..10]
        write_bytes(&mut instruction_data[2..10], &self.multiplier);
        // Set effective timestamp as u64 at offset [10..18]
        write_bytes(
            &mut instruction_data[10..18],
            &self.effective_timestamp.to_le_bytes(),
        );

        let instruction = instruction::Instruction {
            program_id: &crate::TOKEN_2022_PROGRAM_ID,
            accounts: &account_metas,
            data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as _, 18) },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], seeds)?;

        Ok(())
    }
}
