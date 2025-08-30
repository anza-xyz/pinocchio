use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{state::Fee, write_bytes, UNINIT_BYTE};

///   Initializes a new `StakePool`.
///
/// ### Accounts:
///   0. `[WRITE]` New `StakePool` to create.
///   1. `[SIGNER]` Manager
///   2. `[]` Staker
///   3. `[]` Stake pool withdraw authority
///   4. `[WRITE]` Uninitialized validator stake list storage account
///   5. `[]` Reserve stake account must be initialized, have zero balance,
///      and staker / withdrawer authority set to pool withdraw authority.
///   6. `[]` Pool token mint. Must have zero supply, owned by withdraw
///      authority.
///   7. `[]` Pool account to deposit the generated fee for manager.
///   8. `[]` Token program id
///   9. `[]` (Optional) Deposit authority that must sign all deposits.
///      Defaults to the program address generated using
///      `find_deposit_authority_program_address`, making deposits
///      permissionless.
pub struct Initialize<'a> {
    /// Accounts
    /// Stake Pool Account.
    pub stake_pool: &'a AccountInfo,
    /// Manager Account.
    pub manager: &'a AccountInfo,
    /// Staker Account.
    pub staker: &'a AccountInfo,
    /// Withdraw Authority Account.
    pub stake_pool_withdraw_authority: &'a AccountInfo,
    /// Validator list Account.
    pub validator_list: &'a AccountInfo,
    /// Reserve stake Account.
    pub reserve_stake: &'a AccountInfo,
    /// Pool mint Account.
    pub pool_mint: &'a AccountInfo,
    /// Manager Pool Account.
    pub manager_pool_account: &'a AccountInfo,
    /// Token program.
    pub token_program: &'a AccountInfo,
    /// Deposit Authority (Optional)
    pub deposit_authority: Option<&'a AccountInfo>,

    /// input
    /// Fee.
    pub fee: Fee,
    /// Withdrawal Fee.
    pub withdrawal_fee: Fee,
    /// Deposit Fee.
    pub deposit_fee: Fee,
    /// Referral Fee.
    pub referral_fee: u8,
    /// Max validators.
    pub max_validators: u32,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: &[AccountMeta] = if let Some(deposit_info) = self.deposit_authority {
            &[
                AccountMeta::writable(self.stake_pool.key()),
                AccountMeta::readonly_signer(self.manager.key()),
                AccountMeta::readonly(self.staker.key()),
                AccountMeta::readonly(self.stake_pool_withdraw_authority.key()),
                AccountMeta::writable(self.validator_list.key()),
                AccountMeta::readonly(self.reserve_stake.key()),
                AccountMeta::readonly(self.pool_mint.key()),
                AccountMeta::readonly(self.manager_pool_account.key()),
                AccountMeta::readonly(self.token_program.key()),
                AccountMeta::readonly(deposit_info.key()),
            ]
        } else {
            &[
                AccountMeta::writable(self.stake_pool.key()),
                AccountMeta::readonly_signer(self.manager.key()),
                AccountMeta::readonly(self.staker.key()),
                AccountMeta::readonly(self.stake_pool_withdraw_authority.key()),
                AccountMeta::writable(self.validator_list.key()),
                AccountMeta::readonly(self.reserve_stake.key()),
                AccountMeta::readonly(self.pool_mint.key()),
                AccountMeta::readonly(self.manager_pool_account.key()),
                AccountMeta::readonly(self.token_program.key()),
            ]
        };

        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..17]: fee (16 bytes, Fee(u64, u64))
        // -  [17..33]: withdrawal_fee (16 bytes, Fee(u64, u64))
        // -  [33..49]: deposit_fee (16 bytes, Fee(u64, u64))
        // -  [49]: referral_fee (1 byte, u8)
        // -  [50..54] max_validators (4 bytes, u32)
        let mut instruction_data = [UNINIT_BYTE; 54];

        // Set discriminator as u8 at offet [0]
        write_bytes(&mut instruction_data, &[0]);
        // Set fee.denominator as u64 at offset [1..9]
        write_bytes(
            &mut instruction_data[1..9],
            &self.fee.denominator.to_le_bytes(),
        );
        // Set fee.numerator as u64 at offset [9..17]
        write_bytes(
            &mut instruction_data[9..17],
            &self.fee.numerator.to_le_bytes(),
        );
        // Set withdrawal_fee.denominator as u64 at offset [17..25]
        write_bytes(
            &mut instruction_data[17..25],
            &self.withdrawal_fee.denominator.to_le_bytes(),
        );
        // Set withdrawal_fee.numerator as u64 at offset [25..33]
        write_bytes(
            &mut instruction_data[25..33],
            &self.withdrawal_fee.numerator.to_le_bytes(),
        );
        // Set deposit_fee.denominator as u64 at offset [33..41]
        write_bytes(
            &mut instruction_data[33..41],
            &self.deposit_fee.denominator.to_le_bytes(),
        );
        // Set deposit_fee.numerator as u64 at offset [41..49]
        write_bytes(
            &mut instruction_data[41..49],
            &self.deposit_fee.numerator.to_le_bytes(),
        );
        // Set referral_fee as u8 at offset [49]
        write_bytes(&mut instruction_data[49..50], &[self.referral_fee]);
        // Set max_validators as u32 at offet [50..54]
        write_bytes(
            &mut instruction_data[50..54],
            &self.max_validators.to_le_bytes(),
        );

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 56) },
        };

        if let Some(deposit_info) = self.deposit_authority {
            invoke_signed(
                &instruction,
                &[
                    self.stake_pool,
                    self.manager,
                    self.staker,
                    self.stake_pool_withdraw_authority,
                    self.validator_list,
                    self.reserve_stake,
                    self.pool_mint,
                    self.manager_pool_account,
                    self.token_program,
                    deposit_info,
                ],
                signers,
            )
        } else {
            invoke_signed(
                &instruction,
                &[
                    self.stake_pool,
                    self.manager,
                    self.staker,
                    self.stake_pool_withdraw_authority,
                    self.validator_list,
                    self.reserve_stake,
                    self.pool_mint,
                    self.manager_pool_account,
                    self.token_program,
                ],
                signers,
            )
        }
    }
}
