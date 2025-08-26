//! Data structures to represent account information.

#[cfg(target_os = "solana")]
use crate::syscalls::sol_memset_;
use crate::{program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use core::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Maximum number of bytes a program may add to an account during a
/// single top-level instruction.
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

/// Represents masks for borrow state of an account.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum BorrowState {
    /// Mask to check whether an account is already borrowed.
    ///
    /// This will test both data and lamports borrow state. Any position
    /// in the borrow byte that is not set means that the account
    /// is borrowed in that state.
    Borrowed = 0b_1111_1111,

    /// Mask to check whether an account is already mutably borrowed.
    ///
    /// This will test both data and lamports mutable borrow state. If
    /// one of the mutably borrowed bits is not set, then the account
    /// is mutably borrowed in that state.
    MutablyBorrowed = 0b_1000_1000,
}

/// Raw account data.
///
/// This data is wrapped in an `AccountInfo` struct, which provides safe access
/// to the data.
#[repr(C)]
#[derive(Default, Debug)]
pub(crate) struct Account {
    /// Borrow state for lamports and account data.
    ///
    /// This reuses the memory reserved for the duplicate flag in the
    /// account to track lamports and data borrows. It represents the
    /// numbers of borrows available.
    ///
    /// Bits in the borrow byte are used as follows:
    ///
    ///   * lamport mutable borrow flag
    ///     - `7 6 5 4 3 2 1 0`
    ///     - `x . . . . . . .`: `1` - the lamport field can be mutably borrowed;
    ///       `0` - there is an outstanding mutable borrow for the lamports.
    ///
    ///   * lamport immutable borrow count
    ///     - `7 6 5 4 3 2 1 0`
    ///     - `. x x x . . . .`: number of immutable borrows that can still be
    ///       allocated, for the lamports field. Ranges from 7 (`111`) to
    ///       0 (`000`).
    ///
    ///   * data mutable borrow flag
    ///     - `7 6 5 4 3 2 1 0`
    ///     - `. . . . x . . .`:  `1` - the account data can be mutably borrowed;
    ///       `0` - there is an outstanding mutable borrow for the account data.
    ///
    ///   * data immutable borrow count
    ///     - `7 6 5 4 3 2 1 0`
    ///     - `. . . . . x x x`: Number of immutable borrows that can still be
    ///       allocated, for the account data. Ranges from 7 (`111`) to 0 (`000`).
    ///
    /// Note that this values are shared across `AccountInfo`s over the
    /// same account, e.g., in case of duplicated accounts, they share
    /// the same borrow state.
    pub(crate) borrow_state: Cell<u8>,

    /// Indicates whether the transaction was signed by this account.
    is_signer: u8,

    /// Indicates whether the account is writable.
    is_writable: u8,

    /// Indicates whether this account represents a program.
    executable: u8,

    /// Difference between the original data length and the current
    /// data length.
    ///
    /// This is used to track the original data length of the account
    /// when the account is resized. The runtime guarantees that this
    /// value is zero at the start of the instruction.
    resize_delta: Cell<i32>,

    /// Public key of the account.
    key: Pubkey,

    /// Program that owns this account. Modifiable by programs.
    owner: UnsafeCell<Pubkey>,

    /// The lamports in the account. Modifiable by programs.
    lamports: Cell<u64>,

    /// Length of the data. Modifiable by programs.
    pub(crate) data_len: UnsafeCell<u64>,
}

/// Wrapper struct for an `Account`.
///
/// This struct provides safe access to the data in an `Account`. It is also
/// used to track borrows of the account data and lamports, given that an
/// account can be "shared" across multiple `AccountInfo` instances.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AccountInfo {
    /// Raw (pointer to) account data.
    ///
    /// Note that this is a pointer can be shared across multiple `AccountInfo`.
    pub(crate) raw: &'static Account,
}
impl PartialEq for AccountInfo {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.raw as *const _, other.raw as *const _)
    }
}
impl Eq for AccountInfo {}

impl AccountInfo {
    /// Public key of the account.
    #[inline(always)]
    pub fn key(&self) -> &Pubkey {
        &self.raw.key
    }

    /// Program that owns this account.
    #[inline(always)]
    pub fn owner(&self) -> Pubkey {
        unsafe { *self.owner_ref() }
    }

    /// Returns `true` if this account's owner is `other`
    #[inline(always)]
    pub fn owner_is(&self, other: &Pubkey) -> bool {
        self.owner_with_fn(|x| x == other)
    }

    /// Operate on a ref to the program that owns this account.
    #[inline(always)]
    pub fn owner_with_fn<T>(&self, f: impl FnOnce(&Pubkey) -> T) -> T {
        f(unsafe { self.owner_ref() })
    }

    /// Program that owns this account.
    ///
    /// # Safety
    /// This reference should not be held when `assign` is called.
    #[inline(always)]
    pub unsafe fn owner_ref(&self) -> &Pubkey {
        unsafe { &*self.raw.owner.get() }
    }

    /// Indicates whether the transaction was signed by this account.
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        self.raw.is_signer != 0
    }

    /// Indicates whether the account is writable.
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        self.raw.is_writable != 0
    }

    /// Indicates whether this account represents a program.
    ///
    /// Program accounts are always read-only.
    #[inline(always)]
    pub fn executable(&self) -> bool {
        self.raw.executable != 0
    }

    /// Returns the size of the data in the account.
    #[inline(always)]
    pub fn data_len(&self) -> usize {
        unsafe { *self.raw.data_len.get() as usize }
    }

    /// Returns the delta between the original data length and the current
    /// data length.
    ///
    /// This value will be different than zero if the account has been resized
    /// during the current instruction.
    #[inline(always)]
    pub fn resize_delta(&self) -> i32 {
        self.raw.resize_delta.get()
    }

    /// Returns the lamports in the account.
    #[inline(always)]
    pub fn lamports(&self) -> u64 {
        self.raw.lamports.get()
    }

    /// Sets the lamports and returns the old value.
    #[inline(always)]
    pub fn set_lamports(&self, lamports: u64) -> u64 {
        self.raw.lamports.replace(lamports)
    }

    /// Gets the cell that stores the account's lamports.
    #[inline(always)]
    pub fn borrow_lamports(&self) -> &Cell<u64> {
        &self.raw.lamports
    }

    /// Indicates whether the account data is empty.
    ///
    /// An account is considered empty if the data length is zero.
    #[inline(always)]
    pub fn data_is_empty(&self) -> bool {
        self.data_len() == 0
    }

    /// Checks if the account is owned by the given program.
    #[inline(always)]
    pub fn is_owned_by(&self, program: &Pubkey) -> bool {
        unsafe { self.owner_ref() == program }
    }

    /// Changes the owner of the account.
    ///
    /// # Safety
    ///
    /// It is undefined behavior to use this method while there is an active reference
    /// to the `owner` returned by [`Self::owner`].
    #[inline(always)]
    pub fn assign(&self, new_owner: &Pubkey) {
        unsafe { *self.raw.owner.get() = *new_owner }
    }

    /// Return true if the account borrow state is set to the given state.
    ///
    /// This will test both data and lamports borrow state.
    #[inline(always)]
    pub fn is_borrowed(&self, state: BorrowState) -> bool {
        let borrow_state = self.raw.borrow_state.get();
        let mask = state as u8;
        // If borrow state has any of the state bits of the mask not set,
        // then the account is borrowed for that state.
        (borrow_state & mask) != mask
    }

    /// Returns a read-only reference to the data in the account.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
    /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
    #[inline(always)]
    pub unsafe fn borrow_data_unchecked(&self) -> &[u8] {
        from_raw_parts(self.data_ptr().as_ptr(), self.data_len())
    }

    /// Returns a mutable reference to the data in the account.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
    /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn borrow_mut_data_unchecked(&self) -> &mut [u8] {
        from_raw_parts_mut(self.data_ptr().as_ptr(), self.data_len())
    }

    /// Tries to get a read-only reference to the data field, failing if the field
    /// is already mutable borrowed or if `7` borrows already exist.
    pub fn try_borrow_data(&self) -> Result<Ref<'_, [u8]>, ProgramError> {
        // check if the account data is already borrowed
        self.can_borrow_data()?;

        let borrow_state = self.raw.borrow_state.get();
        // Use one immutable borrow for data by subtracting `1` from the data
        // borrow counter bits; we are guaranteed that there is at least one
        // immutable borrow available.
        self.raw.borrow_state.set(borrow_state - 1);

        // return the reference to data
        Ok(Ref {
            value: NonNull::slice_from_raw_parts(self.data_ptr(), self.data_len()),
            state: &self.raw.borrow_state,
            borrow_shift: DATA_BORROW_SHIFT,
            marker: PhantomData,
        })
    }

    /// Tries to get a mutable reference to the data field, failing if the field
    /// is already borrowed in any form.
    pub fn try_borrow_mut_data(&self) -> Result<RefMut<'_, [u8]>, ProgramError> {
        // check if the account data is already borrowed
        self.can_borrow_mut_data()?;

        let borrow_state = self.raw.borrow_state.get();
        // Set the mutable data borrow bit to `0`; we are guaranteed that account
        // data is not already borrowed in any form.
        self.raw
            .borrow_state
            .set(borrow_state & !DATA_MUTABLE_BORROW_BITMASK);

        // return the mutable reference to data
        Ok(RefMut {
            value: NonNull::slice_from_raw_parts(self.data_ptr(), self.data_len()),
            state: &self.raw.borrow_state,
            borrow_bitmask: DATA_MUTABLE_BORROW_BITMASK,
            marker: PhantomData,
        })
    }

    /// Checks if it is possible to get a read-only reference to the data field, failing
    /// if the field is already mutable borrowed or if 7 borrows already exist.
    #[deprecated(since = "0.8.4", note = "Use `can_borrow_data` instead")]
    #[inline(always)]
    pub fn check_borrow_data(&self) -> Result<(), ProgramError> {
        self.can_borrow_data()
    }

    /// Checks if it is possible to get a read-only reference to the data field, failing
    /// if the field is already mutable borrowed or if 7 borrows already exist.
    #[inline(always)]
    pub fn can_borrow_data(&self) -> Result<(), ProgramError> {
        let borrow_state = self.raw.borrow_state.get();

        // Check whether the mutable data borrow bit is already in
        // use (value `0`) or not. If it is `0`, then the borrow will fail.
        if borrow_state & DATA_MUTABLE_BORROW_BITMASK == 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // Check whether we have reached the maximum immutable data borrow count
        // or not, i.e., it fails when all immutable data borrow bits are `0`.
        if borrow_state & IMMUTABLE_LICENCES_MASK == 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        Ok(())
    }

    /// Checks if it is possible to get a mutable reference to the data field, failing
    /// if the field is already borrowed in any form.
    #[deprecated(since = "0.8.4", note = "Use `can_borrow_mut_data` instead")]
    #[inline(always)]
    pub fn check_borrow_mut_data(&self) -> Result<(), ProgramError> {
        self.can_borrow_mut_data()
    }

    /// Checks if it is possible to get a mutable reference to the data field, failing
    /// if the field is already borrowed in any form.
    #[inline(always)]
    pub fn can_borrow_mut_data(&self) -> Result<(), ProgramError> {
        let borrow_state = self.raw.borrow_state.get();

        // Check whether any (mutable or immutable) data borrow bits are
        // in use (value `0`) or not.
        if borrow_state & (IMMUTABLE_LICENCES_MASK | DATA_MUTABLE_BORROW_BITMASK)
            != (IMMUTABLE_LICENCES_MASK | DATA_MUTABLE_BORROW_BITMASK)
        {
            return Err(ProgramError::AccountBorrowFailed);
        }

        Ok(())
    }

    /// Realloc (either truncating or zero extending) the account's data.
    ///
    /// The account data can be increased by up to [`MAX_PERMITTED_DATA_INCREASE`] bytes
    /// within an instruction.
    ///
    /// # Important
    ///
    /// The use of the `zero_init` parameter, which indicated whether the newly
    /// allocated memory should be zero-initialized or not, is now deprecated and
    /// ignored. The method will always zero-initialize the newly allocated memory
    /// if the new length is larger than the current data length. This is the same
    /// behavior as [`Self::resize`].
    ///
    /// This method makes assumptions about the layout and location of memory
    /// referenced by `AccountInfo` fields. It should only be called for
    /// instances of `AccountInfo` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    #[deprecated(since = "0.9.0", note = "Use AccountInfo::resize() instead")]
    #[inline(always)]
    pub fn realloc(&self, new_len: usize, _zero_init: bool) -> Result<(), ProgramError> {
        self.resize(new_len)
    }

    /// Resize (either truncating or zero extending) the account's data.
    ///
    /// The account data can be increased by up to [`MAX_PERMITTED_DATA_INCREASE`] bytes
    /// within an instruction.
    ///
    /// # Important
    ///
    /// This method makes assumptions about the layout and location of memory
    /// referenced by `AccountInfo` fields. It should only be called for
    /// instances of `AccountInfo` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    #[inline]
    pub fn resize(&self, new_len: usize) -> Result<(), ProgramError> {
        // Check wheather the account data is already borrowed.
        self.can_borrow_mut_data()?;

        // SAFETY:
        // We are checking if the account data is already borrowed, so we are safe to call
        unsafe { self.resize_unchecked(new_len) }
    }

    /// Resize (either truncating or zero extending) the account's data.
    ///
    /// The account data can be increased by up to [`MAX_PERMITTED_DATA_INCREASE`] bytes
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not check if the account data is already
    /// borrowed. The caller must guarantee that there are no active borrows to the account
    /// data.
    #[inline(always)]
    pub unsafe fn resize_unchecked(&self, new_len: usize) -> Result<(), ProgramError> {
        // Account length is always `< i32::MAX`...
        let current_len = self.data_len() as i32;
        // ...so the new length must fit in an `i32`.
        let new_len = i32::try_from(new_len).map_err(|_| ProgramError::InvalidRealloc)?;

        // Return early if length hasn't changed.
        if new_len == current_len {
            return Ok(());
        }

        let difference = new_len - current_len;
        let accumulated_resize_delta = self.resize_delta() + difference;

        // Return an error when the length increase from the original serialized data
        // length is too large and would result in an out of bounds allocation
        if accumulated_resize_delta > MAX_PERMITTED_DATA_INCREASE as i32 {
            return Err(ProgramError::InvalidRealloc);
        }

        unsafe {
            *self.raw.data_len.get() = new_len as u64;
        }
        self.raw.resize_delta.set(accumulated_resize_delta);

        if difference > 0 {
            unsafe {
                #[cfg(target_os = "solana")]
                sol_memset_(
                    self.data_ptr().add(current_len as usize).get(),
                    0,
                    difference as u64,
                );
                #[cfg(not(target_os = "solana"))]
                self.data_ptr()
                    .add(current_len as usize)
                    .write_bytes(0, difference as usize)
            }
        }

        Ok(())
    }

    /// Zero out the the account's data length, lamports and owner fields, effectively
    /// closing the account.
    ///
    /// Note: This does not zero the account data. The account data will be zeroed by
    /// the runtime at the end of the instruction where the account was closed or at the
    /// next CPI call.
    ///
    /// # Important
    ///
    /// The lamports must be moved from the account prior to closing it to prevent
    /// an unbalanced instruction error.
    #[inline]
    pub fn close(&self) -> ProgramResult {
        // make sure the account is not borrowed since we are about to
        // resize the data to zero
        if self.is_borrowed(BorrowState::Borrowed) {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // SAFETY: The are no active borrows on the account data or lamports.
        unsafe {
            // Update the resize delta since closing an account will set its data length
            // to zero (account length is always `< i32::MAX`).
            self.raw
                .resize_delta
                .set(self.resize_delta() - self.data_len() as i32);

            self.close_unchecked();
        }

        Ok(())
    }

    /// Zero out the the account's data length, lamports and owner fields, effectively
    /// closing the account.
    ///
    /// Note: This does not zero the account data. The account data will be zeroed by
    /// the runtime at the end of the instruction where the account was closed or at the
    /// next CPI call.
    ///
    /// # Important
    ///
    /// The lamports must be moved from the account prior to closing it to prevent
    /// an unbalanced instruction error.
    ///
    /// If [`Self::realloc`] or [`Self::resize`] are called after closing the account,
    /// they might incorrectly return an error for going over the limit if the account
    /// previously had space allocated since this method does not update the
    /// [`Self::resize_delta`] value.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not check if the account data is already
    /// borrowed. It should only be called when the account is not being used.
    ///
    /// It also makes assumptions about the layout and location of memory
    /// referenced by `AccountInfo` fields. It should only be called for
    /// instances of `AccountInfo` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    #[inline(always)]
    pub unsafe fn close_unchecked(&self) {
        // We take advantage that the 48 bytes before the account data are:
        // - 32 bytes for the owner
        // - 8 bytes for the lamports
        // - 8 bytes for the data_len
        //
        // So we can zero out them directly.
        #[cfg(target_os = "solana")]
        sol_memset_(self.data_ptr().sub(48), 0, 48);
    }

    /// Returns the memory address of the account data.
    fn data_ptr(&self) -> NonNull<u8> {
        unsafe {
            NonNull::new_unchecked(self.raw.data_len.get())
                .cast::<u8>()
                .add(size_of::<u64>())
        }
    }
}

/// Number of bits of the [`Account::borrow_state`] flag to shift to get to
/// the borrow state bits for account data.
///   - `7 6 5 4 3 2 1 0`
///   - `. . . . x x x x`
const DATA_BORROW_SHIFT: u8 = 0;

/// Reference to account data or lamports with checked borrow rules.
#[derive(Debug)]
pub struct Ref<'a, T: ?Sized> {
    value: NonNull<T>,
    state: &'a Cell<u8>,
    /// Indicates the type of borrow (lamports or data) by representing the
    /// shift amount.
    borrow_shift: u8,
    /// The `value` raw pointer is only valid while the `&'a T` lives so we claim
    /// to hold a reference to it.
    marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Ref<'a, T> {
    /// Maps a reference to a new type.
    #[inline]
    pub fn map<U: ?Sized, F>(orig: Ref<'a, T>, f: F) -> Ref<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        // Avoid decrementing the borrow flag on Drop.
        let orig = ManuallyDrop::new(orig);

        Ref {
            value: NonNull::from(f(&*orig)),
            state: orig.state,
            borrow_shift: orig.borrow_shift,
            marker: PhantomData,
        }
    }

    /// Filters and maps a reference to a new type.
    #[inline]
    pub fn filter_map<U: ?Sized, F>(orig: Ref<'a, T>, f: F) -> Result<Ref<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        // Avoid decrementing the borrow flag on Drop.
        let orig = ManuallyDrop::new(orig);

        match f(&*orig) {
            Some(value) => Ok(Ref {
                value: NonNull::from(value),
                state: orig.state,
                borrow_shift: orig.borrow_shift,
                marker: PhantomData,
            }),
            None => Err(ManuallyDrop::into_inner(orig)),
        }
    }
}

impl<T: ?Sized> core::ops::Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T: ?Sized> Drop for Ref<'_, T> {
    // decrement the immutable borrow count
    fn drop(&mut self) {
        self.state.set(self.state.get() + (1 << self.borrow_shift));
    }
}

/// Mask representing the mutable borrow flag for data.
const DATA_MUTABLE_BORROW_BITMASK: u8 = 0b_0000_1000;

const IMMUTABLE_LICENCES_MASK: u8 = 0b_0000_0111;

/// Mutable reference to account data or lamports with checked borrow rules.
#[derive(Debug)]
pub struct RefMut<'a, T: ?Sized> {
    value: NonNull<T>,
    state: &'a Cell<u8>,
    /// Indicates borrowed field (lamports or data) by storing the bitmask
    /// representing the mutable borrow.
    borrow_bitmask: u8,
    /// The `value` raw pointer is only valid while the `&'a T` lives so we claim
    /// to hold a reference to it.
    marker: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> RefMut<'a, T> {
    /// Maps a mutable reference to a new type.
    #[inline]
    pub fn map<U: ?Sized, F>(orig: RefMut<'a, T>, f: F) -> RefMut<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        // Avoid decrementing the borrow flag on Drop.
        let mut orig = ManuallyDrop::new(orig);

        RefMut {
            value: NonNull::from(f(&mut *orig)),
            state: orig.state,
            borrow_bitmask: orig.borrow_bitmask,
            marker: PhantomData,
        }
    }

    /// Filters and maps a mutable reference to a new type.
    #[inline]
    pub fn filter_map<U: ?Sized, F>(orig: RefMut<'a, T>, f: F) -> Result<RefMut<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        // Avoid decrementing the mutable borrow flag on Drop.
        let mut orig = ManuallyDrop::new(orig);

        match f(&mut *orig) {
            Some(value) => {
                let value = NonNull::from(value);
                Ok(RefMut {
                    value,
                    state: orig.state,
                    borrow_bitmask: orig.borrow_bitmask,
                    marker: PhantomData,
                })
            }
            None => Err(ManuallyDrop::into_inner(orig)),
        }
    }
}

impl<T: ?Sized> core::ops::Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}
impl<T: ?Sized> core::ops::DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<T: ?Sized> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        // unset the mutable borrow flag
        self.state.set(self.state.get() | self.borrow_bitmask);
    }
}

#[cfg(test)]
mod tests {
    use core::mem::{size_of, MaybeUninit};

    use crate::NON_DUP_MARKER as NOT_BORROWED;

    use super::*;

    #[test]
    fn test_data_ref() {
        let data: [u8; 4] = [0, 1, 2, 3];
        let state = Cell::new(NOT_BORROWED - (1 << DATA_BORROW_SHIFT));

        let ref_data = Ref {
            value: NonNull::from(&data),
            borrow_shift: DATA_BORROW_SHIFT,
            // borrow state must be a mutable reference
            state: &state,
            marker: PhantomData,
        };

        let new_ref = Ref::map(ref_data, |data| &data[1]);

        assert_eq!(state.get(), NOT_BORROWED - (1 << DATA_BORROW_SHIFT));
        assert_eq!(*new_ref, 1);

        let Ok(new_ref) = Ref::filter_map(new_ref, |_| Some(&3)) else {
            unreachable!()
        };

        assert_eq!(state.get(), NOT_BORROWED - (1 << DATA_BORROW_SHIFT));
        assert_eq!(*new_ref, 3);

        let new_ref = Ref::filter_map(new_ref, |_| Option::<&u8>::None);

        assert_eq!(state.get(), NOT_BORROWED - (1 << DATA_BORROW_SHIFT));
        assert!(new_ref.is_err());

        drop(new_ref);

        assert_eq!(state.get(), NOT_BORROWED);
    }

    #[test]
    fn test_data_ref_mut() {
        let mut data: [u8; 4] = [0, 1, 2, 3];
        let state = Cell::new(0b_1111_0111);

        let ref_data = RefMut {
            value: NonNull::from(&mut data),
            borrow_bitmask: DATA_MUTABLE_BORROW_BITMASK,
            // borrow state must be a mutable reference
            state: &state,
            marker: PhantomData,
        };

        let Ok(mut new_ref) = RefMut::filter_map(ref_data, |data| data.get_mut(0)) else {
            unreachable!()
        };

        *new_ref = 4;

        assert_eq!(state.get(), 0b_1111_0111);
        assert_eq!(*new_ref, 4);

        drop(new_ref);

        assert_eq!(data, [4, 1, 2, 3]);
        assert_eq!(state.get(), NOT_BORROWED);
    }

    #[test]
    fn test_borrow_data() {
        // 8-bytes aligned account data.
        let mut data = [0u64; size_of::<Account>() / size_of::<u64>()];
        // Set the borrow state.
        data[0] = NOT_BORROWED as u64;
        let account_info = AccountInfo {
            raw: unsafe { &*(data.as_mut_ptr() as *mut Account) },
        };

        // Check that we can borrow data and lamports.
        assert!(account_info.can_borrow_data().is_ok());
        assert!(account_info.can_borrow_mut_data().is_ok());

        // Borrow immutable data (7 immutable borrows available).
        const ACCOUNT_REF: MaybeUninit<Ref<[u8]>> = MaybeUninit::<Ref<[u8]>>::uninit();
        let mut refs = [ACCOUNT_REF; 7];

        refs.iter_mut().for_each(|r| {
            let Ok(data_ref) = account_info.try_borrow_data() else {
                panic!("Failed to borrow data");
            };
            r.write(data_ref);
        });

        // Check that we cannot borrow the data anymore.
        assert!(account_info.can_borrow_data().is_err());
        assert!(account_info.try_borrow_data().is_err());
        assert!(account_info.can_borrow_mut_data().is_err());
        assert!(account_info.try_borrow_mut_data().is_err());

        // Drop the immutable borrows.
        refs.iter_mut().for_each(|r| {
            let r = unsafe { r.assume_init_read() };
            drop(r);
        });

        // We should be able to borrow the data again.
        assert!(account_info.can_borrow_data().is_ok());
        assert!(account_info.can_borrow_mut_data().is_ok());

        // Borrow mutable data.
        let ref_mut = account_info.try_borrow_mut_data().unwrap();

        // Check that we cannot borrow the data anymore.
        assert!(account_info.can_borrow_data().is_err());
        assert!(account_info.try_borrow_data().is_err());
        assert!(account_info.can_borrow_mut_data().is_err());
        assert!(account_info.try_borrow_mut_data().is_err());

        drop(ref_mut);

        // We should be able to borrow the data again.
        assert!(account_info.can_borrow_data().is_ok());
        assert!(account_info.can_borrow_mut_data().is_ok());

        let borrow_state = account_info.raw.borrow_state.get();
        assert_eq!(borrow_state, NOT_BORROWED);
    }

    #[test]
    #[allow(deprecated)]
    fn test_realloc() {
        // 8-bytes aligned account data.
        let mut data = [0u64; 100 * size_of::<u64>()];

        // Set the borrow state.
        data[0] = NOT_BORROWED as u64;
        // Set the initial data length to 100.
        //   - index `10` is equal to offset `10 * size_of::<u64>() = 80` bytes.
        data[10] = 100;

        let account = AccountInfo {
            raw: unsafe { &*(data.as_mut_ptr() as *mut Account) },
        };

        let data_len = account.data_len();

        assert_eq!(data_len, 100);
        assert_eq!(account.resize_delta(), 0);

        // increase the size.

        account.realloc(200, false).unwrap();

        assert_eq!(account.data_len(), 200);
        assert_eq!(account.resize_delta(), 100);

        // decrease the size.

        account.realloc(0, false).unwrap();

        assert_eq!(account.data_len(), 0);
        assert_eq!(account.resize_delta(), -100);

        // Invalid reallocation.

        let invalid_realloc = account.realloc(10_000_000_001, false);
        assert!(invalid_realloc.is_err());

        // Reset to its original size.

        account.realloc(100, false).unwrap();

        assert_eq!(account.data_len(), 100);
        assert_eq!(account.resize_delta(), 0);

        // Consecutive reallocations.

        account.realloc(200, false).unwrap();
        account.realloc(50, false).unwrap();
        account.realloc(500, false).unwrap();

        assert_eq!(account.data_len(), 500);
        assert_eq!(account.resize_delta(), 400);

        let data = account.try_borrow_data().unwrap();
        assert_eq!(data.len(), 500);
    }
}
