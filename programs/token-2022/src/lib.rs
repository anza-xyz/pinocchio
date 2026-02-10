#![no_std]

pub mod instructions;
pub mod state;

use {
    core::mem::MaybeUninit, solana_account_view::AccountView,
    solana_instruction_view::InstructionAccount,
};

solana_address::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

const ELGAMAL_PUBKEY_LEN: usize = 32;
const AE_CIPHERTEXT_LEN: usize = 36;

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

const UNINIT_ACCOUNT_REF: MaybeUninit<&AccountView> = MaybeUninit::<&AccountView>::uninit();

const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
    MaybeUninit::<InstructionAccount>::uninit();

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    let len = destination.len().min(source.len());
    // SAFETY:
    // - Both pointers have alignment 1.
    // - For valid (non-UB) references, the borrow checker guarantees no overlap.
    // - `len` is bounded by both slice lengths.
    unsafe {
        core::ptr::copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr() as *mut u8, len);
    }
}

/// The Mint that represents the native token
pub mod native_mint {
    /// There are `10^9` lamports in one SOL.
    pub const DECIMALS: u8 = 9;

    // The Mint for native SOL Token accounts.
    solana_address::declare_id!("9pan9bMn5HatX4EJdBwg9VgCa7Uz5HL8N1m5D3NdXejP");

    /// Seed for the native mint's program-derived address
    pub const PROGRAM_ADDRESS_SEEDS: &[&[u8]] = &["native-mint".as_bytes(), &[255]];

    #[cfg(test)]
    mod tests {
        use {super::*, solana_address::Address};

        #[test]
        fn expected_native_mint_id() {
            let native_mint_id =
                Address::create_program_address(PROGRAM_ADDRESS_SEEDS, &crate::id()).unwrap();
            assert_eq!(id(), native_mint_id);
        }
    }
}
