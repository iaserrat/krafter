use super::model::Injector;

impl Injector {
    pub fn channel_name(&self) -> &'static str {
        match self {
            Self::Url { .. } => "url",
            Self::Header { .. } => "header",
            Self::Body { .. } => "body",
            Self::Multipart { .. } => "multipart",
        }
    }
}
