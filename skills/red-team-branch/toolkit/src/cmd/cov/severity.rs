use super::{bits::*, error::family_name};

const RESET_RANK: u32 = 100;
const TIMEOUT_RANK: u32 = 95;
const SERVER_ERROR_RANK: u32 = 80;
const ERROR_SIG_BASE_RANK: u32 = 70;
const MAX_ERROR_FAMILY_RANK_OFFSET: u8 = 9;
const LATENCY_RANK: u32 = 40;
const RAW_REFLECT_RANK: u32 = 30;
const ENCODED_REFLECT_RANK: u32 = 20;
const DIFF_RANK: u32 = 10;

pub fn severity_rank(mask: u8, family: u8) -> u32 {
    if mask & B_RESET != 0 {
        RESET_RANK
    } else if mask & B_TIMEOUT != 0 {
        TIMEOUT_RANK
    } else if mask & B_5XX != 0 {
        SERVER_ERROR_RANK
    } else if mask & B_ERRSIG != 0 {
        ERROR_SIG_BASE_RANK - family.min(MAX_ERROR_FAMILY_RANK_OFFSET) as u32
    } else if mask & B_LATENCY != 0 {
        LATENCY_RANK
    } else if mask & B_REFLECT != 0 {
        RAW_REFLECT_RANK
    } else if mask & B_ENCREFLECT != 0 {
        ENCODED_REFLECT_RANK
    } else {
        DIFF_RANK
    }
}

pub fn reasons(mask: u8, family: u8) -> Vec<String> {
    let mut r = Vec::new();
    push(
        mask,
        B_RESET,
        "connection reset/refused/closed (possible crash)",
        &mut r,
    );
    push(
        mask,
        B_TIMEOUT,
        "request timed out (hang / time-based blind)",
        &mut r,
    );
    push(mask, B_5XX, "5xx server error", &mut r);
    if mask & B_ERRSIG != 0 {
        r.push(format!(
            "error signature: {} family (absent from baseline)",
            family_name(family)
        ));
    }
    push(
        mask,
        B_LATENCY,
        "latency spike beyond baseline+7σ (confirmed by resend)",
        &mut r,
    );
    push(
        mask,
        B_REFLECT,
        "payload bytes reflected raw in response",
        &mut r,
    );
    push(
        mask,
        B_ENCREFLECT,
        "payload reflected entity-encoded",
        &mut r,
    );
    push(
        mask,
        B_DIFF,
        "differential response vs baseline (status/length class changed)",
        &mut r,
    );
    r
}

fn push(mask: u8, bit: u8, text: &str, out: &mut Vec<String>) {
    if mask & bit != 0 {
        out.push(text.into());
    }
}
