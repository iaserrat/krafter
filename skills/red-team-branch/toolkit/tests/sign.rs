#[test]
fn sign_matches_known_hmac_vector() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_rtk"))
        .args([
            "sign",
            "--secret",
            "key",
            "--payload",
            "The quick brown fox jumps over the lazy dog",
        ])
        .output()
        .expect("run rtk sign");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        v["signature"].as_str().unwrap(),
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );
}
