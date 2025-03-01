use core::{mem::MaybeUninit, ops::Deref, slice::from_raw_parts};

#[cfg(target_os = "solana")]
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

    // Remaining space in the buffer.
    // JC nit: this isn't the remaining space in the buffer, maybe call it `len`
    // directly?
    offset: usize,
}

impl<const BUFFER: usize> Default for Logger<BUFFER> {
    #[inline]
    fn default() -> Self {
        Self {
            buffer: [UNINIT_BYTE; BUFFER],
            offset: 0,
        }
    }
}

impl<const BUFFER: usize> Deref for Logger<BUFFER> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.buffer.as_ptr() as *const _, self.offset) }
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
                unsafe {
                    let last = self.buffer.get_unchecked_mut(BUFFER - 1);
                    last.write(TRUNCATED);
                }
            }
        } else {
            // If someone implements `write_with_args` incorrectly, this can
            // blow past the offset and cause undefined behavior, correct?
            self.offset += value.write_with_args(&mut self.buffer[self.offset..], args);
        }

        self
    }

    /// Log the message in the buffer.
    #[inline(always)]
    pub fn log(&self) {
        log_message(self);
    }

    /// Clear the buffer.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.offset = 0;
    }

    /// Check if the buffer is empty.
    // JC nit: this was a bit confusing since `slice.is_empty()` means nearly
    // the opposite, ie. BUFFER == 0 rather than offset == 0. Is this even
    // used? If not, I'd recommend removing it or coming up with a better name
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.offset == 0
    }

    /// Check if the buffer is full.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.offset == BUFFER
    }

    /// Get the length of the buffer.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.offset
    }

    /// Get the remaining space in the buffer.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        BUFFER - self.offset
    }
}

/// Log a message.
#[inline(always)]
pub fn log_message(message: &[u8]) {
    #[cfg(target_os = "solana")]
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
                        // JC: I got worried that this was actually unsafe, since
                        // there's no bounds check. Can you add a comment saying
                        // we just checked that the buffer isn't empty, so this
                        // is safe?
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

                        // Number of available digits to write.
                        //
                        // JC: `offset` starts at max_digits, so if I write 1
                        // digit, then offset will be `9`, which means available
                        // would be `1`, which is not the number of available
                        // digits, but rather the number of written digits. Am
                        // I missing something?
                        let mut available = $max_digits - offset;

                        if precision > 0 {
                            while precision >= available {
                                available += 1;
                                offset -= 1;

                                unsafe {
                                    digits
                                        .get_unchecked_mut(offset)
                                        .write(*DIGITS.get_unchecked(0));
                                }
                            }
                            // Space for the decimal point.
                            available += 1;
                        }

                        // Size of the buffer.
                        let length = buffer.len();
                        // Determines if the value was truncated or not by calculating the
                        // number of digits that can be written.
                        let (overflow, written, fraction) = if available <= length {
                            (false, available, precision)
                        } else {
                            (true, length, precision.saturating_sub(available - length))
                        };

                        unsafe {
                            let source = digits.as_ptr().add(offset);
                            let ptr = buffer.as_mut_ptr();

                            #[cfg(target_os = "solana")]
                            {
                                // JC: Should we do a check on the size so that
                                // the syscall isn't always used? For example, if
                                // `written` is less than 32 bytes, then we do a
                                // normal copy
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
                        // JC: since this function uses an intermediate array
                        // of `MaybeUninit<u8>`, it could be nicer to have this
                        // function return the `MaybeUninit` array, so that the
                        // logger decides how to deal with overflow if the
                        // returned amount is too big. However, that isn't possible
                        // at compile-time, so maybe this is the best option.
                        if overflow {
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
                        unsafe {
                            buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                        }
                        1
                    }
                    value => {
                        let mut prefix = 0;

                        if *self < 0 {
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

        unsafe {
            buffer.get_unchecked_mut(0).write(b'"');
        }

        let mut offset = 1;
        offset += self.write(&mut buffer[offset..]);

        match buffer.len() - offset {
            0 => unsafe {
                buffer.get_unchecked_mut(offset - 1).write(TRUNCATED);
            },
            _ => {
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
        //  not, and the length of the buffer.

        let (size, truncate_end) = match args
            .iter()
            .find(|arg| matches!(arg, Argument::TruncateEnd(_) | Argument::TruncateStart(_)))
        {
            Some(Argument::TruncateEnd(size)) => (*size, Some(true)),
            Some(Argument::TruncateStart(size)) => (*size, Some(false)),
            _ => (buffer.len(), None),
        };

        // No truncate arguments were provided, so the entire string is copied to the buffer if
        // it fits; otherwise, the buffer is filled with as many characters as possible and the
        // last character is set to `TRUNCATED`.
        // JC nit: the variable name `offset` for a pointer is a little mean for
        // readers :sweat-smile:, same with `length` for "length to write", and
        // `prefix` can in fact also be a `suffix`, so maybe just be totally
        // clear and call it `written_truncated_slice_length`
        let (offset, source, length, prefix, truncated) = if truncate_end.is_none() {
            let length = core::cmp::min(size, self.len());
            (
                buffer.as_mut_ptr(),
                self.as_ptr(),
                length,
                0,
                length != self.len(),
            )
        } else {
            // JC nit: again, some of the variable names are tough to understand
            // without reading the code: `length` -> `max_length` and `ptr` ->
            // `destination` would probably be better
            let length = core::cmp::min(size, buffer.len());
            let ptr = buffer.as_mut_ptr();

            // The buffer is large enough to hold the entire string.
            if length >= self.len() {
                (ptr, self.as_ptr(), self.len(), 0, false)
            }
            // The buffer is large enough to hold the truncated slice and part of the string. In
            // In this case, the characters from the start or end of the string are copied to the
            // buffer together with the `TRUNCATED_SLICE`.
            else if length > TRUNCATED_SLICE.len() {
                // Number of characters that can be copied to the buffer.
                let length = length - TRUNCATED_SLICE.len();

                unsafe {
                    let (offset, source, destination) = if truncate_end == Some(true) {
                        (length, self.as_ptr(), ptr)
                    } else {
                        (
                            0,
                            self.as_ptr().add(self.len() - length),
                            ptr.add(TRUNCATED_SLICE.len()),
                        )
                    };
                    // Copy the truncated slice to the buffer.
                    core::ptr::copy_nonoverlapping(
                        TRUNCATED_SLICE.as_ptr(),
                        ptr.add(offset) as *mut _,
                        TRUNCATED_SLICE.len(),
                    );

                    (destination, source, length, TRUNCATED_SLICE.len(), false)
                }
            }
            // The buffer is smaller than the `PREFIX`: the buffer is filled with the `PREFIX`
            // and the last character is set to `TRUNCATED`.
            else {
                (ptr, TRUNCATED_SLICE.as_ptr(), length, 0, true)
            }
        };

        unsafe {
            core::ptr::copy_nonoverlapping(source, offset as *mut _, length);
        }

        // There might not have been space for all the value.
        if truncated {
            unsafe {
                let last = buffer.get_unchecked_mut(length - 1);
                last.write(TRUNCATED);
            }
        }

        prefix + length
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
    // JC: never seen this syntax before for macros, it's neat!
    ( @generate_write ) => {
        #[inline]
        fn write_with_args(&self, buffer: &mut [MaybeUninit<u8>], _args: &[Argument]) -> usize {
            if buffer.is_empty() {
                return 0;
            }

            // Size of the buffer.
            let length = buffer.len();

            unsafe {
                buffer.get_unchecked_mut(0).write(b'[');
            }

            let mut offset = 1;

            for value in self.iter() {
                if offset >= length {
                    unsafe {
                        buffer.get_unchecked_mut(length - 1).write(TRUNCATED);
                    }
                    offset = length;
                    break;
                }

                if offset > 1 {
                    if offset + 2 >= length {
                        unsafe {
                            buffer.get_unchecked_mut(length - 1).write(TRUNCATED);
                        }
                        offset = length;
                        break;
                    } else {
                        unsafe {
                            buffer.get_unchecked_mut(offset).write(b',');
                            buffer.get_unchecked_mut(offset + 1).write(b' ');
                        }
                        offset += 2;
                    }
                }

                // JC: shouldn't this call `write_with_args` instead? Or should
                // this function be called `debug`? Or both?
                offset += value.debug(&mut buffer[offset..]);
            }

            if offset < length {
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
