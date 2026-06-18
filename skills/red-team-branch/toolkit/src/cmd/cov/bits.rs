pub const B_5XX: u8 = 1;
pub const B_RESET: u8 = 2;
pub const B_TIMEOUT: u8 = 4;
pub const B_LATENCY: u8 = 8;
pub const B_REFLECT: u8 = 16;
pub const B_ENCREFLECT: u8 = 32;
pub const B_ERRSIG: u8 = 64;
pub const B_DIFF: u8 = 128;

pub const STABLE_MASK: u8 = B_5XX | B_REFLECT | B_ENCREFLECT | B_ERRSIG | B_DIFF;
pub const NORM_BYTE: u8 = b'0';
