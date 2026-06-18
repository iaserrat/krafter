use serde_json::Value;
use std::process::Command;

pub fn write_config(port: u16) -> String {
    let path = std::env::temp_dir().join(format!("rtk_it_{port}.toml"));
    std::fs::write(
        &path,
        format!("[http]\nbase_url = \"http://127.0.0.1:{port}\"\n[http.headers]\nX-User = \"A\"\n[profiles.b]\nX-User = \"B\"\n"),
    )
    .unwrap();
    path.to_string_lossy().into_owned()
}

pub fn run_rtk(cfg: &str, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_rtk"))
        .arg("--config")
        .arg(cfg)
        .args(args)
        .output()
        .expect("run rtk");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "rtk {args:?} produced non-JSON ({e})\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}
