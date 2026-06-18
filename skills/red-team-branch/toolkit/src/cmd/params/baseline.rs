use super::{request, Args, CANARY};
use crate::{cmd::cov, http};

const BASELINE_JUNK_A: &str = "rtkjunk9q7w";
const BASELINE_JUNK_B: &str = "rtkjunk7e3r";

#[derive(Clone)]
pub struct Baseline {
    pub fp: u64,
    pub status_class: u16,
    pub len_bucket: u8,
    pub reflected: bool,
    pub noisy: bool,
}

pub async fn sample(
    args: &Args,
    client: &reqwest::Client,
    base: &str,
    headers: &[(String, String)],
) -> Baseline {
    let b1 = http::send_once(
        client,
        &request::spec(args, base, headers, BASELINE_JUNK_A),
        0,
    )
    .await;
    let b2 = http::send_once(
        client,
        &request::spec(args, base, headers, BASELINE_JUNK_B),
        0,
    )
    .await;
    let fp = cov::body_fp(&b1, CANARY, b"");
    let len_bucket = cov::len_bucket(b1.body_len);
    let status_class = http::status_class(b1.status);
    // Status flap (200/503 from rate limits or a flaky upstream) is noise too.
    let noisy = fp != cov::body_fp(&b2, CANARY, b"")
        || len_bucket != cov::len_bucket(b2.body_len)
        || status_class != http::status_class(b2.status);
    Baseline {
        fp,
        status_class,
        len_bucket,
        reflected: reflects(&b1),
        noisy,
    }
}

pub fn reflects(outcome: &http::Outcome) -> bool {
    outcome
        .body_raw
        .windows(CANARY.len())
        .any(|w| w == CANARY.as_bytes())
}
