//! BN254 curve operations

pub mod compression;
pub mod group_op;

pub use compression::*;
pub use group_op::*;

pub const ALT_BN128_FIELD_SIZE: usize = 32;
pub const ALT_BN128_G1_SIZE: usize = ALT_BN128_FIELD_SIZE * 2; // x, y each 32 byte
pub const ALT_BN128_G2_SIZE: usize = ALT_BN128_FIELD_SIZE * 4; // x=(x0,x1), y=(y0,y1) each 32 byte
