use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::Sha256;

pub fn sign(algo: &str, key: &[u8], msg: &[u8]) -> anyhow::Result<Vec<u8>> {
    Ok(match algo {
        "sha256" => hmac_sha256(key, msg),
        "sha1" => hmac_sha1(key, msg),
        other => anyhow::bail!("unsupported algo '{other}'"),
    })
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

fn hmac_sha1(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}
