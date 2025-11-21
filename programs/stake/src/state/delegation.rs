use pinocchio::pubkey::Pubkey;

/// Delegation data.
#[repr(C)]
pub struct Delegation {
    /// The pubkey of the vote account the stake is delegated to.
    voter_pubkey: Pubkey,
    /// The amount of stake delegated.
    stake: [u8; 8],
    /// The epoch at which the stake was activated.
    activation_epoch: [u8; 8],
    /// The epoch at which the stake was deactivated.
    deactivation_epoch: [u8; 8],
    /// The warmup/cooldown rate.
    warmup_cooldown_rate: [u8; 8],
}

impl Delegation {
    /// The length of the `Delegation` data.
    pub const LEN: usize = core::mem::size_of::<Delegation>();

    /// Returns a reference to the voter pubkey.
    #[inline(always)]
    pub fn voter_pubkey(&self) -> &Pubkey {
        &self.voter_pubkey
    }

    /// Returns the amount of stake delegated.
    #[inline(always)]
    pub fn stake(&self) -> u64 {
        u64::from_le_bytes(self.stake)
    }

    /// Returns the activation epoch.
    #[inline(always)]
    pub fn activation_epoch(&self) -> u64 {
        u64::from_le_bytes(self.activation_epoch)
    }

    /// Returns the deactivation epoch.
    #[inline(always)]
    pub fn deactivation_epoch(&self) -> u64 {
        u64::from_le_bytes(self.deactivation_epoch)
    }

    /// Returns the warmup/cooldown rate.
    #[inline(always)]
    pub fn warmup_cooldown_rate(&self) -> f64 {
        f64::from_le_bytes(self.warmup_cooldown_rate)
    }
}