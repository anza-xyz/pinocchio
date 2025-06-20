#![no_std]

pub use five8_const::decode_32_const;
pub use pinocchio;

/// Convenience macro to define a static `Pubkey` value.
/// 
/// This macro validates the pubkey at compile time, ensuring it's a valid
/// base58-encoded 32-byte public key.
/// 
/// # Examples
/// 
/// ```rust
/// use pinocchio_pubkey::pubkey;
/// 
/// const MY_PUBKEY: pinocchio::pubkey::Pubkey = pubkey!("11111111111111111111111111111111");
/// ```
#[macro_export]
macro_rules! pubkey {
    ( $id:literal ) => {
        $crate::from_str($id)
    };
}

/// Convenience macro to define a static `Pubkey` value representing the program ID.
///
/// This macro also defines helper functions to check whether a given pubkey is
/// equal to the program ID, providing a complete program identity management solution.
/// 
/// # Examples
/// 
/// ```rust
/// use pinocchio_pubkey::declare_id;
/// 
/// declare_id!("11111111111111111111111111111111");
/// 
/// // Now you can use:
/// // - ID: the program ID constant
/// // - check_id(&pubkey): returns true if pubkey matches the program ID
/// // - id(): returns the program ID
/// ```
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

/// Create a `Pubkey` from a `&str` at compile time.
/// 
/// This function uses compile-time base58 decoding for maximum efficiency.
/// The input string is validated at compile time, ensuring no runtime errors.
/// 
/// # Panics
/// 
/// This function will cause a compile-time error if the input string is not
/// a valid base58-encoded 32-byte public key.
/// 
/// # Examples
/// 
/// ```rust
/// const SYSTEM_PROGRAM: pinocchio::pubkey::Pubkey = 
///     pinocchio_pubkey::from_str("11111111111111111111111111111111");
/// ```
#[inline(always)]
pub const fn from_str(value: &str) -> pinocchio::pubkey::Pubkey {
    decode_32_const(value)
}

/// Type alias for the Pubkey type for convenience.
pub type Pubkey = pinocchio::pubkey::Pubkey;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_pubkey_creation() {
        const SYSTEM_PROGRAM: Pubkey = from_str("11111111111111111111111111111111");
        // This test just ensures compilation works
        assert_eq!(SYSTEM_PROGRAM.len(), 32);
    }

    #[test]
    fn test_declare_id_macro() {
        declare_id!("11111111111111111111111111111111");
        
        let test_pubkey = from_str("11111111111111111111111111111111");
        assert!(check_id(&test_pubkey));
        assert_eq!(id(), test_pubkey);
        
        // Use a different valid base58 pubkey for testing
        let other_pubkey = from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        assert!(!check_id(&other_pubkey));
    }
}