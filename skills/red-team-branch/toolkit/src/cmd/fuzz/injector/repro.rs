use super::curl::{body_flag, header_flags};
use super::model::Injector;
use crate::{cmd::fuzz::encoding::escape_printf, http};

impl Injector {
    pub fn repro(&self, payload: &[u8]) -> String {
        match self {
            // url channel sends method + static headers + static body too, so the
            // emitted PoC must carry them or it reproduces a different request.
            Self::Url { method, prefix, suffix, headers, body, .. } => format!(
                "curl -s -i -X {}{}{} '{}{}{}'",
                method,
                header_flags(headers),
                body_flag(body),
                prefix,
                http::pct_bytes(payload),
                suffix
            ),
            Self::Body { method, url, .. } => format!(
                "printf '{}' | curl -s -i -X {} '{}' --data-binary @-",
                escape_printf(payload),
                method,
                url
            ),
            Self::Header {
                method, url, name, ..
            } => format!(
                "curl -s -i -X {} '{}' -H '{}: <bytes hex {}>'",
                method,
                url,
                name,
                hex::encode(payload)
            ),
            // The file part is the --body template wrapped around the payload, so
            // the printf content must reproduce that wrapper, not the raw payload.
            Self::Multipart { method, url, field_name, filename, body_prefix, body_suffix, .. } => {
                let mut part = body_prefix.clone();
                part.extend_from_slice(payload);
                part.extend_from_slice(body_suffix);
                format!(
                    "printf '{}' | curl -s -i -X {} '{}' -F '{}=@-;filename={}'",
                    escape_printf(&part),
                    method,
                    url,
                    field_name,
                    filename,
                )
            }
        }
    }
}
