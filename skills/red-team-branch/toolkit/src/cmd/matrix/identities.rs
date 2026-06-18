use crate::config::Ctx;

#[derive(Clone)]
pub struct Identity {
    pub label: String,
    pub client: reqwest::Client,
}

pub fn resolve(ctx: &Ctx, profiles: Option<&str>) -> anyhow::Result<Vec<Identity>> {
    let mut labels = vec![
        ("default".to_string(), None),
        ("anon".to_string(), Some("anon".to_string())),
    ];
    if let Some(profiles) = profiles {
        labels.extend(split_profiles(profiles).map(|name| (name.clone(), Some(name))));
    }
    labels
        .into_iter()
        .map(|(label, profile)| {
            Ok(Identity {
                label,
                client: ctx.client_for(profile.as_deref())?,
            })
        })
        .collect()
}

fn split_profiles(raw: &str) -> impl Iterator<Item = String> + '_ {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}
