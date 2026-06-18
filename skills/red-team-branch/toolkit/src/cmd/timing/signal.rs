//! Two-sample timing decision. Compares the median delta against the combined
//! standard error of the two medians, so the threshold tightens as samples grow
//! (unlike a fixed k*stddev rule). Units are whatever the samples carry (us).

// Asymptotic SE of the median is sqrt(pi/2) * stddev / sqrt(n) for large n.
const MEDIAN_SE_FACTOR: f64 = 1.2533;
// Three-sigma two-sided confidence: delta must clear 3 combined standard errors.
const Z_THRESHOLD: f64 = 3.0;
const ZERO: f64 = 0.0;
const NO_SAMPLES: usize = 0;

pub(super) struct Variant {
    pub stddev: f64,
    pub count: usize,
}

pub(super) fn detect(median_delta: f64, a: Variant, b: Variant) -> bool {
    let combined_se = (median_se(&a).powi(2) + median_se(&b).powi(2)).sqrt();
    if combined_se == ZERO {
        // Zero variance in both samples: any nonzero, reproducible delta is real.
        return median_delta != ZERO;
    }
    median_delta.abs() > Z_THRESHOLD * combined_se
}

fn median_se(v: &Variant) -> f64 {
    if v.count == NO_SAMPLES {
        return ZERO;
    }
    MEDIAN_SE_FACTOR * v.stddev / (v.count as f64).sqrt()
}
