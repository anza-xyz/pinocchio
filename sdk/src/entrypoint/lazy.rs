//! Defines the lazy program entrypoint and the context to access the
//! input buffer.

use crate::{
    account::{AccountView, RuntimeAccount},
    entrypoint::{NON_DUP_MARKER, STATIC_ACCOUNT_DATA},
    error::ProgramError,
    hint::{assume_unchecked, unlikely},
    Address, BPF_ALIGN_OF_U128,
};

/// Declare the lazy program entrypoint.
///
/// This entrypoint is defined as *lazy* because it does not read the accounts
/// upfront. Instead, it provides an [`InstructionContext`] to access input
/// information on demand. This is useful when the program needs more control
/// over the compute units it uses. The trade-off is that the program is
/// responsible for managing potential duplicated accounts and set up a `global
/// allocator` and `panic handler`.
///
/// The usual use-case for a [`crate::lazy_program_entrypoint!`] is small
/// programs with a single instruction. For most use-cases, it is recommended to
/// use the [`crate::program_entrypoint!`] macro instead.
///
/// This macro emits the boilerplate necessary to begin program execution,
/// calling a provided function to process the program instruction supplied by
/// the runtime, and reporting its result to the runtime. Note that it does not
/// set up a global allocator nor a panic handler.
///
/// The only argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///    mut context: InstructionContext, // wrapper around the input buffer
/// ) -> ProgramResult;
/// ```
///
/// # Example
///
/// Defining an entrypoint and making it conditional on the `bpf-entrypoint`
/// feature. Although the `entrypoint` module is written inline in this example,
/// it is common to put it into its own file.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///
///     use pinocchio::{
///         default_allocator,
///         default_panic_handler,
///         entrypoint::InstructionContext,
///         lazy_program_entrypoint,
///         ProgramResult
///     };
///
///     lazy_program_entrypoint!(process_instruction);
///     default_allocator!();
///     default_panic_handler!();
///
///     pub fn process_instruction(
///         mut context: InstructionContext,
///     ) -> ProgramResult {
///         Ok(())
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! lazy_program_entrypoint {
    ( $process_instruction:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            match $process_instruction($crate::entrypoint::lazy::InstructionContext::new_unchecked(
                input,
            )) {
                Ok(_) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

/// Dup guard: controls how duplicate accounts are handled.
///
/// The associated `Output` type determines the return type of
/// [`InstructionContext::next_account_guarded`]:
/// - Guards that allow duplicates return [`MaybeAccount`].
/// - Guards that reject or assume non-duplicate return [`AccountView`].
///
/// # Safety
///
/// Implementations must correctly read and advance the buffer according to
/// the account's duplicate status. The returned `*mut u8` must point to the
/// next account header (or instruction data if this was the last account).
pub unsafe trait DupGuard {
    type Output;

    /// Reads and wraps the account, advancing the buffer.
    ///
    /// Returns `(output, new_buffer_position)` on success.
    ///
    /// # Safety
    ///
    /// - `account` must point to a valid `RuntimeAccount`.
    /// - `after_header` must point 8 bytes past `account` (the post-header position).
    unsafe fn resolve<S: DataGuard>(
        &self,
        account: *mut RuntimeAccount,
        after_header: *mut u8,
        data_guard: &S,
    ) -> Result<(Self::Output, *mut u8), ProgramError>;
}

#[inline(always)]
unsafe fn read_non_dup_account<S: DataGuard>(
    account: *mut RuntimeAccount,
    after_header: *mut u8,
    data_guard: &S,
) -> Result<(AccountView, *mut u8), ProgramError> {
    data_guard.check_account(account)?;
    let new_buf = data_guard.advance_buffer(account, after_header);
    let acct = AccountView::new_unchecked(account);
    assume_unchecked(
        (acct.data_ptr() as *const u64).is_aligned(),
        "account data not aligned",
    );
    data_guard.inform_size(acct.data_len());
    Ok((acct, new_buf))
}

/// Data guard: controls how account data is validated and how the buffer
/// pointer is advanced.
///
/// # Safety
///
/// `advance_buffer` must return a pointer exactly past the account's data
/// (including alignment padding). Returning a wrong pointer corrupts all
/// subsequent reads (UB). The default implementation reads `data_len` at
/// runtime and is always correct. Overrides must compute the same result.
pub unsafe trait DataGuard {
    /// # Safety
    ///
    /// `account` must point to a valid `RuntimeAccount`.
    unsafe fn check_account(&self, account: *const RuntimeAccount) -> Result<(), ProgramError>;

    /// Hint to the compiler that account data has the validated size.
    ///
    /// Called after `check_account` succeeds. Implementations may use
    /// `assume_unchecked`, so `size` must equal the value validated by
    /// `check_account` — passing an incorrect value is UB.
    #[inline(always)]
    fn inform_size(&self, _size: usize) {}

    /// # Safety
    ///
    /// `account` must point to a valid `RuntimeAccount` and `buffer` must
    /// point past the account header (8-byte offset already applied).
    #[inline(always)]
    unsafe fn advance_buffer(&self, account: *const RuntimeAccount, buffer: *mut u8) -> *mut u8 {
        let buf = buffer.add(STATIC_ACCOUNT_DATA + (*account).data_len as usize);
        ((buf as usize + (BPF_ALIGN_OF_U128 - 1)) & !(BPF_ALIGN_OF_U128 - 1)) as *mut u8
    }
}

/// Default pass-through guard. `next_account` delegates to
/// `next_account_guarded(&NoGuards, &NoGuards)`.
pub struct NoGuards;

unsafe impl DupGuard for NoGuards {
    type Output = MaybeAccount;

    #[inline(always)]
    unsafe fn resolve<S: DataGuard>(
        &self,
        account: *mut RuntimeAccount,
        after_header: *mut u8,
        data_guard: &S,
    ) -> Result<(MaybeAccount, *mut u8), ProgramError> {
        let borrow_state = (*account).borrow_state;
        if borrow_state == NON_DUP_MARKER {
            let (acct, buf) = read_non_dup_account(account, after_header, data_guard)?;
            Ok((MaybeAccount::Account(acct), buf))
        } else {
            Ok((MaybeAccount::Duplicated(borrow_state), after_header))
        }
    }
}

unsafe impl DataGuard for NoGuards {
    #[inline(always)]
    unsafe fn check_account(&self, _account: *const RuntimeAccount) -> Result<(), ProgramError> {
        Ok(())
    }
}

/// Assumes the next account is never a duplicate.
///
/// Returns [`AccountView`] directly. The duplicate branch does not exist.
/// Passing a duplicate account causes undefined behavior: the debug build
/// asserts, but in release the buffer pointer and remaining count become
/// inconsistent with the actual buffer layout.
pub struct AssumeNeverDup(());

impl AssumeNeverDup {
    /// # Safety
    ///
    /// The caller must guarantee that the next account is not a duplicate.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        Self(())
    }
}

unsafe impl DupGuard for AssumeNeverDup {
    type Output = AccountView;

    #[inline(always)]
    unsafe fn resolve<S: DataGuard>(
        &self,
        account: *mut RuntimeAccount,
        after_header: *mut u8,
        data_guard: &S,
    ) -> Result<(AccountView, *mut u8), ProgramError> {
        debug_assert_eq!(
            (*account).borrow_state,
            NON_DUP_MARKER,
            "expected non-duplicate account"
        );
        read_non_dup_account(account, after_header, data_guard)
    }
}

/// Checks at runtime that the next account is not a duplicate.
///
/// Returns [`AccountView`] on success, or
/// [`ProgramError::AccountBorrowFailed`] on duplicate.
pub struct CheckNonDup;

unsafe impl DupGuard for CheckNonDup {
    type Output = AccountView;

    #[inline(always)]
    unsafe fn resolve<S: DataGuard>(
        &self,
        account: *mut RuntimeAccount,
        after_header: *mut u8,
        data_guard: &S,
    ) -> Result<(AccountView, *mut u8), ProgramError> {
        if (*account).borrow_state != NON_DUP_MARKER {
            return Err(ProgramError::AccountBorrowFailed);
        }
        read_non_dup_account(account, after_header, data_guard)
    }
}

/// Checks account data length against an expected byte length.
///
/// Returns [`ProgramError::InvalidAccountData`] on size mismatch.
pub struct CheckSize {
    expected_size: usize,
}

impl CheckSize {
    #[inline(always)]
    pub fn new(expected_size: usize) -> Self {
        Self { expected_size }
    }
}

unsafe impl DataGuard for CheckSize {
    #[inline(always)]
    unsafe fn check_account(&self, account: *const RuntimeAccount) -> Result<(), ProgramError> {
        if (*account).data_len as usize != self.expected_size {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    #[inline(always)]
    fn inform_size(&self, size: usize) {
        // Safety: we already checked the size in account validation
        unsafe {
            assume_unchecked(size == self.expected_size, "unexpected account data length");
        }
    }
}

/// Assumes the account data length is exactly `N` bytes.
///
/// Enables compile-time buffer stride computation. Wrong `N` corrupts the
/// buffer cursor (UB).
pub struct AssumeSize<const N: usize>(());

impl<const N: usize> AssumeSize<N> {
    /// # Safety
    ///
    /// The caller must guarantee that the next account's `data_len == N`.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        Self(())
    }
}
#[inline(always)]
const fn const_advance_buffer(by: usize, buffer: *mut u8) -> *mut u8 {
    if by == 0 || by % BPF_ALIGN_OF_U128 == 0 {
        unsafe { buffer.add(STATIC_ACCOUNT_DATA + by) }
    } else {
        unsafe { buffer.add(STATIC_ACCOUNT_DATA + by + BPF_ALIGN_OF_U128 - by % BPF_ALIGN_OF_U128) }
    }
}

// STATIC_ACCOUNT_DATA must be aligned for AssumeSize to compute const advances.
const _: () = assert!(STATIC_ACCOUNT_DATA % BPF_ALIGN_OF_U128 == 0);

unsafe impl<const N: usize> DataGuard for AssumeSize<N> {
    #[inline(always)]
    unsafe fn check_account(&self, account: *const RuntimeAccount) -> Result<(), ProgramError> {
        // SAFETY: The caller must have already validated `data_len == N`;
        // otherwise this fast path can miscompute the next cursor position.
        assume_unchecked(
            (*account).data_len as usize == N,
            "unexpected account data length",
        );
        Ok(())
    }

    #[inline(always)]
    fn inform_size(&self, _size: usize) {
        unsafe {
            assume_unchecked(_size == N, "unexpected account data length");
        }
    }

    #[inline(always)]
    unsafe fn advance_buffer(&self, _account: *const RuntimeAccount, buffer: *mut u8) -> *mut u8 {
        const_advance_buffer(N, buffer)
    }
}

/// Checks account data length for a concrete Rust type.
///
/// - Asserts `align_of::<T>() <= BPF_ALIGN_OF_U128` at compile time.
/// - Returns `InvalidAccountData` when `data_len != size_of::<T>()`.
/// - Advances the cursor by `size_of::<T>()` plus alignment padding.
pub struct CheckLikeType<T>(core::marker::PhantomData<T>);

impl<T> CheckLikeType<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T> Default for CheckLikeType<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> DataGuard for CheckLikeType<T> {
    #[inline(always)]
    unsafe fn check_account(&self, account: *const RuntimeAccount) -> Result<(), ProgramError> {
        const { assert!(core::mem::align_of::<T>() <= BPF_ALIGN_OF_U128) };
        if (*account).data_len as usize != core::mem::size_of::<T>() {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    #[inline(always)]
    fn inform_size(&self, _size: usize) {
        // SAFETY: we already checked the size in account validation
        unsafe {
            assume_unchecked(
                _size == core::mem::size_of::<T>(),
                "unexpected account data length",
            );
        }
    }

    #[inline(always)]
    unsafe fn advance_buffer(&self, _account: *const RuntimeAccount, buffer: *mut u8) -> *mut u8 {
        const_advance_buffer(core::mem::size_of::<T>(), buffer)
    }
}

/// Assumes account data length is exactly `size_of::<T>()` and skips the size
/// check in release.
///
/// Use only when the protocol guarantees the invariant upstream.
pub struct AssumeLikeType<T>(core::marker::PhantomData<T>);

impl<T> AssumeLikeType<T> {
    /// # Safety
    ///
    /// The caller must guarantee that the next account's `data_len ==
    /// size_of::<T>()` and the data is properly initialized.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

unsafe impl<T> DataGuard for AssumeLikeType<T> {
    #[inline(always)]
    unsafe fn check_account(&self, account: *const RuntimeAccount) -> Result<(), ProgramError> {
        const { assert!(core::mem::align_of::<T>() <= BPF_ALIGN_OF_U128) };
        // SAFETY: The caller must guarantee the next account has
        // `data_len == size_of::<T>()`; this removes the runtime size check.
        assume_unchecked(
            (*account).data_len as usize == core::mem::size_of::<T>(),
            "unexpected account data length",
        );
        Ok(())
    }

    #[inline(always)]
    fn inform_size(&self, _size: usize) {
        // Safety: we already checked the size in account validation
        unsafe {
            assume_unchecked(
                _size == core::mem::size_of::<T>(),
                "unexpected account data length",
            );
        }
    }

    #[inline(always)]
    unsafe fn advance_buffer(&self, _account: *const RuntimeAccount, buffer: *mut u8) -> *mut u8 {
        const_advance_buffer(core::mem::size_of::<T>(), buffer)
    }
}

/// Context to access data from the input buffer for the instruction.
///
/// This is a wrapper around the input buffer that provides methods to read the
/// accounts and instruction data. It is used by the lazy entrypoint to access
/// the input data on demand.
#[derive(Debug)]
pub struct InstructionContext {
    /// Pointer to the runtime input buffer to read from.
    ///
    /// This pointer is moved forward as accounts are read from the buffer.
    buffer: *mut u8,

    /// Number of remaining accounts.
    ///
    /// This value is decremented each time [`next_account`] is called.
    remaining: u64,
}

impl InstructionContext {
    /// Creates a new [`InstructionContext`] for the input buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the input buffer is valid, i.e., it
    /// represents the program input parameters serialized by the SVM
    /// loader. The SVM loader serializes the input parameters aligned to
    /// `8` bytes, with the first `8` bytes representing the number of
    /// accounts, followed by the accounts themselves, the instruction data
    /// and the program id.
    ///
    /// More information on the input buffer format can be found in the
    /// [SVM documentation].
    ///
    /// [SVM documentation]: https://solana.com/docs/programs/faq#input-parameter-serialization
    #[inline(always)]
    pub unsafe fn new_unchecked(input: *mut u8) -> Self {
        // SAFETY: `input` must be a valid SVM buffer and aligned for `BPF_ALIGN_OF_U128`.
        assume_unchecked(
            input.align_offset(BPF_ALIGN_OF_U128) == 0,
            "input buffer not aligned",
        );
        Self {
            // SAFETY: The first 8 bytes of the input buffer represent the
            // number of accounts when serialized by the SVM loader, which is read
            // when the context is created.
            buffer: unsafe { input.add(core::mem::size_of::<u64>()) },
            // SAFETY: Read the number of accounts from the input buffer serialized
            // by the SVM loader.
            remaining: unsafe { *(input as *const u64) },
        }
    }

    /// Reads the next account for the instruction.
    ///
    /// The account is represented as a [`MaybeAccount`], since it can either
    /// represent an [`AccountView`] or the index of a duplicated account. It
    /// is up to the caller to handle the mapping back to the source
    /// account.
    ///
    /// # Error
    ///
    /// Returns a [`ProgramError::NotEnoughAccountKeys`] error if there are
    /// no remaining accounts.
    #[inline(always)]
    pub fn next_account(&mut self) -> Result<MaybeAccount, ProgramError> {
        self.next_account_guarded(&NoGuards, &NoGuards)
    }

    /// Returns the next account for the instruction.
    ///
    /// Note that this method does *not* decrement the number of remaining
    /// accounts, but moves the input pointer forward. It is intended for
    /// use when the caller is certain on the number of remaining accounts.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that there are remaining accounts;
    /// calling this when there are no more remaining accounts results in
    /// undefined behavior.
    #[inline(always)]
    pub unsafe fn next_account_unchecked(&mut self) -> MaybeAccount {
        self.read_account_unchecked(&NoGuards, &NoGuards)
            .unwrap_unchecked()
    }

    /// Reads the next account with caller-chosen guard combination.
    ///
    /// The return type is determined by the [`DupGuard::Output`] associated
    /// type: guards that permit duplicates return [`MaybeAccount`], while
    /// guards that reject or assume non-duplicate return [`AccountView`].
    ///
    /// # Error
    ///
    /// Returns [`ProgramError::NotEnoughAccountKeys`] if no accounts remain.
    /// Other errors are guard-specific.
    ///
    /// # Note
    ///
    /// All error paths leave the cursor and remaining count unmodified.
    #[inline(always)]
    pub fn next_account_guarded<D: DupGuard, S: DataGuard>(
        &mut self,
        dup_guard: &D,
        data_guard: &S,
    ) -> Result<D::Output, ProgramError> {
        if unlikely(self.remaining == 0) {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let output = unsafe { self.read_account_unchecked(dup_guard, data_guard) }?;
        self.remaining -= 1;
        Ok(output)
    }

    /// Returns the number of remaining accounts.
    ///
    /// This value is decremented by [`next_account`] and
    /// [`next_account_guarded`] when successful.
    #[inline(always)]
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the data for the instruction.
    ///
    /// This method can only be used after all accounts have been read;
    /// otherwise, it will return a [`ProgramError::InvalidInstructionData`]
    /// error.
    #[inline(always)]
    pub fn instruction_data(&self) -> Result<&[u8], ProgramError> {
        if self.remaining > 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(unsafe { self.instruction_data_unchecked() })
    }

    /// Returns the instruction data for the instruction.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that all accounts have been read;
    /// calling this method before reading all accounts will result in
    /// undefined behavior.
    #[inline(always)]
    pub unsafe fn instruction_data_unchecked(&self) -> &[u8] {
        let data_len = *(self.buffer as *const usize);
        // shadowing the input to avoid leaving it in an inconsistent position
        let data = self.buffer.add(core::mem::size_of::<u64>());
        assume_unchecked(
            (data as *const u64).is_aligned(),
            "instruction data not aligned",
        );
        core::slice::from_raw_parts(data, data_len)
    }

    /// Returns the program id for the instruction.
    ///
    /// This method can only be used after all accounts have been read;
    /// otherwise, it will return a [`ProgramError::InvalidInstructionData`]
    /// error.
    #[inline(always)]
    pub fn program_id(&self) -> Result<&Address, ProgramError> {
        if self.remaining > 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(unsafe { self.program_id_unchecked() })
    }

    /// Returns the program id for the instruction.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that all accounts have been read;
    /// calling this method before reading all accounts will result in
    /// undefined behavior.
    #[inline(always)]
    pub unsafe fn program_id_unchecked(&self) -> &Address {
        let data_len = *(self.buffer as *const usize);
        &*(self.buffer.add(core::mem::size_of::<u64>() + data_len) as *const Address)
    }

    /// Returns the current buffer cursor position.
    #[inline(always)]
    pub fn cursor(&self) -> *mut u8 {
        self.buffer
    }

    #[allow(clippy::cast_ptr_alignment)]
    #[inline(always)]
    unsafe fn read_account_unchecked<D: DupGuard, S: DataGuard>(
        &mut self,
        dup_guard: &D,
        data_guard: &S,
    ) -> Result<D::Output, ProgramError> {
        assume_unchecked(
            self.buffer.align_offset(BPF_ALIGN_OF_U128) == 0,
            "buffer not aligned",
        );
        let account: *mut RuntimeAccount = self.buffer as *mut RuntimeAccount;
        let after_header = self.buffer.add(core::mem::size_of::<u64>());
        let (output, new_buffer) = dup_guard.resolve(account, after_header, data_guard)?;
        self.buffer = new_buffer;
        Ok(output)
    }
}

/// Wrapper type around an [`AccountView`] that may be a duplicate.
#[cfg_attr(feature = "copy", derive(Copy))]
#[derive(Debug, Clone)]
pub enum MaybeAccount {
    /// An [`AccountView`] that is not a duplicate.
    Account(AccountView),

    /// The index of the original account that was duplicated.
    Duplicated(u8),
}

impl MaybeAccount {
    /// Extracts the wrapped [`AccountView`].
    ///
    /// It is up to the caller to guarantee that the [`MaybeAccount`] really is
    /// in an [`MaybeAccount::Account`]. Calling this method when the
    /// variant is a [`MaybeAccount::Duplicated`] will result in a panic.
    #[inline(always)]
    pub fn assume_account(self) -> AccountView {
        let MaybeAccount::Account(account) = self else {
            panic!("Duplicated account")
        };
        account
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::entrypoint::test_utils::{
            create_input, create_input_custom, create_input_with_duplicates, AccountDesc,
            MOCK_PROGRAM_ID,
        },
        crate::error::ProgramError,
    };

    const IX_DATA: [u8; 8] = [0xAB; 8];

    #[repr(C)]
    struct MyType {
        _a: u64,
        _b: u64,
    }

    #[test]
    fn test_dup_guard_error_leaves_context_untouched() {
        let mut input = unsafe { create_input_with_duplicates(3, &IX_DATA, 2) };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let remaining_before = ctx.remaining();

        let acct = ctx.next_account_guarded(&CheckNonDup, &NoGuards).unwrap();
        assert_eq!(ctx.remaining(), remaining_before - 1);
        assert_eq!(acct.data_len(), 0);

        let remaining_before_dup = ctx.remaining();
        let cursor_before_dup = ctx.cursor();
        let err = ctx
            .next_account_guarded(&CheckNonDup, &NoGuards)
            .unwrap_err();
        assert_eq!(err, ProgramError::AccountBorrowFailed);
        assert_eq!(ctx.remaining(), remaining_before_dup);
        assert_eq!(ctx.cursor(), cursor_before_dup);

        let maybe = ctx.next_account_guarded(&NoGuards, &NoGuards).unwrap();
        assert!(matches!(maybe, MaybeAccount::Duplicated(0)));
    }

    #[test]
    fn test_size_guard_error_leaves_context_untouched() {
        let mut input =
            unsafe { create_input_custom(&[AccountDesc::NonDup { data_len: 32 }], &IX_DATA) };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let remaining_before = ctx.remaining();
        let cursor_before = ctx.cursor();

        let err = ctx
            .next_account_guarded(&NoGuards, &CheckSize::new(64))
            .unwrap_err();
        assert_eq!(err, ProgramError::InvalidAccountData);
        assert_eq!(ctx.remaining(), remaining_before);
        assert_eq!(ctx.cursor(), cursor_before);

        let acct = ctx
            .next_account_guarded(&NoGuards, &CheckSize::new(32))
            .unwrap()
            .assume_account();
        assert_eq!(acct.data_len(), 32);
    }

    #[test]
    fn test_multiple_guard_errors_then_success() {
        let mut input =
            unsafe { create_input_custom(&[AccountDesc::NonDup { data_len: 100 }], &IX_DATA) };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let remaining_before = ctx.remaining();

        for wrong in [0, 1, 50, 99, 101, 200] {
            let err = ctx
                .next_account_guarded(&NoGuards, &CheckSize::new(wrong))
                .unwrap_err();
            assert_eq!(err, ProgramError::InvalidAccountData);
            assert_eq!(ctx.remaining(), remaining_before);
        }

        let acct = ctx
            .next_account_guarded(&NoGuards, &CheckSize::new(100))
            .unwrap()
            .assume_account();
        assert_eq!(acct.data_len(), 100);
    }

    #[test]
    fn test_happy_path_advances_correctly() {
        let mut input = unsafe {
            create_input_custom(
                &[
                    AccountDesc::NonDup { data_len: 16 },
                    AccountDesc::NonDup { data_len: 64 },
                ],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        assert_eq!(ctx.remaining(), 2);

        let a0 = ctx.next_account().unwrap().assume_account();
        assert_eq!(a0.data_len(), 16);
        assert_eq!(ctx.remaining(), 1);

        let a1 = ctx.next_account().unwrap().assume_account();
        assert_eq!(a1.data_len(), 64);
        assert_eq!(ctx.remaining(), 0);

        let data = ctx.instruction_data().unwrap();
        assert_eq!(data, &IX_DATA);

        let pid = ctx.program_id().unwrap();
        assert_eq!(pid, &MOCK_PROGRAM_ID);
    }

    #[test]
    fn test_not_enough_account_keys() {
        let mut input = unsafe { create_input(0, &IX_DATA) };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        assert_eq!(ctx.remaining(), 0);

        let err = ctx.next_account().unwrap_err();
        assert_eq!(err, ProgramError::NotEnoughAccountKeys);
        assert_eq!(ctx.remaining(), 0);
    }

    #[test]
    fn test_instruction_data_fails_when_accounts_remain() {
        let mut input = unsafe {
            create_input_custom(
                &[
                    AccountDesc::NonDup { data_len: 0 },
                    AccountDesc::NonDup { data_len: 0 },
                ],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        // Read only first account, leave second unread
        let _ = ctx.next_account().unwrap();
        assert_eq!(ctx.remaining(), 1);

        // instruction_data() should fail because accounts remain
        let err = ctx.instruction_data().unwrap_err();
        assert_eq!(err, ProgramError::InvalidInstructionData);
        // Verify state unchanged
        assert_eq!(ctx.remaining(), 1);
    }

    #[test]
    fn test_check_like_type_rejects_wrong_size() {
        // data_len = 32 but MyType is 16 bytes
        let mut input =
            unsafe { create_input_custom(&[AccountDesc::NonDup { data_len: 32 }], &IX_DATA) };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let err = ctx
            .next_account_guarded(&NoGuards, &CheckLikeType::<MyType>::new())
            .unwrap_err();
        assert_eq!(err, ProgramError::InvalidAccountData);
        assert_eq!(ctx.remaining(), 1);
    }

    #[test]
    fn test_check_like_type_accepts_correct_size() {
        let type_size = core::mem::size_of::<MyType>();
        let mut input = unsafe {
            create_input_custom(
                &[AccountDesc::NonDup {
                    data_len: type_size,
                }],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let acct = ctx
            .next_account_guarded(&NoGuards, &CheckLikeType::<MyType>::new())
            .unwrap()
            .assume_account();
        assert_eq!(acct.data_len(), type_size);
    }

    #[test]
    fn test_assume_like_type_accepts_correct_size() {
        let type_size = core::mem::size_of::<MyType>();
        let mut input = unsafe {
            create_input_custom(
                &[AccountDesc::NonDup {
                    data_len: type_size,
                }],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let guard = unsafe { AssumeLikeType::<MyType>::new() };
        let acct = ctx
            .next_account_guarded(&NoGuards, &guard)
            .unwrap()
            .assume_account();
        assert_eq!(acct.data_len(), type_size);
        assert_eq!(ctx.remaining(), 0);

        let data = ctx.instruction_data().unwrap();
        assert_eq!(data, &IX_DATA);
    }

    #[test]
    fn test_create_input_custom_with_dup() {
        let mut input = unsafe {
            create_input_custom(
                &[
                    AccountDesc::NonDup { data_len: 48 },
                    AccountDesc::Dup { original_index: 0 },
                    AccountDesc::NonDup { data_len: 24 },
                ],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        assert_eq!(ctx.remaining(), 3);

        let a0 = ctx.next_account().unwrap().assume_account();
        assert_eq!(a0.data_len(), 48);

        let maybe = ctx.next_account().unwrap();
        assert!(matches!(maybe, MaybeAccount::Duplicated(0)));

        let a2 = ctx.next_account().unwrap().assume_account();
        assert_eq!(a2.data_len(), 24);

        assert_eq!(ctx.remaining(), 0);

        let data = ctx.instruction_data().unwrap();
        assert_eq!(data, &IX_DATA);

        let pid = ctx.program_id().unwrap();
        assert_eq!(pid, &MOCK_PROGRAM_ID);
    }

    #[test]
    fn test_assume_never_dup_on_non_duplicate() {
        let mut input =
            unsafe { create_input_custom(&[AccountDesc::NonDup { data_len: 32 }], &IX_DATA) };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let guard = unsafe { AssumeNeverDup::new() };
        let acct = ctx.next_account_guarded(&guard, &NoGuards).unwrap();
        assert_eq!(acct.data_len(), 32);
        assert_eq!(ctx.remaining(), 0);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "expected non-duplicate account")]
    fn test_assume_never_dup_panics_on_dup() {
        let mut input = unsafe {
            create_input_custom(
                &[
                    AccountDesc::NonDup { data_len: 0 },
                    AccountDesc::Dup { original_index: 0 },
                ],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };
        let _ = ctx.next_account().unwrap();
        let guard = unsafe { AssumeNeverDup::new() };
        let _ = ctx.next_account_guarded(&guard, &NoGuards);
    }

    #[test]
    fn test_cursor_advances_with_accounts() {
        let mut input = unsafe {
            create_input_custom(
                &[
                    AccountDesc::NonDup { data_len: 0 },
                    AccountDesc::NonDup { data_len: 32 },
                ],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };
        let before = ctx.cursor();
        ctx.next_account().unwrap();
        let after_first = ctx.cursor();
        assert!(after_first > before);
        ctx.next_account().unwrap();
        let after_second = ctx.cursor();
        assert!(after_second > after_first);
    }

    #[test]
    fn test_assume_size_advances_cursor_correctly() {
        let mut input = unsafe {
            create_input_custom(
                &[
                    AccountDesc::NonDup { data_len: 32 },
                    AccountDesc::NonDup { data_len: 64 },
                ],
                &IX_DATA,
            )
        };
        let mut ctx = unsafe { InstructionContext::new_unchecked(input.as_mut_ptr()) };

        let guard_a = unsafe { AssumeSize::<32>::new() };
        let a0 = ctx
            .next_account_guarded(&NoGuards, &guard_a)
            .unwrap()
            .assume_account();
        assert_eq!(a0.data_len(), 32);

        let guard_b = unsafe { AssumeSize::<64>::new() };
        let a1 = ctx
            .next_account_guarded(&NoGuards, &guard_b)
            .unwrap()
            .assume_account();
        assert_eq!(a1.data_len(), 64);

        assert_eq!(ctx.remaining(), 0);

        let data = ctx.instruction_data().unwrap();
        assert_eq!(data, &IX_DATA);

        let pid = ctx.program_id().unwrap();
        assert_eq!(pid, &MOCK_PROGRAM_ID);
    }
}
