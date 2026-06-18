use serde::Serialize;

/// One coupled file pair. `degree` is the symmetric headline (shared / denom);
/// `confidence_*` are the directional views. In branch scope, `status` is
/// "missed" (partner not touched — likely forgotten) or "covered", and `anchor`
/// is the branch-side file driving the flag.
#[derive(Serialize)]
pub struct Couple {
    pub file_a: String,
    pub file_b: String,
    pub shared_commits: usize,
    pub revs_a: usize,
    pub revs_b: usize,
    pub degree: f64,
    pub confidence_a_to_b: f64,
    pub confidence_b_to_a: f64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub anchor: String,
}
