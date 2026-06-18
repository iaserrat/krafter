pub struct Hit {
    pub id: String,
    pub class: &'static str,
    pub a_status: u16,
    pub anon_status: u16,
    pub b_status: Option<u16>,
    pub body_len: usize,
    pub sha8: String,
    pub keys: Vec<String>,
    pub sensitive: Vec<String>,
    pub snippet: String,
    pub blocked: Option<String>,
}
