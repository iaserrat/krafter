use super::Args;
use std::io::Read;

pub fn read(args: &Args) -> anyhow::Result<String> {
    if let Some(payload) = &args.payload {
        return Ok(payload.clone());
    }
    if let Some(file) = &args.payload_file {
        return Ok(std::fs::read_to_string(file)?);
    }
    let mut payload = String::new();
    std::io::stdin().read_to_string(&mut payload)?;
    Ok(payload)
}
