pub mod initialize_group;
pub mod initialize_member;
pub mod update_group_authority;
pub mod update_group_max_size;

use crate::Token2022Program;

/// Initializes a new `TokenGroup` in the provided account.
///
/// Token-2022 only supports storing the group in the mint itself, so the
/// group account must be the mint. The mint must have the `GroupPointer`
/// extension pointing to itself and enough lamports to cover the rent of
/// the additional space.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[]` The mint.
///   2. `[signer]` The mint authority.
pub type InitializeGroup<'account, 'address> =
    initialize_group::InitializeGroup<'account, 'address, Token2022Program>;

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
pub type InitializeMember<'account> =
    initialize_member::InitializeMember<'account, Token2022Program>;

/// Updates the update authority of a `TokenGroup`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[signer]` The current group update authority.
pub type UpdateGroupAuthority<'account, 'address> =
    update_group_authority::UpdateGroupAuthority<'account, 'address, Token2022Program>;

/// Updates the maximum number of members of a `TokenGroup`.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The group account.
///   1. `[signer]` The group update authority.
pub type UpdateGroupMaxSize<'account> =
    update_group_max_size::UpdateGroupMaxSize<'account, Token2022Program>;
