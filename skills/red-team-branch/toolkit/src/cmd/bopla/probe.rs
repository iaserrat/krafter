use super::{args::Args, field, request};
use crate::http;
use serde_json::Value;

const WRITE_SNIPPET_LEN: usize = 200;

/// Mass-assignment requires a negative control: the privileged value must be
/// absent/different BEFORE the write and present AFTER, read independently.
#[derive(PartialEq)]
pub(super) enum Status {
    Accepted,
    Cleared,
    Inconclusive,
}

pub(super) struct ProbeResult {
    pub(super) write_status: u16,
    pub(super) read_status: u16,
    pub(super) status: Status,
}

pub(super) async fn run(
    args: &Args,
    client: &reqwest::Client,
    target: &request::Target,
    base: &serde_json::Map<String, Value>,
    key: &str,
    value: &Value,
) -> ProbeResult {
    // Without an independent read the write echo cannot be told from a commit.
    let Some(read) = target.read_url.as_ref().map(|_| request::read(target)) else {
        let write = http::send_once(client, &request::write(args, target, base, key, value), WRITE_SNIPPET_LEN).await;
        return ProbeResult { write_status: write.status, read_status: 0, status: Status::Inconclusive };
    };
    let before = http::send_once(client, &read, http::NO_SNIPPET_LEN).await;
    let baseline = field::persisted(&before.body_raw, key, value);
    let write = http::send_once(client, &request::write(args, target, base, key, value), WRITE_SNIPPET_LEN).await;
    let after = http::send_once(client, &read, http::NO_SNIPPET_LEN).await;
    let now = field::persisted(&after.body_raw, key, value);
    ProbeResult {
        write_status: write.status,
        read_status: after.status,
        status: if now && !baseline { Status::Accepted } else { Status::Cleared },
    }
}
