//! OS-keychain wrap for the SQLCipher database encryption key.
//!
//! The 32-byte DB key is never written to disk in plaintext (interface-contracts.md §5:
//! "DB key wrapped by OS keychain, never persisted plaintext"). It lives in the OS secret
//! store — the macOS Keychain (via the `keyring` crate's Security-framework backend on
//! `target_os = "macos"`, the platform this lab targets first). On first open a fresh
//! random key is generated and stored; subsequent opens retrieve it.
//!
//! The key is stored hex-encoded because the keychain APIs traffic in UTF-8 strings, not
//! arbitrary bytes.

use keyring::{Entry, Error as KeyringError};

use crate::error::{crypto_err, VaultError};
use crate::random::fill_random;

/// Returns the DB encryption key for `(service, account)`, generating and storing a fresh
/// random 32-byte key in the OS keychain the first time (when no entry exists yet).
pub(crate) fn load_or_create_db_key(service: &str, account: &str) -> Result<[u8; 32], VaultError> {
    let entry = Entry::new(service, account)
        .map_err(|e| crypto_err(format!("keychain entry init failed: {e}")))?;

    match entry.get_password() {
        Ok(hex) => decode_key(&hex),
        Err(KeyringError::NoEntry) => {
            let mut key = [0u8; 32];
            fill_random(&mut key)?;
            entry
                .set_password(&encode_key(&key))
                .map_err(|e| crypto_err(format!("keychain store failed: {e}")))?;
            Ok(key)
        }
        Err(e) => Err(crypto_err(format!("keychain read failed: {e}"))),
    }
}

fn encode_key(key: &[u8; 32]) -> String {
    key.iter().map(|b| format!("{b:02x}")).collect()
}

fn decode_key(hex: &str) -> Result<[u8; 32], VaultError> {
    let bytes =
        decode_hex(hex).ok_or_else(|| crypto_err("stored DB key is not valid hex".to_string()))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| crypto_err("stored DB key is not 32 bytes".to_string()))?;
    Ok(arr)
}

/// Decodes an even-length lowercase/uppercase hex string, or `None` on any non-hex byte.
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_hex_round_trips() {
        let key = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98,
            0x76, 0x54, 0x32, 0x10,
        ];
        let hex = encode_key(&key);
        assert_eq!(hex.len(), 64);
        assert_eq!(decode_key(&hex).unwrap(), key);
    }

    #[test]
    fn decode_key_rejects_wrong_length() {
        assert!(decode_key("abcd").is_err());
    }

    #[test]
    fn decode_key_rejects_non_hex() {
        assert!(decode_key(&"zz".repeat(32)).is_err());
    }
}
