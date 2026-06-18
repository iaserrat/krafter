use crate::cmd::fuzz::args::Args;

pub enum Channel {
    Url,
    Header,
    Body,
    Multipart,
}

pub fn channel(args: &Args, url: &str, headers: &[(String, String)]) -> anyhow::Result<Channel> {
    Ok(match args.channel.as_str() {
        "url" => Channel::Url,
        "header" => Channel::Header,
        "body" => Channel::Body,
        "multipart" => Channel::Multipart,
        "auto" if url.contains("{FUZZ}") => Channel::Url,
        "auto" if headers.iter().any(|(_, v)| v.contains("{FUZZ}")) => Channel::Header,
        "auto" if args.body.as_deref().is_some_and(|b| b.contains("{FUZZ}")) => Channel::Body,
        "auto" => anyhow::bail!("no {{FUZZ}} placeholder found"),
        other => anyhow::bail!("unknown --channel '{other}'"),
    })
}

pub fn authority_end(url: &str) -> usize {
    let after = url.find("://").map(|i| i + 3).unwrap_or(0);
    after
        + url[after..]
            .find(['/', '?', '#'])
            .unwrap_or(url.len() - after)
}
