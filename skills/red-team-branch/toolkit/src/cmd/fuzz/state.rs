use super::{encoding, session::Session};
use crate::cmd::cov;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub type SavedCorpus = Vec<Vec<u8>>;
pub type SavedCoverage = Vec<u64>;
pub type SavedFindings = Vec<(u64, cov::Repro)>;

pub struct SavedStateParts {
    pub corpus: SavedCorpus,
    pub seen: SavedCoverage,
    pub findings: SavedFindings,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct FuzzState {
    pub corpus: Vec<String>,
    pub seen: Vec<u64>,
    pub findings: Vec<(u64, cov::Repro)>,
}

impl FuzzState {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(path: &Path, session: &Session) -> std::io::Result<()> {
        let state = Self::from_session(session);
        std::fs::write(path, serde_json::to_string(&state).unwrap_or_default())
    }

    pub fn into_parts(self) -> SavedStateParts {
        let corpus = self
            .corpus
            .iter()
            .filter_map(|s| encoding::b64dec(s))
            .collect();
        SavedStateParts {
            corpus,
            seen: self.seen,
            findings: self.findings,
        }
    }

    fn from_session(session: &Session) -> Self {
        Self {
            corpus: session
                .corpus
                .iter()
                .map(|s| encoding::b64(&s.buf))
                .collect(),
            seen: session
                .seen
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            findings: session
                .dedup
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<HashMap<_, _>>()
                .into_iter()
                .collect(),
        }
    }
}
