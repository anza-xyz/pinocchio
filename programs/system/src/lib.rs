#![no_std]
use core::usize;

use pinocchio::{
    account_info::AccountInfo,
    cpi,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
};

pub mod instructions;

pinocchio_pubkey::declare_id!("11111111111111111111111111111111");

/// Represents the components required to construct and invoke a Solana instruction.
///
/// `InvokeParts` is a helper structure that encapsulates all parts of a Solana instruction call,
/// including the program ID, account references, account metadata, and the instruction data payload.
pub struct InvokeParts<'a, const ACCOUNTS: usize, Data> {
    pub program_id: Pubkey,
    pub accounts: [&'a AccountInfo; ACCOUNTS],
    pub account_metas: [AccountMeta<'a>; ACCOUNTS],
    pub instruction_data: Data,
}

/// A fixed-size container for raw instruction data in Solana programs.
///
/// `FullInstructionData` holds a byte array of length `N` that represents the full
/// serialized instruction data to be sent in a Solana instruction.
pub struct FullInstructionData<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> FullInstructionData<N> {
    pub fn new(data: [u8; N]) -> Self {
        Self { data }
    }
}

/// A fixed-capacity, variable-length container for instruction data in Solana programs.
///
/// `TruncatedInstructionData` stores instruction data in a `[u8; N]` buffer, with an explicit `size`
/// indicating how many bytes are actually valid. This allows efficient handling of instruction data
/// without heap allocations, while still supporting "dynamically" sized payloads up to a maximum of `N` bytes.
pub struct TruncatedInstructionData<const N: usize> {
    data: [u8; N],
    size: usize,
}

impl<const N: usize> TruncatedInstructionData<N> {
    pub fn new(data: [u8; N], size: usize) -> Self {
        Self { data, size }
    }
}
/// A trait for types that can provide a view into their instruction data as a byte slice.
///
/// This trait abstracts over different instruction data representations—fixed-size, truncated,
/// or dynamically sized—allowing them to be used interchangeably in contexts where a raw byte
/// slice is needed (e.g., for constructing or invoking Solana instructions).
pub trait InstructionData {
    /// Returns a byte slice of the instruction data.
    fn as_slice(&self) -> &[u8];

    /// Returns the number of valid bytes in the instruction data.
    ///
    /// This is equivalent to `self.as_slice().len()`, and can be used to avoid
    /// calling `as_slice()` if only the length is needed.
    fn len(&self) -> usize;
}

impl<const N: usize> InstructionData for FullInstructionData<N> {
    fn as_slice(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> InstructionData for TruncatedInstructionData<N> {
    fn as_slice(&self) -> &[u8] {
        &self.data[..self.size]
    }

    fn len(&self) -> usize {
        self.size
    }
}

/// A trait for types that represent invocable Solana instructions.
///
/// This trait abstracts over any type that can be converted into [`InvokeParts`],
/// allowing it to be invoked via Solana's CPI (cross-program invocation) mechanisms.
///
/// It provides safe and unsafe methods for both standard and signed invocations,
/// with support for unchecked access when required.
///
/// # Blanket Implementation
///
/// Any type that implements `Into<InvokeParts<'a, ACCOUNTS, Data>>` automatically
/// implements `Invoke`, as long as `Data: InstructionData`.
///
/// This makes it easy to define lightweight instruction structs that carry the required
/// invocation data and still support the full CPI interface via this trait.
///
/// # Safety
///
/// Unsafe methods are provided for advanced use cases where the caller can guarantee
/// account safety outside of runtime checks.
pub trait Invoke<'a, const ACCOUNTS: usize, Data>: Into<InvokeParts<'a, ACCOUNTS, Data>>
where
    Data: InstructionData,
{
    fn invoke(self) -> pinocchio::ProgramResult {
        self.invoke_signed(&[])
    }

    fn invoke_signed(self, signers: &[Signer]) -> pinocchio::ProgramResult {
        invoke_invoker(self.into(), signers, |ix, acc, sigs| {
            cpi::invoke_signed(&ix, acc, sigs)
        })
    }

    unsafe fn invoke_access_unchecked(self) -> pinocchio::ProgramResult {
        self.invoke_singed_access_unchecked(&[])
    }

    unsafe fn invoke_singed_access_unchecked(self, signers: &[Signer]) -> pinocchio::ProgramResult {
        invoke_invoker(self.into(), signers, |ix, acc, sigs| unsafe {
            cpi::invoke_signed_access_unchecked(&ix, acc, sigs)
        })
    }
}

impl<'a, const ACCOUNTS: usize, T, Data> Invoke<'a, ACCOUNTS, Data> for T
where
    T: Into<InvokeParts<'a, ACCOUNTS, Data>>,
    Data: InstructionData,
{
}

fn invoke_invoker<'a, const ACCOUNTS: usize>(
    invoke_parts: InvokeParts<'a, ACCOUNTS, impl InstructionData>,
    signers: &[Signer],
    invoker: impl FnOnce(Instruction, &[&AccountInfo; ACCOUNTS], &[Signer]) -> pinocchio::ProgramResult,
) -> pinocchio::ProgramResult {
    let instruction = Instruction {
        program_id: &invoke_parts.program_id,
        accounts: &invoke_parts.account_metas,
        data: invoke_parts.instruction_data.as_slice(),
    };
    invoker(instruction, &invoke_parts.accounts, signers)
}
