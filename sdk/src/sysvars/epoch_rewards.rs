use crate::{impl_sysvar_get, sysvars::Sysvar, Address, Hash};

/// The ID of the epoch rewards sysvar.
pub const EPOCH_REWARDS_ID: Address = Address::new_from_array([
    6, 167, 213, 23, 24, 220, 63, 238, 2, 165, 88, 191, 131, 206, 102, 225, 68, 66, 42, 28, 52,
    149, 11, 39, 193, 134, 155, 90, 156, 0, 0, 0,
]);

/// Epoch rewards sysvar
#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Debug, Default, Clone)]
pub struct EpochRewards {
    /// The starting block height of the rewards distribution in the current
    /// epoch
    pub distribution_starting_block_height: u64,

    /// Number of partitions in the rewards distribution in the current epoch,
    /// used to generate an EpochRewardsHasher
    pub num_partitions: u64,

    /// The blockhash of the parent block of the first block in the epoch, used
    /// to seed an EpochRewardsHasher
    pub parent_blockhash: Hash,

    /// The total rewards points calculated for the current epoch, where points
    /// equals the sum of (delegated stake * credits observed) for all
    /// delegations
    pub total_points: u128,

    /// The total rewards calculated for the current epoch. This may be greater
    /// than the total `distributed_rewards` at the end of the rewards period,
    /// due to rounding and inability to deliver rewards smaller than 1 lamport.
    pub total_rewards: u64,

    /// The rewards currently distributed for the current epoch, in lamports
    pub distributed_rewards: u64,

    /// Whether the rewards period (including calculation and distribution) is
    /// active
    pub active: bool,
}

impl Sysvar for EpochRewards {
    // SAFETY: upstream invariant: the sysvar data is created exclusively
    // by the Solana runtime and serializes bool as 0x00 or 0x01, so the final
    // `bool` field of `EpochRewards` can be re-aligned with padding and read
    // directly without validation.
    impl_sysvar_get!(EPOCH_REWARDS_ID, 15);
}
