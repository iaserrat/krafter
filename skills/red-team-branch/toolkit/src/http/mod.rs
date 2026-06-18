mod client;
mod guard;
mod request;
mod response;
mod send;
mod status;
mod url;

pub use client::{build_client, HttpOpts};
pub use guard::guard_target;
pub use request::{pct_bytes, RequestBody, RequestHeader, RequestSpec};
pub use response::{find_header, header_str, Outcome, NO_SNIPPET_LEN, RAW_CAP};
pub use send::send_once;
pub use status::{is_denied, is_success, reached_operation, status_class, NOT_FOUND};
pub use url::resolve_url;

#[cfg(test)]
mod tests;
