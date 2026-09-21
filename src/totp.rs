use base32::Alphabet;
use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

const TIME_STEP: u64 = 30;
const CODE_MODULUS: u32 = 1_000_000;

type HmacSha1 = Hmac<Sha1>;

pub fn decode_secret(secret: &str) -> Result<Vec<u8>, String> {
    let normalized: String = secret
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '=')
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if normalized.is_empty() {
        return Err("Base32 password is not null".to_string());
    }

    base32::decode(Alphabet::Rfc4648 { padding: false }, &normalized)
        .ok_or_else(|| " Base32 password is ilvailed".to_string())
}

pub fn generate(secret: &[u8], timestamp: u64) -> u32 {
    let counter = timestamp / TIME_STEP;

    let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC-SHA1 not support this password");

    mac.update(&counter.to_be_bytes());

    let code_bytes = mac.finalize().into_bytes();
    let offset = (code_bytes[19] & 0x0f) as usize;

    let binary_code = ((code_bytes[offset] as u32 & 0x7f) << 24)
        | ((code_bytes[offset + 1] as u32) << 16)
        | ((code_bytes[offset + 2] as u32) << 8)
        | code_bytes[offset + 3] as u32;

    binary_code % CODE_MODULUS
}

pub fn remaining_seconds(timestamp: u64) -> u64 {
    TIME_STEP - timestamp % TIME_STEP
}
