use {
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Initializes a new `TokenGroupMember` in the provided account and adds
/// it to the group, incrementing the group size.
///
/// Token-2022 only supports storing the member in the member mint itself,
/// so the member account must be the member mint. The member mint must
/// have the `GroupMemberPointer` extension pointing to itself and enough
/// lamports to cover the rent of the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The member account.
///   1. `[]` The member mint.
///   2. `[signer]` The member mint authority.
///   3. `[writable]` The group account.
///   4. `[signer]` The group update authority.
pub struct InitializeMember<'a, 'b> {
    /// The member account.
    pub member: &'a AccountView,

    /// The member mint.
    pub member_mint: &'a AccountView,

    /// The member mint authority.
    pub member_mint_authority: &'a AccountView,

    /// The group account.
    pub group: &'a AccountView,

    /// The group update authority.
    pub group_update_authority: &'a AccountView,

    /// The token program.
    pub token_program: &'b Address,
}

impl InitializeMember<'_, '_> {
    /// Hash of `spl_token_group_interface:initialize_member`.
    pub const DISCRIMINATOR: [u8; 8] = [152, 32, 222, 176, 223, 237, 116, 134];

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        invoke_signed(
            &InstructionView {
                program_id: self.token_program,
                accounts: &[
                    InstructionAccount::writable(self.member.address()),
                    InstructionAccount::readonly(self.member_mint.address()),
                    InstructionAccount::readonly_signer(self.member_mint_authority.address()),
                    InstructionAccount::writable(self.group.address()),
                    InstructionAccount::readonly_signer(self.group_update_authority.address()),
                ],
                data: &Self::DISCRIMINATOR,
            },
            &[
                self.member,
                self.member_mint,
                self.member_mint_authority,
                self.group,
                self.group_update_authority,
            ],
            signers,
        )
    }
}
