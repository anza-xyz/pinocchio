use core::{mem::MaybeUninit, ops::Deref, slice::from_raw_parts};

#[cfg(target_os = "solana")]
// Syscalls provided by the SVM runtime.
extern "C" {
    pub fn sol_log_(message: *const u8, len: u64);

    pub fn sol_memcpy_(dst: *mut u8, src: *const u8, n: u64);
}

#[cfg(not(target_os = "solana"))]
extern crate std;

/// Byte representation of the digits [0, 9].
const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];

/// Bytes for a truncated `str` log message.
const TRUNCATED_SLICE: [u8; 3] = [b'.', b'.', b'.'];

/// Byte representing a truncated log.
const TRUNCATED: u8 = b'@';

/// An uninitialized byte.
const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();

/// Logger to efficiently format log messages.
///
/// The logger is a fixed size buffer that can be used to format log messages
/// before sending them to the log output. Any type that implements the `Log`
/// trait can be appended to the logger.
pub struct Logger<const BUFFER: usize> {
    // Byte buffer to store the log message.
    buffer: [MaybeUninit<u8>; BUFFER],

    // Length of the log message.
    len: usize,
}

impl<const BUFFER: usize> Default for Logger<BUFFER> {
    #[inline]
    fn default() -> Self {
        Self {
            buffer: [UNINIT_BYTE; BUFFER],
            len: 0,
        }
    }
}

impl<const BUFFER: usize> Deref for Logger<BUFFER> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        // SAFETY: the slice is created from the buffer up to the length
        // of the message.
        unsafe { from_raw_parts(self.buffer.as_ptr() as *const _, self.len) }
    }
}

impl<const BUFFER: usize> Logger<BUFFER> {
    /// Append a value to the logger.
    #[inline(always)]
    pub fn append<T: Log>(&mut self, value: T) -> &mut Self {
        self.append_with_args(value, &[]);
        self
    }

    /// Append a value to the logger with formatting arguments.
    #[inline]
    pub fn append_with_args<T: Log>(&mut self, value: T, args: &[Argument]) -> &mut Self {
        if self.is_full() {
            if BUFFER > 0 {
                // SAFETY: the buffer is checked to be full.
                unsafe {
                    let last = self.buffer.get_unchecked_mut(BUFFER - 1);
                    last.write(TRUNCATED);
                }
            }
        } else {
            self.len += value.write_with_args(&mut self.buffer[self.len..], args);

            if self.len() > BUFFER {
                // Indicates that the buffer is full.
                self.len = BUFFER;
                // SAFETY: the buffer length is checked to greater than `BUFFER`.
                unsafe {
                    let last = self.buffer.get_unchecked_mut(BUFFER - 1);
                    last.write(TRUNCATED);
                }
            }
        }

        self
    }

    /// Log the message in the buffer.
    #[inline(always)]
    pub fn log(&self) {
        log_message(self);
    }

    /// Clear the message buffer.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Check wheather the log buffer is empty or not.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check whether the log buffer is at the maximum length or not.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.len == BUFFER
    }

    /// Get the length of log message.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get the remaining space in the log buffer.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        BUFFER - self.len
    }
}

/// Log a message.
#[inline(always)]
pub fn log_message(message: &[u8]) {
    #[cfg(target_os = "solana")]
    // SAFETY: the message is always a valid pointer to a slice of bytes
    // and `sol_log_` is a syscall.
    unsafe {
        sol_log_(message.as_ptr(), message.len() as u64);
    }
    #[cfg(not(target_os = "solana"))]
    {
        let message = core::str::from_utf8(message).unwrap();
        std::println!("{}", message);
    }
}

/// Formatting arguments.
///
/// Arguments can be used to specify additional formatting options for the log message.
/// Note that types might not support all arguments.
#[non_exhaustive]
pub enum Argument {
    /// Number of decimal places to display for numbers.
    ///
    /// This is only applicable for numeric types.
    Precision(u8),

    /// Truncate the output at the end when the specified maximum number of characters
    /// is exceeded.
    ///
    /// This is only applicable for `str` types.
    TruncateEnd(usize),

    /// Truncate the output at the start when the specified maximum number of characters
    /// is exceeded.
    ///
    /// This is only applicable for `str` types.
    TruncateStart(usize),
}

/// Trait to specify the log behavior for a type.
pub trait Log {
    #[inline(always)]
    fn debug(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        self.debug_with_args(buffer, &[])
    }

    #[inline(always)]
    fn debug_with_args(&self, buffer: &mut [MaybeUninit<u8>], args: &[Argument]) -> usize {
        self.write_with_args(buffer, args)
    }

    #[inline(always)]
    fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        self.write_with_args(buffer, &[])
    }

    fn write_with_args(&self, buffer: &mut [MaybeUninit<u8>], parameters: &[Argument]) -> usize;
}

/// Implement the log trait for unsigned integer types.
macro_rules! impl_log_for_unsigned_integer {
    ( $type:tt, $max_digits:literal ) => {
        impl Log for $type {
            #[inline]
            fn write_with_args(&self, buffer: &mut [MaybeUninit<u8>], args: &[Argument]) -> usize {
                if buffer.is_empty() {
                    return 0;
                }

                match *self {
                    // Handle zero as a special case.
                    0 => {
                        // SAFETY: the buffer is checked to be non-empty.
                        unsafe {
                            buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                        }
                        1
                    }
                    mut value => {
                        let mut digits = [UNINIT_BYTE; $max_digits];
                        let mut offset = $max_digits;

                        while value > 0 {
                            let remainder = value % 10;
                            value /= 10;
                            offset -= 1;
                            // SAFETY: the offset is always within the bounds of the array since
                            // the `offset` is initialized with the maximum number of digits that
                            // the type can have and decremented on each iteration. The `digits`
                            // array is also initialized with the same length as the offset.
                            unsafe {
                                digits
                                    .get_unchecked_mut(offset)
                                    .write(*DIGITS.get_unchecked(remainder as usize));
                            }
                        }

                        let precision = if let Some(Argument::Precision(p)) = args
                            .iter()
                            .find(|arg| matches!(arg, Argument::Precision(_)))
                        {
                            *p as usize
                        } else {
                            0
                        };

                        // Number of digits written.
                        let mut written = $max_digits - offset;

                        if precision > 0 {
                            while precision >= written {
                                written += 1;
                                offset -= 1;
                                // SAFETY: the offset is always within the bounds of the array since
                                // the `offset` is initialized with the maximum number of digits that
                                // the type can have and decremented on each iteration. The `digits`
                                // array is also initialized with the same length as the offset.
                                unsafe {
                                    digits
                                        .get_unchecked_mut(offset)
                                        .write(*DIGITS.get_unchecked(0));
                                }
                            }
                            // Space for the decimal point.
                            written += 1;
                        }

                        // Size of the buffer.
                        let length = buffer.len();
                        // Determines if the value was truncated or not by calculating the
                        // number of digits that can be written.
                        let (overflow, written, fraction) = if written <= length {
                            (false, written, precision)
                        } else {
                            (true, length, precision.saturating_sub(written - length))
                        };
                        // SAFETY: the length of both `digits` and `buffer` arrays are guaranteed
                        // to be within bounds and the `written` value is always less than their
                        // maximum length.
                        unsafe {
                            let source = digits.as_ptr().add(offset);
                            let ptr = buffer.as_mut_ptr();

                            #[cfg(target_os = "solana")]
                            {
                                if precision == 0 {
                                    sol_memcpy_(ptr as *mut _, source as *const _, written as u64);
                                } else {
                                    // Integer part of the number.
                                    let integer_part = written - (fraction + 1);
                                    sol_memcpy_(
                                        ptr as *mut _,
                                        source as *const _,
                                        integer_part as u64,
                                    );

                                    // Decimal point.
                                    (ptr.add(integer_part) as *mut u8).write(b'.');

                                    // Fractional part of the number.
                                    sol_memcpy_(
                                        ptr.add(integer_part + 1) as *mut _,
                                        source.add(integer_part) as *const _,
                                        fraction as u64,
                                    );
                                }
                            }

                            #[cfg(not(target_os = "solana"))]
                            {
                                if precision == 0 {
                                    core::ptr::copy_nonoverlapping(source, ptr, written);
                                } else {
                                    // Integer part of the number.
                                    let integer_part = written - (fraction + 1);
                                    core::ptr::copy_nonoverlapping(source, ptr, integer_part);

                                    // Decimal point.
                                    (ptr.add(integer_part) as *mut u8).write(b'.');

                                    // Fractional part of the number.
                                    core::ptr::copy_nonoverlapping(
                                        source.add(integer_part),
                                        ptr.add(integer_part + 1),
                                        fraction,
                                    );
                                }
                            }
                        }

                        // There might not have been space for all the value.
                        if overflow {
                            // SAFETY: the buffer is checked to be within `written` bounds.
                            unsafe {
                                let last = buffer.get_unchecked_mut(written - 1);
                                last.write(TRUNCATED);
                            }
                        }
                        written
                    }
                }
            }
        }
    };
}

// Supported unsigned integer types.
impl_log_for_unsigned_integer!(u8, 3);
impl_log_for_unsigned_integer!(u16, 5);
impl_log_for_unsigned_integer!(u32, 10);
impl_log_for_unsigned_integer!(u64, 20);
impl_log_for_unsigned_integer!(u128, 39);
// Handle the `usize` type.
#[cfg(target_pointer_width = "32")]
impl_log_for_unsigned_integer!(usize, 10);
#[cfg(target_pointer_width = "64")]
impl_log_for_unsigned_integer!(usize, 20);

/// Implement the log trait for the signed integer types.
macro_rules! impl_log_for_signed {
    ( $type:tt ) => {
        impl Log for $type {
            #[inline]
            fn write_with_args(&self, buffer: &mut [MaybeUninit<u8>], args: &[Argument]) -> usize {
                if buffer.is_empty() {
                    return 0;
                }

                match *self {
                    // Handle zero as a special case.
                    0 => {
                        // SAFETY: the buffer is checked to be non-empty.
                        unsafe {
                            buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                        }
                        1
                    }
                    value => {
                        let mut prefix = 0;

                        if *self < 0 {
                            // SAFETY: the buffer is checked to be non-empty.
                            unsafe {
                                buffer.get_unchecked_mut(0).write(b'-');
                            }
                            prefix += 1;
                        };

                        prefix
                            + $type::unsigned_abs(value)
                                .write_with_args(&mut buffer[prefix..], args)
                    }
                }
            }
        }
    };
}

// Supported signed integer types.
impl_log_for_signed!(i8);
impl_log_for_signed!(i16);
impl_log_for_signed!(i32);
impl_log_for_signed!(i64);
impl_log_for_signed!(i128);
impl_log_for_signed!(isize);

/// Implement the log trait for the &str type.
impl Log for &str {
    #[inline]
    fn debug_with_args(&self, buffer: &mut [MaybeUninit<u8>], _args: &[Argument]) -> usize {
        if buffer.is_empty() {
            return 0;
        }
        // SAFETY: the buffer is checked to be non-empty.
        unsafe {
            buffer.get_unchecked_mut(0).write(b'"');
        }

        let mut offset = 1;
        offset += self.write(&mut buffer[offset..]);

        match buffer.len() - offset {
            0 => {
                // SAFETY: the buffer is guaranteed to be within `offset` bounds.
                unsafe {
                    buffer.get_unchecked_mut(offset - 1).write(TRUNCATED);
                }
            }
            _ => {
                // SAFETY: the buffer is guaranteed to be within `offset` bounds.
                unsafe {
                    buffer.get_unchecked_mut(offset).write(b'"');
                }
                offset += 1;
            }
        }

        offset
    }

    #[inline]
    fn write_with_args(&self, buffer: &mut [MaybeUninit<u8>], args: &[Argument]) -> usize {
        // There are 4 different cases to consider:
        //
        // 1. No arguments were provided, so the entire string is copied to the buffer if it fits;
        //    otherwise, the buffer is filled as many characters as possible and the last character
        //    is set to `TRUNCATED`.
        //
        // Then cases only applicable when precision formatting is used:
        //
        // 2. The buffer is large enough to hold the entire string: the string is copied to the
        //    buffer and the length of the string is returned.
        //
        // 3. The buffer is smaller than the string, but large enough to hold the prefix and part
        //    of the string: the prefix and part of the string are copied to the buffer. The length
        //    returned is `prefix` + number of characters copied.
        //
        // 4. The buffer is smaller than the string and the prefix: the buffer is filled with the
        //    prefix and the last character is set to `TRUNCATED`. The length returned is the length
        //    of the buffer.
        //
        // The length of the message is determined by whether a precision formatting was used or
        // not, and the length of the buffer.

        let (size, truncate_end) = match args
            .iter()
            .find(|arg| matches!(arg, Argument::TruncateEnd(_) | Argument::TruncateStart(_)))
        {
            Some(Argument::TruncateEnd(size)) => (*size, Some(true)),
            Some(Argument::TruncateStart(size)) => (*size, Some(false)),
            _ => (buffer.len(), None),
        };

        // Handles the write of the `str` to the buffer.
        //
        // - `destination`: pointer to the buffer where the string will be copied. This is always
        //   the a pointer to the log buffer, but it could de in a different offset depending on
        //   whether the trucated slice is copied or not.
        //
        // - `source`: pointer to the string that will be copied. This could either be a pointer
        //   to the `str` itself or `TRUNCATE_SLICE`).
        //
        // - `length_to_write`: number of characters from `source` that will be copied.
        //
        // - `written_truncated_slice_length`: number of characters copied from `TRUNCATED_SLICE`.
        //   This is used to determine the total number of characters copied to the buffer.
        //
        // - `truncated`: indicates whether the `str` was truncated or not. This is used to set
        //   the last character of the buffer to `TRUNCATED`.
        let (destination, source, length_to_write, written_truncated_slice_length, truncated) =
            // No truncate arguments were provided, so the entire `str` is copied to the buffer
            // if it fits; otherwise indicates that the `str` was truncated.
            if truncate_end.is_none() {
                let length = core::cmp::min(size, self.len());
                (
                    buffer.as_mut_ptr(),
                    self.as_ptr(),
                    length,
                    0,
                    length != self.len(),
                )
            } else {
                let max_length = core::cmp::min(size, buffer.len());
                let destination = buffer.as_mut_ptr();

                // The buffer is large enough to hold the entire `str`, so no need to use the
                // truncate args.
                if max_length >= self.len() {
                    (destination, self.as_ptr(), self.len(), 0, false)
                }
                // The buffer is large enough to hold the truncated slice and part of the string.
                // In this case, the characters from the start or end of the string are copied to
                // the buffer together with the `TRUNCATED_SLICE`.
                else if max_length > TRUNCATED_SLICE.len() {
                    // Number of characters that can be copied to the buffer.
                    let length = max_length - TRUNCATED_SLICE.len();
                    // SAFETY: the `destination` is always within `length` bounds.
                    unsafe {
                        let (offset, source, destination) = if truncate_end == Some(true) {
                            (length, self.as_ptr(), destination)
                        } else {
                            (
                                0,
                                self.as_ptr().add(self.len() - length),
                                destination.add(TRUNCATED_SLICE.len()),
                            )
                        };
                        // Copy the truncated slice to the buffer.
                        core::ptr::copy_nonoverlapping(
                            TRUNCATED_SLICE.as_ptr(),
                            destination.add(offset) as *mut _,
                            TRUNCATED_SLICE.len(),
                        );

                        (destination, source, length, TRUNCATED_SLICE.len(), false)
                    }
                }
                // The buffer is smaller than the `PREFIX`: the buffer is filled with the `PREFIX`
                // and the last character is set to `TRUNCATED`.
                else {
                    (destination, TRUNCATED_SLICE.as_ptr(), max_length, 0, true)
                }
            };

        // SAFETY: the `destination` is always within `length_to_write` bounds.
        unsafe {
            core::ptr::copy_nonoverlapping(source, destination as *mut _, length_to_write);
        }

        // There might not have been space for all the value.
        if truncated {
            // SAFETY: the `destination` is always within `length_to_write` bounds.
            unsafe {
                let last = buffer.get_unchecked_mut(length_to_write - 1);
                last.write(TRUNCATED);
            }
        }

        written_truncated_slice_length + length_to_write
    }
}

/// Implement the log trait for the slice type.
macro_rules! impl_log_for_slice {
    ( [$type:ident] ) => {
        impl<$type> Log for &[$type]
        where
            $type: Log
        {
            impl_log_for_slice!(@generate_write);
        }
    };
    ( [$type:ident; $size:ident] ) => {
        impl<$type, const $size: usize> Log for &[$type; $size]
        where
            $type: Log
        {
            impl_log_for_slice!(@generate_write);
        }
    };
    ( @generate_write ) => {
        #[inline]
        fn write_with_args(&self, buffer: &mut [MaybeUninit<u8>], _args: &[Argument]) -> usize {
            if buffer.is_empty() {
                return 0;
            }

            // Size of the buffer.
            let length = buffer.len();
            // SAFETY: the buffer is checked to be non-empty.
            unsafe {
                buffer.get_unchecked_mut(0).write(b'[');
            }

            let mut offset = 1;

            for value in self.iter() {
                if offset >= length {
                    // SAFETY: the buffer is checked to be non-empty and the `length`
                    // represents the buffer length.
                    unsafe {
                        buffer.get_unchecked_mut(length - 1).write(TRUNCATED);
                    }
                    offset = length;
                    break;
                }

                if offset > 1 {
                    if offset + 2 >= length {
                        // SAFETY: the buffer is checked to be non-empty and the `length`
                        // represents the buffer length.
                        unsafe {
                            buffer.get_unchecked_mut(length - 1).write(TRUNCATED);
                        }
                        offset = length;
                        break;
                    } else {
                        // SAFETY: the buffer is checked to be non-empty and the `offset`
                        // is smaller than the buffer length.
                        unsafe {
                            buffer.get_unchecked_mut(offset).write(b',');
                            buffer.get_unchecked_mut(offset + 1).write(b' ');
                        }
                        offset += 2;
                    }
                }

                offset += value.debug(&mut buffer[offset..]);
            }

            if offset < length {
                // SAFETY: the buffer is checked to be non-empty and the `offset`
                // is smaller than the buffer length.
                unsafe {
                    buffer.get_unchecked_mut(offset).write(b']');
                }
                offset += 1;
            }

            offset
        }
    };
}

// Supported slice types.
impl_log_for_slice!([T]);
impl_log_for_slice!([T; N]);
