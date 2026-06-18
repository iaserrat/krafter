use crate::http;

// A probed body within this fraction of the calibration band is treated as the
// same dynamic-length catch-all response, not a distinct route.
const BAND_TOLERANCE_NUM: usize = 1;
const BAND_TOLERANCE_DEN: usize = 4;

/// Soft-404 baseline calibrated from two probes to distinct non-existent paths.
/// Carries a body-length band so a dynamic-length catch-all (SPA shell) is not
/// mistaken for a wall of live routes.
#[derive(Clone, Copy)]
pub(super) struct Baseline {
    pub(super) status_class: u16,
    low: usize,
    high: usize,
}

impl Baseline {
    pub(super) fn calibrate(a: &http::Outcome, b: &http::Outcome) -> Self {
        let (low, high) = (a.body_len.min(b.body_len), a.body_len.max(b.body_len));
        Self { status_class: http::status_class(a.status), low, high }
    }

    pub(super) fn len_bucket(&self) -> usize {
        self.low
    }

    /// True when `outcome` looks like the baseline (same status class and a body
    /// length inside the tolerance-widened calibration band).
    pub(super) fn indistinguishable(&self, outcome: &http::Outcome) -> bool {
        if http::status_class(outcome.status) != self.status_class {
            return false;
        }
        let span = self.high - self.low;
        let pad = (span * BAND_TOLERANCE_NUM / BAND_TOLERANCE_DEN).max(span);
        let lo = self.low.saturating_sub(pad);
        (lo..=self.high + pad).contains(&outcome.body_len)
    }
}
