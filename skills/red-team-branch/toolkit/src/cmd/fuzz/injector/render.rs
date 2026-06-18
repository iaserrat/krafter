use super::model::Injector;
use super::request_parts::{raw_header, text_body, text_headers};
use crate::http::{self, RequestBody, RequestSpec};

impl Injector {
    pub fn spec(&self, payload: &[u8]) -> RequestSpec {
        match self {
            Self::Url { method, prefix, suffix, headers, body, .. } => RequestSpec {
                method: method.clone(),
                url: format!("{}{}{}", prefix, http::pct_bytes(payload), suffix),
                headers: text_headers(headers),
                body: text_body(body.clone()),
            },
            Self::Header { method, url, headers, name, prefix, suffix, body, .. } => {
                let mut value = prefix.clone();
                value.extend_from_slice(payload);
                value.extend_from_slice(suffix);
                RequestSpec {
                    method: method.clone(), url: url.clone(),
                    headers: raw_header(headers, name, value),
                    body: text_body(body.clone()),
                }
            }
            Self::Body { method, url, headers, prefix, suffix, .. } => {
                let mut body = prefix.clone();
                body.extend_from_slice(payload);
                body.extend_from_slice(suffix);
                RequestSpec {
                    method: method.clone(), url: url.clone(),
                    headers: text_headers(headers), body: RequestBody::Bytes(body),
                }
            }
            Self::Multipart { method, url, headers, body_prefix, body_suffix, preamble, postamble, boundary, .. } => {
                let mut body = preamble.clone();
                body.extend_from_slice(body_prefix);
                body.extend_from_slice(payload);
                body.extend_from_slice(body_suffix);
                body.extend_from_slice(postamble);
                let ct = format!("multipart/form-data; boundary={boundary}");
                let mut hdrs = text_headers(headers);
                hdrs.push(crate::http::RequestHeader::Text("Content-Type".into(), ct));
                RequestSpec { method: method.clone(), url: url.clone(), headers: hdrs, body: RequestBody::Bytes(body) }
            }
        }
    }

    pub fn guard_setup(&self) -> anyhow::Result<()> {
        match self {
            Self::Url { prefix, suffix, safety, .. } => {
                http::guard_target(&format!("{prefix}x{suffix}"), safety.allow_remote, &safety.allow_hosts)
            }
            Self::Header { url, safety, .. }
            | Self::Body { url, safety, .. }
            | Self::Multipart { url, safety, .. } => {
                http::guard_target(url, safety.allow_remote, &safety.allow_hosts)
            }
        }
    }

    pub fn guard_rendered(&self, url: &str) -> bool {
        match self {
            Self::Url { safety, .. } => http::guard_target(url, safety.allow_remote, &safety.allow_hosts).is_ok(),
            _ => true,
        }
    }
}
