#[derive(serde::Serialize)]
pub struct Stats {
    pub count: usize,
    pub min: u64,
    pub max: u64,
    pub mean_ms: f64,
    pub median_ms: f64,
    pub p90_ms: f64,
    pub stddev_ms: f64,
}

const P90_PERCENTILE: f64 = 0.9;
const MEDIAN_PERCENTILE: f64 = 0.5;
const ROUND_2_SCALE: f64 = 100.0;

pub fn stats(samples: &[u64]) -> Option<Stats> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    Some(Stats {
        count: sorted.len(),
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        mean_ms: round2(mean(&sorted)),
        median_ms: percentile(&sorted, MEDIAN_PERCENTILE),
        p90_ms: percentile(&sorted, P90_PERCENTILE),
        stddev_ms: round2(stddev(&sorted)),
    })
}

fn mean(samples: &[u64]) -> f64 {
    samples.iter().sum::<u64>() as f64 / samples.len() as f64
}

fn stddev(samples: &[u64]) -> f64 {
    let mean = mean(samples);
    let var = samples
        .iter()
        .map(|&x| (x as f64 - mean).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    var.sqrt()
}

fn percentile(samples: &[u64], p: f64) -> f64 {
    let idx = ((samples.len() as f64 * p) as usize).min(samples.len() - 1);
    samples[idx] as f64
}

fn round2(x: f64) -> f64 {
    (x * ROUND_2_SCALE).round() / ROUND_2_SCALE
}
