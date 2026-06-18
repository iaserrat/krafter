const SPLITMIX_INCREMENT: u64 = 0x9E3779B97F4A7C15;
const SPLITMIX_MUL_1: u64 = 0xBF58476D1CE4E5B9;
const SPLITMIX_MUL_2: u64 = 0x94D049BB133111EB;
const ROMU_MUL: u64 = 0xD3833E804F4C574B;
const ROMU_ROTATE: u32 = 27;

pub struct Rng {
    x: u64,
    y: u64,
}

impl Rng {
    pub fn seed(mut s: u64) -> Self {
        let mut sm = move || {
            s = s.wrapping_add(SPLITMIX_INCREMENT);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX_MUL_1);
            z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX_MUL_2);
            z ^ (z >> 31)
        };
        Self { x: sm(), y: sm() }
    }

    pub fn next(&mut self) -> u64 {
        let xp = self.x;
        self.x = ROMU_MUL.wrapping_mul(self.y);
        self.y = self.y.wrapping_sub(xp).rotate_left(ROMU_ROTATE);
        xp
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }

    pub fn bool(&mut self) -> bool {
        self.next() & 1 == 1
    }

    pub fn byte(&mut self) -> u8 {
        self.next() as u8
    }

    pub fn u32(&mut self) -> u32 {
        self.next() as u32
    }
}
