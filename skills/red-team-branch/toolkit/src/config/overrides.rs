use super::Ctx;

impl Ctx {
    pub fn apply_overrides(
        &mut self,
        base_url: Option<String>,
        auth: &[String],
        profiles: &[String],
    ) -> anyhow::Result<()> {
        if let Some(base) = base_url {
            self.base_url = Some(base);
        }
        for header in auth {
            let (key, value) = split_header("--auth", header)?;
            self.http.headers.insert(key, value);
        }
        for profile in profiles {
            let (name, header) = profile
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("--profile must be 'name: Header: value'"))?;
            let (key, value) = split_header("--profile", header)?;
            self.profiles
                .entry(name.trim().to_string())
                .or_default()
                .insert(key, value);
        }
        Ok(())
    }
}

fn split_header(flag: &str, raw: &str) -> anyhow::Result<(String, String)> {
    let (key, value) = raw
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("{flag} must be 'Header: value', got '{raw}'"))?;
    Ok((key.trim().to_string(), value.trim().to_string()))
}
