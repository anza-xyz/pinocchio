use {
    crate::{
        instructions::extensions::{
            group_pointer::state::{
                offset_group_pointer_update as OFFSET, InstructionDiscriminatorGroupPointer,
            },
            ExtensionDiscriminator,
        },
        instructions::MAX_MULTISIG_SIGNERS,
    },
    core::{mem::MaybeUninit, slice},
    pinocchio::{
        account_info::AccountInfo,
        cpi::invoke_signed_with_bounds,
        instruction::{AccountMeta, Instruction, Signer},
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult,
    },
};

/// Update the group pointer address. Only supported for mints that
/// include the `GroupPointer` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The group pointer authority.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's group pointer authority.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct Update<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// The group pointer authority.
    pub authority: &'a AccountInfo,
    /// The new account address that holds the group
    pub group_address: Option<&'b Pubkey>,
    /// The Signer accounts if `authority` is a multisig
    pub signers: &'a [AccountInfo],
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl Update<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            authority,
            signers: account_signers,
            token_program,
            ..
        } = self;

        if account_signers.len() > MAX_MULTISIG_SIGNERS {
            Err(ProgramError::InvalidArgument)?;
        }

        let num_accounts = 2 + account_signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(mint.key()));
            // - Index 1 is always present
            if account_signers.is_empty() {
                acc_metas
                    .get_unchecked_mut(1)
                    .write(AccountMeta::readonly_signer(authority.key()));
            } else {
                acc_metas
                    .get_unchecked_mut(1)
                    .write(AccountMeta::readonly(authority.key()));
            }
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(account_signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        let mut buffer = [0u8; OFFSET::END as usize];
        let data = update_instruction_data(&mut buffer, self.group_address);

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(mint);
            // - Index 1 is always present
            acc_infos.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[2..].iter_mut().zip(account_signers.iter()) {
            account_info.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}

#[inline(always)]
fn update_instruction_data<'a>(buffer: &'a mut [u8], group_address: Option<&Pubkey>) -> &'a [u8] {
    let offset = OFFSET::START as usize;

    // Set discriminators (GroupPointer + Update)
    buffer[..offset].copy_from_slice(&[
        ExtensionDiscriminator::GroupPointer as u8,
        InstructionDiscriminatorGroupPointer::Update as u8,
    ]);

    // Set group_address
    if let Some(x) = group_address {
        buffer[offset..offset + OFFSET::GROUP_ADDRESS_PUBKEY as usize].copy_from_slice(x);
    }

    buffer
}
