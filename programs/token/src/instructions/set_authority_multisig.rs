use {
    crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub use super::set_authority::AuthorityType;

/// Sets a new authority of a mint or account where the current authority
/// is a multisig account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint or account to change the authority of.
///   1. `[]` The current authority (multisig).
///   2. ..`2+N`. `[SIGNER]` The N signer accounts, where N is between 1 and 11.
pub struct SetAuthorityMultisig<'a, 'b>
where
    'a: 'b,
{
    /// Account (Mint or Token).
    pub account: &'a AccountView,
    /// Multisig authority account.
    pub multisig: &'a AccountView,
    /// Signer accounts.
    pub signers: &'b [&'a AccountView],
    /// The type of authority to update.
    pub authority_type: AuthorityType,
    /// The new authority.
    pub new_authority: Option<&'a Address>,
}

impl SetAuthorityMultisig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            account,
            multisig,
            signers: multisig_signers,
            authority_type,
            new_authority,
        } = self;

        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + multisig_signers.len();

        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts =
            [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(account.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(multisig.address()));
        }

        for (instruction_account, signer) in
            instruction_accounts[2..].iter_mut().zip(multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        // Instruction data layout:
        // - [0]: instruction discriminator (1 byte, u8)
        // - [1]: authority_type (1 byte, u8)
        // - [2]: new_authority presence flag (1 byte)
        // - [3..35] new_authority (optional, 32 bytes, Address)
        let mut instruction_data = [UNINIT_BYTE; 35];
        let mut length = instruction_data.len();

        write_bytes(&mut instruction_data, &[6]);
        write_bytes(&mut instruction_data[1..2], &[authority_type as u8]);

        if let Some(new_authority) = new_authority {
            write_bytes(&mut instruction_data[2..3], &[1]);
            write_bytes(&mut instruction_data[3..], new_authority.as_array());
        } else {
            write_bytes(&mut instruction_data[2..3], &[0]);
            length = 3;
        }

        let instruction = InstructionView {
            program_id: &crate::ID,
            accounts: unsafe {
                from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, length) },
        };

        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            acc_views.get_unchecked_mut(0).write(account);
            acc_views.get_unchecked_mut(1).write(multisig);
        }

        for (account_view, signer) in acc_views[2..].iter_mut().zip(multisig_signers.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { from_raw_parts(acc_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
