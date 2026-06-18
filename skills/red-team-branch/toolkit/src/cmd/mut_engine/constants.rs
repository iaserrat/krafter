pub const ARITH_MAX: i32 = 35;
pub const MAX_FILE: usize = 1 << 20;

pub const HAVOC_STACK_POW2: usize = 7;
pub const HAVOC_BLK_SMALL: usize = 32;
pub const MAX_FACTOR: u32 = 32;
pub const HAVOC_MAX_MULT: u32 = 16;
pub const PERF_BASE: u32 = 100;

pub const INTERESTING_8: [i8; 9] = [-128, -1, 0, 1, 16, 32, 64, 100, 127];
pub const INTERESTING_16: [i16; 19] = [
    -128, -1, 0, 1, 16, 32, 64, 100, 127, -32768, -129, 128, 255, 256, 512, 1000, 1024, 4096, 32767,
];
pub const INTERESTING_32: [i32; 27] = [
    -128,
    -1,
    0,
    1,
    16,
    32,
    64,
    100,
    127,
    -32768,
    -129,
    128,
    255,
    256,
    512,
    1000,
    1024,
    4096,
    32767,
    -2147483648,
    -100663046,
    -32769,
    32768,
    65535,
    65536,
    100663045,
    2147483647,
];
