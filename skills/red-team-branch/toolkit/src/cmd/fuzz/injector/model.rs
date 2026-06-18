#[derive(Clone)]
pub struct Safety {
    pub allow_remote: bool,
    pub allow_hosts: Vec<String>,
}

#[derive(Clone)]
pub enum Injector {
    Url {
        method: String,
        prefix: String,
        suffix: String,
        headers: Vec<(String, String)>,
        body: Option<String>,
        safety: Safety,
    },
    Header {
        method: String,
        url: String,
        headers: Vec<(String, String)>,
        name: String,
        prefix: Vec<u8>,
        suffix: Vec<u8>,
        body: Option<String>,
        safety: Safety,
    },
    Body {
        method: String,
        url: String,
        headers: Vec<(String, String)>,
        prefix: Vec<u8>,
        suffix: Vec<u8>,
        safety: Safety,
    },
    Multipart {
        method: String,
        url: String,
        headers: Vec<(String, String)>,
        field_name: String,
        filename: String,
        body_prefix: Vec<u8>,
        body_suffix: Vec<u8>,
        preamble: Vec<u8>,
        postamble: Vec<u8>,
        boundary: String,
        safety: Safety,
    },
}
