//! Lightweight log utility for Solana programs.
//!
//! This crate provides a `Logger` struct that can be used to efficiently log messages
//! in a Solana program. The `Logger` struct is a wrapper around a fixed-size buffer,
//! where types that implement the `Log` trait can be appended to the buffer.
//!
//! The `Logger` struct is generic over the size of the buffer, and the buffer size
//! should be chosen based on the expected size of the log messages. When the buffer is
//! full, the log message will be truncated. This is represented by the `@` character
//! at the end of the log message.
//!
//! # Example
//!
//! Creating a `Logger` with a buffer size of `100` bytes, and appending a string and an
//! `u64` value:
//!
//! ```
//! use pinocchio_log::logger::Logger;
//!
//! let mut logger = Logger::<100>::default();
//! logger.append("balance=");
//! logger.append(1_000_000_000);
//! logger.log();
//!
//! // Clear the logger buffer.
//! logger.clear();
//!
//! logger.append(&["Hello ", "world!"]);
//! logger.log();
//! ```
//!
//! It also support adding precision to numeric types:
//!
//! ```
//! use pinocchio_log::logger::{Argument, Logger};
//!
//! let mut logger = Logger::<100>::default();
//!
//! let lamports = 1_000_000_000u64;
//!
//! logger.append("balance (SOL)=");
//! logger.append_with_args(lamports, &[Argument::Precision(9)]);
//! logger.log();
//! ```

#![no_std]

pub mod logger;

#[cfg(feature = "macro")]
pub use pinocchio_log_macro::*;

#[cfg(test)]
mod tests {
    use crate::logger::{Argument, Logger};

    /// Helper macro to generate test cases for numeric types.
    ///
    /// The test cases are generated for the given type and buffer size. The
    /// assert compares that the logger buffer length is less than or equal to
    /// the maximum length.
    macro_rules! generate_numeric_test_case {
        ( $value:expr, $max_len:expr, $($size:expr),+ $(,)? ) => {
            $(
                let mut logger = Logger::<$size>::default();
                logger.append($value);
                assert!((*logger).len() <= $max_len);
            )*
        };
    }

    /// Helper macro to generate test cases for `str` type.
    ///
    /// The test cases are generated for the given value and buffer size. The
    /// assert compares that the logger buffer length is equal to the minimum
    /// between the buffer size and the `str` length.
    macro_rules! generate_str_test_case {
        ( $str:expr, $($size:expr),+ $(,)? ) => {
            $(
                let mut logger = Logger::<$size>::default();
                logger.append(core::str::from_utf8($str).unwrap());
                assert_eq!((*logger).len(), core::cmp::min($str.len(), $size));
            )*
        };
    }

    #[test]
    fn test_logger() {
        let mut logger = Logger::<100>::default();
        logger.append("Hello ");
        logger.append("world!");

        assert!(&*logger == "Hello world!".as_bytes());

        logger.clear();

        logger.append("balance=");
        logger.append(1_000_000_000);

        assert!(&*logger == "balance=1000000000".as_bytes());
    }

    #[test]
    fn test_logger_truncated() {
        let mut logger = Logger::<8>::default();
        logger.append("Hello ");
        logger.append("world!");

        assert!(&*logger == "Hello w@".as_bytes());

        let mut logger = Logger::<12>::default();

        logger.append("balance=");
        logger.append(1_000_000_000);

        assert!(&*logger == "balance=100@".as_bytes());
    }

    #[test]
    fn test_logger_slice() {
        let mut logger = Logger::<20>::default();
        logger.append(&["Hello ", "world!"]);

        assert!(&*logger == "[\"Hello \", \"world!\"]".as_bytes());

        let mut logger = Logger::<20>::default();
        logger.append(&[123, 456]);

        assert!(&*logger == "[123, 456]".as_bytes());
    }

    #[test]
    fn test_logger_truncated_slice() {
        let mut logger = Logger::<5>::default();
        logger.append(&["Hello ", "world!"]);

        assert!(&*logger == "[\"He@".as_bytes());

        let mut logger = Logger::<4>::default();
        logger.append(&[123, 456]);

        assert!(&*logger == "[12@".as_bytes());
    }

    #[test]
    fn test_logger_signed() {
        let mut logger = Logger::<2>::default();
        logger.append(-2);

        assert!(&*logger == "-2".as_bytes());

        let mut logger = Logger::<5>::default();
        logger.append(-200_000_000);

        assert!(&*logger == "-200@".as_bytes());
    }

    #[test]
    fn test_logger_with_precision() {
        let mut logger = Logger::<10>::default();

        logger.append_with_args(200_000_000u64, &[Argument::Precision(2)]);
        assert!(&*logger == "2000000.00".as_bytes());

        logger.clear();

        logger.append_with_args(2_000_000_000u64, &[Argument::Precision(2)]);
        assert!(&*logger == "20000000.@".as_bytes());

        logger.clear();

        logger.append_with_args(2_000_000_000u64, &[Argument::Precision(5)]);
        assert!(&*logger == "20000.000@".as_bytes());

        logger.clear();

        logger.append_with_args(2_000_000_000u64, &[Argument::Precision(10)]);
        assert!(&*logger == "0.2000000@".as_bytes());

        logger.clear();

        logger.append_with_args(2u64, &[Argument::Precision(6)]);
        assert!(&*logger == "0.000002".as_bytes());

        logger.clear();

        logger.append_with_args(2u64, &[Argument::Precision(9)]);
        assert!(&*logger == "0.0000000@".as_bytes());

        logger.clear();

        logger.append_with_args(-2000000i32, &[Argument::Precision(6)]);
        assert!(&*logger == "-2.000000".as_bytes());

        logger.clear();

        logger.append_with_args(-2i64, &[Argument::Precision(9)]);
        assert!(&*logger == "-0.000000@".as_bytes());

        logger.clear();

        // This should have no effect.
        logger.append_with_args("0123456789", &[Argument::Precision(2)]);
        assert!(&*logger == "0123456789".as_bytes());
    }

    #[test]
    fn test_logger_with_truncate() {
        let mut logger = Logger::<10>::default();

        logger.append_with_args("0123456789", &[Argument::TruncateEnd(10)]);
        assert!(&*logger == "0123456789".as_bytes());

        logger.clear();

        logger.append_with_args("0123456789", &[Argument::TruncateStart(10)]);
        assert!(&*logger == "0123456789".as_bytes());

        logger.clear();

        logger.append_with_args("0123456789", &[Argument::TruncateEnd(9)]);
        assert!(&*logger == "012345...".as_bytes());

        logger.clear();

        logger.append_with_args("0123456789", &[Argument::TruncateStart(9)]);
        assert!(&*logger == "...456789".as_bytes());

        let mut logger = Logger::<3>::default();

        logger.append_with_args("0123456789", &[Argument::TruncateEnd(9)]);
        assert!(&*logger == "..@".as_bytes());

        logger.clear();

        logger.append_with_args("0123456789", &[Argument::TruncateStart(9)]);
        assert!(&*logger == "..@".as_bytes());
    }

    #[test]
    fn test_logger_with_usize() {
        let mut logger = Logger::<20>::default();

        logger.append(usize::MIN);
        assert!(&*logger == "0".as_bytes());

        logger.clear();

        logger.append(usize::MAX);

        #[cfg(target_pointer_width = "32")]
        {
            assert!(&*logger == "4294967295".as_bytes());
            assert_eq!(logger.len(), 10);
        }
        #[cfg(target_pointer_width = "64")]
        {
            assert!(&*logger == "18446744073709551615".as_bytes());
            assert_eq!(logger.len(), 20);
        }
    }

    #[test]
    fn test_logger_with_isize() {
        let mut logger = Logger::<20>::default();

        logger.append(isize::MIN);

        #[cfg(target_pointer_width = "32")]
        {
            assert!(&*logger == "-2147483648".as_bytes());
            assert_eq!(logger.len(), 11);
        }
        #[cfg(target_pointer_width = "64")]
        {
            assert!(&*logger == "-9223372036854775808".as_bytes());
            assert_eq!(logger.len(), 20);
        }

        logger.clear();

        logger.append(isize::MAX);

        #[cfg(target_pointer_width = "32")]
        {
            assert!(&*logger == "2147483647".as_bytes());
            assert_eq!(logger.len(), 10);
        }
        #[cfg(target_pointer_width = "64")]
        {
            assert!(&*logger == "9223372036854775807".as_bytes());
            assert_eq!(logger.len(), 19);
        }
    }

    #[test]
    fn test_logger_buffer_size_unsigned() {
        // Test case for an unsigned numeric type.
        macro_rules! unsigned_test_case {
            ( $( ($ty:ident, $max_len:literal) ),+ $(,)? ) => {
                    $(
                        generate_numeric_test_case!($ty::MIN, ($max_len + 1), 1,
                        2,
                        3,
                        4,
                        5,
                        6,
                        7,
                        8,
                        9,
                        10,
                        11,
                        12,
                        13,
                        14,
                        15,
                        16,
                        17,
                        18,
                        19,
                        20,
                        50,
                        100,
                        1000);
                )*
            };
        }

        unsigned_test_case!(
            (u8, 3),
            (u16, 5),
            (u32, 10),
            (u64, 20),
            (u128, 39),
            (usize, 20)
        );
    }

    #[test]
    fn test_logger_buffer_size_signed() {
        // Test case for a signed numeric type.
        macro_rules! signed_test_case {
            ( $( ($ty:ident, $max_len:literal) ),+ $(,)? ) => {
                    $(
                        generate_numeric_test_case!($ty::MIN, ($max_len + 1), 1,
                            2,
                            3,
                            4,
                            5,
                            6,
                            7,
                            8,
                            9,
                            10,
                            11,
                            12,
                            13,
                            14,
                            15,
                            16,
                            17,
                            18,
                            19,
                            20,
                            50,
                            100,
                            1000);
                    )*
            };
        }

        signed_test_case!(
            (i8, 3),
            (i16, 5),
            (i32, 10),
            (i64, 20),
            (i128, 39),
            (isize, 20)
        );
    }

    #[test]
    fn test_logger_buffer_size_str() {
        // Test case for a str type.
        macro_rules! str_test_case {
            ( $( $size:expr ),+ $(,)? ) => {
                    $(
                        generate_str_test_case!(&[b'x'; $size], 1,
                            2,
                            3,
                            4,
                            5,
                            6,
                            7,
                            8,
                            9,
                            10,
                            11,
                            12,
                            13,
                            14,
                            15,
                            16,
                            17,
                            18,
                            19,
                            20,
                            50,
                            100,
                            1000);
                    )*
            };
        }

        str_test_case!(1, 5, 10, 50, 100, 1000, 10000);
    }
}
