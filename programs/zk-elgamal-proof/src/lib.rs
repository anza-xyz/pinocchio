#![no_std]

pub mod instructions;

use {
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_instruction_view::{cpi::invoke, InstructionAccount, InstructionView},
    solana_program_error::ProgramResult,
};

solana_address::declare_id!("ZkE1Gama1Proof11111111111111111111111111111");

/// Byte length of a zero-ciphertext proof with context
pub const ZERO_CIPHERTEXT_PROOF_FULL_LEN: usize = 96 + 96; // 96 for context + 96 for proof

/// Byte length of a ciphertext-ciphertext equality proof with context
pub const CIPHERTEXT_CIPHERTEXT_EQUALITY_PROOF_FULL_LEN: usize = 192 + 224; // 192 for context + 224 for proof

/// Byte length of a ciphertext-commitment equality proof with context
pub const CIPHERTEXT_COMMITMENT_EQUALITY_PROOF_FULL_LEN: usize = 128 + 192; // 128 for context + 192 for proof

/// Byte length of a public key validity proof with context
pub const PUBKEY_VALIDITY_PROOF_FULL_LEN: usize = 32 + 64; // 32 for context + 64 for proof

/// Byte length of a percentage with cap proof with context
pub const PERCENTAGE_WITH_CAP_PROOF_FULL_LEN: usize = 104 + 256; // 104 for context + 256 for proof

/// Byte length of a range proof for an unsigned 64-bit number with context
pub const RANGE_PROOF_U64_FULL_LEN: usize = 264 + 672; // 264 for context + 672 for proof

/// Byte length of a range proof for an unsigned 128-bit number with context
pub const RANGE_PROOF_U128_FULL_LEN: usize = 264 + 736; // 264 for context + 736 for proof

/// Byte length of a range proof for an unsigned 256-bit number with context
pub const RANGE_PROOF_U256_FULL_LEN: usize = 264 + 800; // 264 for context + 800 for proof

/// Byte length of a grouped ciphertext for 2 handles validity proof with
/// context
pub const GROUPED_CIPHERTEXT_2_HANDLES_VALIDITY_PROOF_FULL_LEN: usize = 160 + 160; // 160 for context + 160 for proof

/// Byte length of a grouped ciphertext for 3 handles validity proof with
/// context
pub const GROUPED_CIPHERTEXT_3_HANDLES_VALIDITY_PROOF_FULL_LEN: usize = 224 + 192; // 224 for context + 192 for proof

/// Byte length of a batched grouped ciphertext for 2 handles validity proof
/// with context
pub const BATCHED_GROUPED_CIPHERTEXT_2_HANDLES_VALIDITY_PROOF_FULL_LEN: usize = 256 + 160; // 256 for context + 160 for proof

/// Byte length of a batched grouped ciphertext for 3 handles validity proof
/// with context
pub const BATCHED_GROUPED_CIPHERTEXT_3_HANDLES_VALIDITY_PROOF_FULL_LEN: usize = 352 + 192; // 352 for context + 192 for proof

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

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

/// An enum that represents a proof.
///
/// It can contain two types of proofs:
///
/// 1. A reference to an account that contains a proof. The `offset` field
///    specifies where in the account the proof is located.
/// 2. A proof stored in a byte array of size `PROOF_LEN`.
#[derive(Clone, Debug, PartialEq)]
pub enum Proof<'a, const PROOF_LEN: usize> {
    Account {
        account: &'a AccountView,
        offset: u32,
    },
    Data(&'a [u8; PROOF_LEN]),
}

/// A struct that holds references to the context state account and authority.
///
/// It is used to provide information about the context state when invoking an
/// instruction.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextStateInfo<'a> {
    pub context_state_account: &'a AccountView,
    pub context_state_authority: &'a AccountView,
}

macro_rules! create_instruction_struct {
    (
        DOC_MAIN = $doc_main:literal,
        DOC_AUX = $doc_aux:literal,
        INSTRUCTION_NAME = $name:ident,
        DISCRIMINATOR = $discriminator:expr,
        PROOF_LEN = $proof_len:expr
    ) => {
        #[doc = $doc_main]
        ///
        #[doc = $doc_aux]
        ///
        /// Accounts expected by this instruction:
        ///
        ///   There are four ways to structure the accounts, depending on whether the
        ///   proof is provided as instruction data or in a separate account, and
        ///   whether a proof context is created.
        ///
        ///   1. **Proof in instruction data, no context state:**
        ///      - No accounts are required.
        ///
        ///   2. **Proof in instruction data, with context state:**
        ///      - `[writable]` The proof context account to create.
        ///      - `[]` The proof context account owner.
        ///
        ///   3. **Proof in account, no context state:**
        ///      - `[]` Account to read the proof from.
        ///
        ///   4. **Proof in account, with context state:**
        ///      - `[]` Account to read the proof from.
        ///      - `[writable]` The proof context account to create.
        ///      - `[]` The proof context account owner.
        pub struct $name<'a, 'b> {
            /// Optional context state info.
            pub context_state_info: Option<$crate::ContextStateInfo<'a>>,
            /// Proof.
            pub proof: $crate::Proof<'b, $proof_len>,
        }

        impl $name<'_, '_> {
            #[inline(always)]
            pub fn invoke(&self) -> ::solana_program_error::ProgramResult {
                match self.proof {
                    $crate::Proof::Account {
                        account: proof_account,
                        offset,
                    } => {
                        // Instruction data layout:
                        // - [0]: instruction discriminator (1 byte, u8)
                        // - [1..5]: offset (4 bytes, u32)
                        let mut instruction_data = [$crate::UNINIT_BYTE; 1 + 4];

                        instruction_data[0].write($discriminator);
                        $crate::write_bytes(&mut instruction_data[1..], &offset.to_le_bytes());

                        let instruction_data =
                            unsafe { ::core::slice::from_raw_parts(instruction_data.as_ptr() as _, 1 + 4) };

                        if let Some(ref context_state_info) = self.context_state_info {
                            let instruction_accounts: [::solana_instruction_view::InstructionAccount; 3] = [
                                ::solana_instruction_view::InstructionAccount::readonly(proof_account.address()),
                                ::solana_instruction_view::InstructionAccount::writable(
                                    context_state_info.context_state_account.address(),
                                ),
                                ::solana_instruction_view::InstructionAccount::readonly(
                                    context_state_info.context_state_authority.address(),
                                ),
                            ];

                            $crate::build_and_invoke_instruction(
                                &instruction_accounts,
                                instruction_data,
                                &[
                                    proof_account,
                                    context_state_info.context_state_account,
                                    context_state_info.context_state_authority,
                                ],
                            )
                        } else {
                            let instruction_accounts: [::solana_instruction_view::InstructionAccount; 1] =
                                [::solana_instruction_view::InstructionAccount::readonly(proof_account.address())];

                            $crate::build_and_invoke_instruction(
                                &instruction_accounts,
                                instruction_data,
                                &[proof_account],
                            )
                        }
                    }
                    $crate::Proof::Data(proof_data) => {
                        // Instruction data layout:
                        // - [0]: instruction discriminator (1 byte, u8)
                        // - [1..=$proof_len]: proof
                        let mut instruction_data = [$crate::UNINIT_BYTE; 1 + $proof_len];

                        instruction_data[0].write($discriminator);
                        $crate::write_bytes(&mut instruction_data[1..], proof_data);

                        let instruction_data = unsafe {
                            ::core::slice::from_raw_parts(instruction_data.as_ptr() as _, 1 + $proof_len)
                        };

                        if let Some(ref context_state_info) = self.context_state_info {
                            let instruction_accounts: [::solana_instruction_view::InstructionAccount; 2] = [
                                ::solana_instruction_view::InstructionAccount::writable(
                                    context_state_info.context_state_account.address(),
                                ),
                                ::solana_instruction_view::InstructionAccount::readonly(
                                    context_state_info.context_state_authority.address(),
                                ),
                            ];

                            $crate::build_and_invoke_instruction(
                                &instruction_accounts,
                                instruction_data,
                                &[
                                    context_state_info.context_state_account,
                                    context_state_info.context_state_authority,
                                ],
                            )
                        } else {
                            $crate::build_and_invoke_instruction(&[], instruction_data, &[])
                        }
                    }
                }
            }
        }
    };
}

use create_instruction_struct;

#[inline(always)]
fn build_and_invoke_instruction<const ACCOUNTS: usize>(
    accounts: &[InstructionAccount],
    data: &[u8],
    account_views: &[&AccountView; ACCOUNTS],
) -> ProgramResult {
    let instruction = InstructionView {
        program_id: &crate::ID,
        accounts,
        data,
    };
    invoke(&instruction, account_views)
}
