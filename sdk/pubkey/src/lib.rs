#![no_std]

pub use five8_const::decode_32_const;
pub use pinocchio;

/// Convenience macro to create a `Pubkey` from a base58 string literal.
/// 
/// # Examples
/// 
/// ```rust
/// use pinocchio_pubkey::pubkey;
/// 
/// const MY_PUBKEY: pinocchio::pubkey::Pubkey = pubkey!("11111111111111111111111111111111");
/// 
/// fn example() {
///     let pubkey = pubkey!("11111111111111111111111111111111");
/// }
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
/// equal to the program ID.
/// 
/// # Examples
/// 
/// ```rust
/// use pinocchio_pubkey::{declare_id, pubkey};
/// 
/// declare_id!("11111111111111111111111111111111");
/// 
/// // Now you can use:
/// // - ID: the program ID constant
/// // - check_id(&pubkey): returns true if pubkey matches the program ID
/// // - id(): returns the program ID
/// 
/// // Access the program ID constant
/// let program_id = ID;
/// 
/// // Get the program ID using the helper function
/// let same_id = id();
/// // program_id == same_id (both return the same pubkey)
/// 
/// // Check if a pubkey matches the program ID
/// let system_program = pubkey!("11111111111111111111111111111111");
/// let is_program_id = check_id(&system_program); // returns true
/// 
/// let other_pubkey = pubkey!("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
/// let is_other_program = check_id(&other_pubkey); // returns false
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

/// Create a `Pubkey` from a `&str`.
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
        
        // Test basic properties
        assert_eq!(SYSTEM_PROGRAM.len(), 32);
        
        // Test that it's the actual system program pubkey
        let expected_bytes = [0u8; 32]; // System program is all zeros
        assert_eq!(SYSTEM_PROGRAM.as_ref(), &expected_bytes);
        
        // Test that const creation works the same as runtime creation
        let runtime_pubkey = from_str("11111111111111111111111111111111");
        assert_eq!(SYSTEM_PROGRAM, runtime_pubkey);
        
        // Test with a different known pubkey
        const TOKEN_PROGRAM: Pubkey = from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert_eq!(TOKEN_PROGRAM.len(), 32);
        assert_ne!(TOKEN_PROGRAM, SYSTEM_PROGRAM);
        
        // Verify the bytes are different (not all zeros)
        assert_ne!(TOKEN_PROGRAM.as_ref(), &[0u8; 32]);
    }

    #[test]
    fn test_declare_id_macro() {
        declare_id!("11111111111111111111111111111111");
        
        // Test the ID constant
        assert_eq!(ID.len(), 32);
        assert_eq!(ID.as_ref(), &[0u8; 32]); // System program is all zeros
        
        // Test check_id function with matching pubkey
        let test_pubkey = from_str("11111111111111111111111111111111");
        assert!(check_id(&test_pubkey));
        assert_eq!(test_pubkey, ID);
        
        // Test id() function returns the same value as ID constant
        assert_eq!(id(), test_pubkey);
        assert_eq!(id(), ID);
        
        // Test check_id function with non-matching pubkey
        let other_pubkey = from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        assert!(!check_id(&other_pubkey));
        assert_ne!(other_pubkey, ID);
        assert_ne!(other_pubkey, id());
        
        // Test that the functions are consistent
        assert_eq!(check_id(&ID), true);
        assert_eq!(check_id(&id()), true);
        
        // Test with multiple different pubkeys to ensure robustness
        let token_program = from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        let associated_token_program = from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
        
        assert!(!check_id(&token_program));
        assert!(!check_id(&associated_token_program));
        assert_ne!(token_program, associated_token_program);
    }

    #[test]
    fn test_pubkey_macro() {
        // Test that the pubkey! macro works correctly
        let system_program = pubkey!("11111111111111111111111111111111");
        assert_eq!(system_program.len(), 32);
        assert_eq!(system_program.as_ref(), &[0u8; 32]);
        
        // Test with different pubkeys
        let token_program = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        let other_program = pubkey!("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        
        assert_eq!(token_program.len(), 32);
        assert_eq!(other_program.len(), 32);
        assert_ne!(token_program, other_program);
        assert_ne!(token_program, system_program);
        assert_ne!(other_program, system_program);
        
        // Test that macro produces same result as from_str
        assert_eq!(system_program, from_str("11111111111111111111111111111111"));
        assert_eq!(token_program, from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"));
        assert_eq!(other_program, from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"));
    }
}
