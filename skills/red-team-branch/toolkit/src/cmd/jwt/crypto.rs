//! HMAC signing for JWT forgery. Dispatches on the HSxxx algorithm so a forged
//! header's `alg` stays consistent with the signature it carries.
use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha384, Sha512};

const HS384: &str = "HS384";
const HS512: &str = "HS512";

pub fn hmac_sign(key: &[u8], algorithm: &str, signing_input: &str) -> String {
    let msg = signing_input.as_bytes();
    let sig = match algorithm {
        HS384 => {
            let mut m = Hmac::<Sha384>::new_from_slice(key).unwrap();
            m.update(msg);
            m.finalize().into_bytes().to_vec()
        }
        HS512 => {
            let mut m = Hmac::<Sha512>::new_from_slice(key).unwrap();
            m.update(msg);
            m.finalize().into_bytes().to_vec()
        }
        _ => {
            let mut m = Hmac::<Sha256>::new_from_slice(key).unwrap();
            m.update(msg);
            m.finalize().into_bytes().to_vec()
        }
    };
    super::parse::encode_part(&sig)
}
