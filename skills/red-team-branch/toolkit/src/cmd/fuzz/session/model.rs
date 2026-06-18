use super::super::{args::Args, injector::Injector};
use crate::{cmd::cov, cmd::mut_engine};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub struct Session {
    pub args: Args,
    pub client: reqwest::Client,
    pub inj: Injector,
    pub rng: mut_engine::Rng,
    pub seed_val: u64,
    pub canary: String,
    pub base: crate::http::Outcome,
    pub lat_lower: f64,
    pub exec: u64,
    pub max_exec: u64,
    pub conc: usize,
    pub corpus: Vec<mut_engine::Seed>,
    pub dict: Vec<Vec<u8>>,
    pub dict_set: HashSet<Vec<u8>>,
    pub seen: HashSet<u64>,
    pub top_rated: HashMap<u64, usize>,
    pub dedup: HashMap<u64, cov::Repro>,
    pub started: Instant,
    pub rounds: u64,
    pub plateau_count: u64,
    pub plateau_hit: bool,
}

pub struct ChildCase {
    pub order: usize,
    pub spec: crate::http::RequestSpec,
    pub payload: Vec<u8>,
}

pub struct BatchResult {
    pub order: usize,
    pub payload: Vec<u8>,
    pub outcome: crate::http::Outcome,
}

pub fn mk_seed(buf: Vec<u8>) -> mut_engine::Seed {
    mut_engine::Seed {
        buf,
        fuzz_level: 0,
        n_fuzz: 1,
        favored: false,
        barren: false,
    }
}
