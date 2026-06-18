use super::injector::Injector;
use crate::{cmd::cov, http};

pub struct Probe<'a> {
    pub client: &'a reqwest::Client,
    pub inj: &'a Injector,
    pub base: &'a http::Outcome,
    pub canary: &'a str,
    pub lat_lower: f64,
    pub max_exec: u64,
    pub exec: &'a mut u64,
}

impl Probe<'_> {
    pub fn exhausted(&self) -> bool {
        *self.exec >= self.max_exec
    }

    pub async fn send(&mut self, payload: &[u8]) -> Option<http::Outcome> {
        if self.exhausted() {
            return None;
        }
        let spec = self.inj.spec(payload);
        if !self.inj.guard_rendered(&spec.url) {
            return None;
        }
        *self.exec += 1;
        Some(http::send_once(self.client, &spec, crate::http::NO_SNIPPET_LEN).await)
    }

    pub fn bucket(&self, o: &http::Outcome, payload: &[u8]) -> u64 {
        cov::bucket(o, self.base, self.canary, payload, self.lat_lower)
    }
}
