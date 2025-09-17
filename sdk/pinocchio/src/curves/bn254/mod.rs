//! BN254 curve operations

pub mod compression;
pub mod group_op;

pub use compression::*;
pub use group_op::*;

/// Size of the EC point field, in bytes.
pub const ALT_BN128_FIELD_SIZE: usize = 32;
/// A group element in G1 consists of two field elements `(x, y)`.
pub const ALT_BN128_G1_POINT_SIZE: usize = ALT_BN128_FIELD_SIZE * 2;
/// Elements in G2 is represented by 2 field-extension elements `(x, y)`.
pub const ALT_BN128_G2_POINT_SIZE: usize = ALT_BN128_FIELD_SIZE * 4;
