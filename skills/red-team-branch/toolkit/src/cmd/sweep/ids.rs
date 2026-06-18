use super::args::Args;

pub fn collect(args: &Args) -> anyhow::Result<Vec<String>> {
    if let Some(list) = &args.ids {
        return Ok(split_ids(list));
    }
    if let Some(path) = &args.ids_file {
        return Ok(split_lines(&std::fs::read_to_string(path)?));
    }
    if let Some(range) = &args.range {
        return parse_range(range);
    }
    Ok(Vec::new())
}

fn split_ids(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn split_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_range(range: &str) -> anyhow::Result<Vec<String>> {
    let Some((lo, hi)) = range.split_once('-') else {
        anyhow::bail!("--range must look like '1-1000'");
    };
    let (lo, hi) = (lo.trim().parse::<u64>()?, hi.trim().parse::<u64>()?);
    Ok((lo.min(hi)..=lo.max(hi)).map(|n| n.to_string()).collect())
}
