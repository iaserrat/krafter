pub const TITLE_SNIPPET_LIMIT: usize = 120;
pub const ELLIPSIS: char = '…';

pub fn truncate(s: &str, n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push(ELLIPSIS);
    }
    out
}

pub fn extract_title(body: &str) -> Option<String> {
    let lower = body.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let gt = lower[start..].find('>')? + start + 1;
    let end = lower[gt..].find("</title>")? + gt;
    let title = body[gt..end].trim();
    (!title.is_empty()).then(|| truncate(title, TITLE_SNIPPET_LIMIT))
}
