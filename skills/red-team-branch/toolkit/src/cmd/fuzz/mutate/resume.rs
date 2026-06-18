use crate::cmd::fuzz::{session, session::Session, state::FuzzState};
use std::collections::HashSet;

pub fn resume(session: &mut Session) {
    let Some(path) = &session.args.state else {
        return;
    };
    let Ok(state) = FuzzState::load(path) else {
        return;
    };
    let before = session.corpus.len();
    let parts = state.into_parts();
    // Dedup against seeds already built by start() so resume never re-injects them.
    let mut have: HashSet<Vec<u8>> = session.corpus.iter().map(|s| s.buf.clone()).collect();
    for buf in parts.corpus {
        if have.insert(buf.clone()) {
            session.corpus.push(session::mk_seed(buf));
        }
    }
    session.seen.extend(parts.seen);
    for (key, finding) in parts.findings {
        session.dedup.entry(key).or_insert(finding);
    }
    eprintln!(
        "[rtk] resumed from {}: +{} corpus, {} coverage buckets, {} findings",
        path.display(),
        session.corpus.len() - before,
        session.seen.len(),
        session.dedup.len()
    );
}
