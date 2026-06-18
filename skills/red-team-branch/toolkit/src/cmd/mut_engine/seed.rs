#[derive(Clone)]
pub struct Seed {
    pub buf: Vec<u8>,
    pub fuzz_level: u32,
    pub n_fuzz: u32,
    pub favored: bool,
    pub barren: bool,
}
