use super::{BatchResult, ChildCase, Session};
use crate::http;
use futures::{stream, StreamExt};

const SEND_SNIPPET_LEN: usize = 200;

impl Session {
    pub async fn execute_seed(&mut self, idx: usize) -> Vec<BatchResult> {
        let batch = self.children(idx);
        let inj = self.inj.clone();
        let client = self.client.clone();
        let mut results = stream::iter(batch.into_iter().map(move |case| {
            let client = client.clone();
            let inj = inj.clone();
            async move { send_case(&client, &inj, case).await }
        }))
        .buffer_unordered(self.conc)
        .collect::<Vec<_>>()
        .await;
        self.exec += results.len() as u64;
        results.sort_by_key(|result| result.order);
        results
    }
}

async fn send_case(
    client: &reqwest::Client,
    inj: &crate::cmd::fuzz::injector::Injector,
    case: ChildCase,
) -> BatchResult {
    BatchResult {
        order: case.order,
        payload: case.payload,
        outcome: send_guarded(client, inj, case.spec).await,
    }
}

async fn send_guarded(
    client: &reqwest::Client,
    inj: &crate::cmd::fuzz::injector::Injector,
    spec: http::RequestSpec,
) -> http::Outcome {
    if inj.guard_rendered(&spec.url) {
        http::send_once(client, &spec, SEND_SNIPPET_LEN).await
    } else {
        http::Outcome {
            error: Some("guard: rendered host not local".into()),
            ..Default::default()
        }
    }
}
