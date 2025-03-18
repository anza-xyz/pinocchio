pub use five8_const::decode_32_const;
pub use pinocchio;

/// Convenience macro to define a static `Pubkey` value.
#[macro_export]
macro_rules! pubkey {
    ( $id:literal ) => {
        $crate::from_str($id)
    };
}

/// Convenience macro to define a static `Pubkey` value representing the program ID.
///
/// This macro also defines a helper function to check whether a given pubkey is
/// equal to the program ID.
#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        #[doc = "The const program ID."]
        pub const ID: $crate::pinocchio::pubkey::Pubkey = $crate::from_str($id);

        #[doc = "Returns `true` if given pubkey is the program ID."]
        #[inline]
        pub fn check_id(id: &$crate::pinocchio::pubkey::Pubkey) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> $crate::pinocchio::pubkey::Pubkey {
            ID
        }
    };
}

/// Create a `Pubkey` from a `&str`.
#[inline(always)]
pub const fn from_str(value: &str) -> pinocchio::pubkey::Pubkey {
    decode_32_const(value)
}
