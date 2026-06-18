use crate::{cmd::cov, cmd::fuzz::minimize, cmd::fuzz::probe::Probe, http};

pub(super) struct ConfirmedMask {
    pub(super) mask: u8,
    pub(super) latency: u64,
}

pub(super) async fn confirmed_mask(
    probe: &mut Probe<'_>,
    payload: &[u8],
    outcome: &http::Outcome,
    mut mask: u8,
) -> ConfirmedMask {
    let mut latency = outcome.latency_ms;
    if mask & cov::B_LATENCY == 0 {
        return ConfirmedMask { mask, latency };
    }
    let latency_confirmation = minimize::confirm_latency(probe, payload).await;
    if latency_confirmation.confirmed {
        latency = latency_confirmation.median;
    } else {
        mask &= !cov::B_LATENCY;
    }
    ConfirmedMask { mask, latency }
}
