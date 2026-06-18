use super::constants::{HAVOC_MAX_MULT, MAX_FACTOR, PERF_BASE};
use super::Seed;

const MIN_ENERGY: u32 = 1;
const FUZZ_LEVEL_SHIFT_LIMIT: u32 = 16;
const NEXT_POW2_OVERFLOW: u32 = 0x8000_0000;

pub fn next_pow2(n: u32) -> u32 {
    if n <= MIN_ENERGY {
        MIN_ENERGY
    } else {
        n.checked_next_power_of_two().unwrap_or(NEXT_POW2_OVERFLOW)
    }
}

pub fn energy(seed: &Seed) -> u32 {
    let factor = if seed.fuzz_level < FUZZ_LEVEL_SHIFT_LIMIT {
        (MIN_ENERGY.checked_shl(seed.fuzz_level).unwrap_or(u32::MAX)) / seed.n_fuzz.max(MIN_ENERGY)
    } else {
        MAX_FACTOR / next_pow2(seed.n_fuzz.max(MIN_ENERGY))
    };
    (PERF_BASE * factor.min(MAX_FACTOR)).clamp(MIN_ENERGY, HAVOC_MAX_MULT * PERF_BASE)
}
