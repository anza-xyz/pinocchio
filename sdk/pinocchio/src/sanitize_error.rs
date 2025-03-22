//! A trait for sanitizing values and members of over the wire messages.

use crate::program_error::ProgramError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SanitizeError {
    /// 0 
    /// The index of the instruction is out of bounds.
    IndexOutOfBounds,
    /// 1
    /// The value is out of bounds.
    ValueOutOfBounds,
    /// 2
    /// The value is invalid.
    InvalidValue,
}

impl From<SanitizeError> for ProgramError {
    fn from(e: SanitizeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}