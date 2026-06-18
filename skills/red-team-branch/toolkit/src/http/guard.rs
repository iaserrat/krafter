use url::Url;

pub fn guard_target(
    target: &str,
    allow_remote: bool,
    allow_hosts: &[String],
) -> anyhow::Result<()> {
    let url = Url::parse(target).map_err(|e| anyhow::anyhow!("bad url '{target}': {e}"))?;
    let host = url.host_str().unwrap_or("");
    if is_local(host) || allow_hosts.iter().any(|h| h == host) {
        return Ok(());
    }
    if allow_remote {
        eprintln!("[rtk][WARN] targeting non-local host '{host}' (--allow-remote)");
        return Ok(());
    }
    anyhow::bail!(
        "refusing non-local target '{host}'. Add safety.allow_hosts or pass --allow-remote with authorization."
    )
}

fn is_local(host: &str) -> bool {
    let bare = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host);
    match bare.parse::<std::net::IpAddr>() {
        Ok(ip) => ip.is_loopback() || ip.is_unspecified(),
        Err(_) => host.eq_ignore_ascii_case("localhost"),
    }
}

#[cfg(test)]
mod tests;
