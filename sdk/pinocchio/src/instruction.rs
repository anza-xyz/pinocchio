//! Instruction types.

use core::{marker::PhantomData, ops::Deref};

use crate::{account::AccountView, Address};

/// Information about a CPI instruction.
#[derive(Debug, Clone)]
pub struct Instruction<'a, 'b, 'c, 'd>
where
    'a: 'b,
{
    /// Address of the program.
    pub program_id: &'c Address,

    /// Data expected by the program instruction.
    pub data: &'d [u8],

    /// Metadata describing accounts that should be passed to the program.
    pub accounts: &'b [AccountMeta<'a>],
}

/// Use to query and convey information about the sibling instruction components
/// when calling the `sol_get_processed_sibling_instruction` syscall.
#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessedSiblingInstruction {
    /// Length of the instruction data
    pub data_len: u64,

    /// Number of `AccountMeta` structures
    pub accounts_len: u64,
}

/// An `Account` for CPI invocations.
///
/// This struct contains the same information as an [`AccountView`], but has
/// the memory layout as expected by `sol_invoke_signed_c` syscall.
#[repr(C)]
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Clone, Debug)]
pub struct Account<'a> {
    // Address of the account.
    key: *const Address,

    // Number of lamports owned by this account.
    lamports: *const u64,

    // Length of data in bytes.
    data_len: u64,

    // On-chain data within this account.
    data: *const u8,

    // Program that owns this account.
    owner: *const Address,

    // The epoch at which this account will next owe rent.
    rent_epoch: u64,

    // Transaction was signed by this account's key?
    is_signer: bool,

    // Is the account writable?
    is_writable: bool,

    // This account's data contains a loaded program (and is now read-only).
    executable: bool,

    /// The pointers to the `AccountView` data are only valid for as long as the
    /// `&'a AccountView` lives. Instead of holding a reference to the actual `AccountView`,
    /// which would increase the size of the type, we claim to hold a reference without
    /// actually holding one using a `PhantomData<&'a AccountInfo>`.
    _account_info: PhantomData<&'a AccountView>,
}

impl<'a> From<&'a AccountView> for Account<'a> {
    fn from(account: &'a AccountView) -> Self {
        Account {
            key: account.address(),
            lamports: &account.lamports(),
            data_len: account.data_len() as u64,
            data: account.data_ptr(),
            owner: unsafe { account.owner() },
            // The `rent_epoch` field is not present in the `AccountView` struct,
            // since the value occurs after the variable data of the account in
            // the runtime input data.
            rent_epoch: 0,
            is_signer: account.is_signer(),
            is_writable: account.is_writable(),
            executable: account.executable(),
            _account_info: PhantomData::<&'a AccountView>,
        }
    }
}

/// Describes a single account read or written by a program during instruction
/// execution.
///
/// When constructing an [`Instruction`], a list of all accounts that may be
/// read or written during the execution of that instruction must be supplied.
/// Any account that may be mutated by the program during execution, either its
/// data or metadata such as held lamports, must be writable.
///
/// Note that because the Solana runtime schedules parallel transaction
/// execution around which accounts are writable, care should be taken that only
/// accounts which actually may be mutated are specified as writable.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AccountMeta<'a> {
    /// Address of the account.
    pub address: &'a Address,

    /// Indicates whether the account is writable or not.
    pub is_writable: bool,

    /// Indicates whether the account signed the instruction or not.
    pub is_signer: bool,
}

impl<'a> AccountMeta<'a> {
    /// Creates a new `AccountMeta`.
    #[inline(always)]
    pub const fn new(address: &'a Address, is_writable: bool, is_signer: bool) -> Self {
        Self {
            address,
            is_writable,
            is_signer,
        }
    }

    /// Creates a new read-only `AccountMeta`.
    #[inline(always)]
    pub const fn readonly(address: &'a Address) -> Self {
        Self::new(address, false, false)
    }

    /// Creates a new writable `AccountMeta`.
    #[inline(always)]
    pub const fn writable(address: &'a Address) -> Self {
        Self::new(address, true, false)
    }

    /// Creates a new read-only and signer `AccountMeta`.
    #[inline(always)]
    pub const fn readonly_signer(address: &'a Address) -> Self {
        Self::new(address, false, true)
    }

    /// Creates a new writable and signer `AccountMeta`.
    #[inline(always)]
    pub const fn writable_signer(address: &'a Address) -> Self {
        Self::new(address, true, true)
    }
}

impl<'a> From<&'a AccountView> for AccountMeta<'a> {
    fn from(account: &'a AccountView) -> Self {
        AccountMeta::new(
            account.address(),
            account.is_writable(),
            account.is_signer(),
        )
    }
}

/// Represents a signer seed.
///
/// This struct contains the same information as a `[u8]`, but
/// has the memory layout as expected by `sol_invoke_signed_c`
/// syscall.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Seed<'a> {
    /// Seed bytes.
    pub(crate) seed: *const u8,

    /// Length of the seed bytes.
    pub(crate) len: u64,

    /// The pointer to the seed bytes is only valid while the `&'a [u8]` lives. Instead
    /// of holding a reference to the actual `[u8]`, which would increase the size of the
    /// type, we claim to hold a reference without actually holding one using a
    /// `PhantomData<&'a [u8]>`.
    _bytes: PhantomData<&'a [u8]>,
}

impl<'a> From<&'a [u8]> for Seed<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self {
            seed: value.as_ptr(),
            len: value.len() as u64,
            _bytes: PhantomData::<&[u8]>,
        }
    }
}

impl<'a, const SIZE: usize> From<&'a [u8; SIZE]> for Seed<'a> {
    fn from(value: &'a [u8; SIZE]) -> Self {
        Self {
            seed: value.as_ptr(),
            len: value.len() as u64,
            _bytes: PhantomData::<&[u8]>,
        }
    }
}

impl Deref for Seed<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.seed, self.len as usize) }
    }
}

/// Represents a [program derived address][pda] (PDA) signer controlled by the
/// calling program.
///
/// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Signer<'a, 'b> {
    /// Signer seeds.
    pub(crate) seeds: *const Seed<'a>,

    /// Number of seeds.
    pub(crate) len: u64,

    /// The pointer to the seeds is only valid while the `&'b [Seed<'a>]` lives. Instead
    /// of holding a reference to the actual `[Seed<'a>]`, which would increase the size
    /// of the type, we claim to hold a reference without actually holding one using a
    /// `PhantomData<&'b [Seed<'a>]>`.
    _seeds: PhantomData<&'b [Seed<'a>]>,
}

impl<'a, 'b> From<&'b [Seed<'a>]> for Signer<'a, 'b> {
    fn from(value: &'b [Seed<'a>]) -> Self {
        Self {
            seeds: value.as_ptr(),
            len: value.len() as u64,
            _seeds: PhantomData::<&'b [Seed<'a>]>,
        }
    }
}

impl<'a, 'b, const SIZE: usize> From<&'b [Seed<'a>; SIZE]> for Signer<'a, 'b> {
    fn from(value: &'b [Seed<'a>; SIZE]) -> Self {
        Self {
            seeds: value.as_ptr(),
            len: value.len() as u64,
            _seeds: PhantomData::<&'b [Seed<'a>]>,
        }
    }
}

/// Convenience macro for constructing a `Signer` from a list of seeds
/// represented as byte slices.
///
/// # Example
///
/// Creating a signer for a PDA with a single seed and bump value:
/// ```
/// use pinocchio::signer;
///
/// let pda_bump = 255;
/// let signer = signer!(b"seed", &[pda_bump]);
/// ```
#[macro_export]
#[deprecated(since = "0.8.0", note = "Use `seeds!` macro instead")]
macro_rules! signer {
    ( $($seed:expr),* ) => {
            $crate::instruction::Signer::from(&[$(
                $seed.into(),
            )*])
    };
}

/// Convenience macro for constructing a `[Seed; N]` array from a list of seeds.
///
/// # Example
///
/// Creating seeds array and signer for a PDA with a single seed and bump value:
/// ```
/// use pinocchio::{seeds, instruction::Signer, Address};
///
/// let pda_bump = 0xffu8;
/// let pda_ref = &[pda_bump];  // prevent temporary value being freed
/// let example_key = Address::default();
/// let seeds = seeds!(b"seed", example_key.as_ref(), pda_ref);
/// let signer = Signer::from(&seeds);
/// ```
#[macro_export]
macro_rules! seeds {
    ( $($seed:expr),* ) => {
        [$(
            $crate::instruction::Seed::from($seed),
        )*]
    };
}
