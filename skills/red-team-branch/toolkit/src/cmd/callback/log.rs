use std::io::Write;

pub struct HitLog {
    file: Option<std::fs::File>,
}

impl HitLog {
    pub fn open(path: Option<&std::path::PathBuf>) -> anyhow::Result<Self> {
        let file = path
            .map(|p| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .map_err(|e| anyhow::anyhow!("open log file {}: {e}", p.display()))
            })
            .transpose()?;
        Ok(Self { file })
    }

    pub fn write(&mut self, line: &str) {
        if let Some(file) = self.file.as_mut() {
            if let Err(e) = writeln!(file, "{line}") {
                eprintln!("[rtk][warn] log write failed: {e}");
            }
        }
    }
}
