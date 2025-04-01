#![no_std]

pub use five8_const::decode_32_const;
pub use solana_address::Address;

/// Convenience macro to define a static [`Address`] value.
#[macro_export]
macro_rules! address {
    ( $id:literal ) => {
        $crate::from_str($id)
    };
}

/// Convenience macro to define a static [`Address`] value representing the program ID.
///
/// This macro also defines a helper function to check whether a given address is
/// equal to the program ID.
#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        #[doc = "The const program ID."]
        pub const ID: $crate::Address = $crate::from_str($id);

        #[doc = "Returns `true` if given address is the program ID."]
        #[inline]
        pub fn check_id(id: &$crate::Address) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> $crate::Address {
            ID
        }
    };
}

/// Create a [`Address`] from a `&str`.
#[inline(always)]
pub const fn from_str(value: &str) -> Address {
    decode_32_const(value)
}
