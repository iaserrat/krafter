const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
const U64_BYTES: usize = 8;

pub struct Fnv {
    state: u64,
}

impl Fnv {
    pub fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    pub fn u8(&mut self, x: u8) {
        self.state ^= x as u64;
        self.state = self.state.wrapping_mul(FNV_PRIME);
    }

    pub fn u64(&mut self, x: u64) {
        for i in 0..U64_BYTES {
            self.u8((x >> (i * U64_BYTES)) as u8);
        }
    }

    pub fn bytes(&mut self, xs: &[u8]) {
        for &x in xs {
            self.u8(x);
        }
    }

    pub fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }

    pub fn finish(&self) -> u64 {
        self.state
    }
}
