use crate::error::{AppError, AppResult};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

// 32 symbols without I, L, O and U: 10 characters carry 50 bits
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
pub const CODE_LEN: usize = 10;
const TOKEN_BYTES: usize = 32;

fn random_bytes<const N: usize>() -> AppResult<[u8; N]> {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|_| AppError::Sync("the system random source failed".to_string()))?;
    Ok(bytes)
}

pub fn pairing_code() -> AppResult<String> {
    let bytes = random_bytes::<CODE_LEN>()?;
    Ok(bytes
        .iter()
        .map(|b| char::from(ALPHABET[usize::from(b & 31)]))
        .collect())
}

/// A fresh device credential, shown to the phone once.
pub fn device_token() -> AppResult<String> {
    Ok(hex(&random_bytes::<TOKEN_BYTES>()?))
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(Sha256::digest(bytes).as_ref())
}

/// Compares two secrets without stopping at the first different byte.
pub fn secrets_equal(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_the_alphabet_and_differ() {
        let first = pairing_code().expect("code");
        let second = pairing_code().expect("code");
        assert_eq!(first.len(), CODE_LEN);
        assert!(first.bytes().all(|b| ALPHABET.contains(&b)));
        assert_ne!(first, second);
    }

    #[test]
    fn tokens_are_long_and_hash_stably() {
        let token = device_token().expect("token");
        assert_eq!(token.len(), TOKEN_BYTES * 2);
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn secret_comparison_is_exact() {
        assert!(secrets_equal("abc", "abc"));
        assert!(!secrets_equal("abc", "abd"));
        assert!(!secrets_equal("abc", "ab"));
    }
}
