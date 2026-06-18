pub(super) struct StatusRange {
    lo: u16,
    hi: u16,
}

impl StatusRange {
    pub(super) fn contains(&self, status: u16) -> bool {
        (self.lo..=self.hi).contains(&status)
    }
}

pub(super) fn parse(s: &str) -> anyhow::Result<StatusRange> {
    let (lo, hi) = s
        .split_once('-')
        .ok_or_else(|| anyhow::anyhow!("range must be 'lo-hi'"))?;
    Ok(StatusRange {
        lo: lo.trim().parse()?,
        hi: hi.trim().parse()?,
    })
}
