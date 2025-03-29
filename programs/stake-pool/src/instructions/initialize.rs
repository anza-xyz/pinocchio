use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{state::Fee, write_bytes, UNINIT_BYTE};

pub struct Initialize<'a> {
    // accounts
    pub stake_pool: &'a AccountInfo,
    pub manager: &'a AccountInfo,
    pub staker: &'a AccountInfo,
    pub stake_pool_withdraw_authority: &'a AccountInfo,
    pub validator_list: &'a AccountInfo,
    pub reserve_stake: &'a AccountInfo,
    pub pool_mint: &'a AccountInfo,
    pub manager_pool_account: &'a AccountInfo,
    pub token_program: &'a AccountInfo,

    /// input
    pub amount: u64,
    pub deposit_authority: Option<&'a Pubkey>,
    pub fee: Fee,
    pub withdrawal_fee: Fee,
    pub deposit_fee: Fee,
    pub referral_fee: u8,
    pub max_validators: u32,
}

impl Initialize<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 9] = [
            AccountMeta::writable(self.stake_pool.key()),
            AccountMeta::readonly_signer(self.manager.key()),
            AccountMeta::readonly(self.staker.key()),
            AccountMeta::readonly(self.stake_pool_withdraw_authority.key()),
            AccountMeta::writable(self.validator_list.key()),
            AccountMeta::readonly(self.reserve_stake.key()),
            AccountMeta::readonly(self.pool_mint.key()),
            AccountMeta::readonly(self.manager_pool_account.key()),
            AccountMeta::readonly(self.token_program.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]: deposit_authority presence flag (1 byte, u8)
        // -  [10..42]: deposit_authority  (optional, 32 bytes, Pubkey)
        // -  [42..58]: fee (16 bytes, Fee(u64, u64))
        // -  [58..74]: withdrawal_fee (16 bytes, Fee(u64, u64))
        // -  [74..90]: deposit_fee (16 bytes, Fee(u64, u64))
        // -  [90]: referral_fee (1 byte, u8)
        // -  [91..95]: max_validators (4 bytes, u32)
        let mut instruction_data = [UNINIT_BYTE; 95];

        unimplemented!()
    }
}
