//! Cryptographically-secure random bytes for the DB key and the keying salt.
//!
//! Uses the OS CSPRNG via `getrandom` (the same source `uuid`'s v4 generation draws on),
//! so there is no seeded/deterministic path here — both the encryption key and the salt
//! must be unpredictable per install.

use crate::error::{io_err, VaultError};

/// Fills `buf` with cryptographically-secure random bytes, or returns `VaultError::Io` if
/// the OS entropy source is unavailable.
pub(crate) fn fill_random(buf: &mut [u8]) -> Result<(), VaultError> {
    getrandom::getrandom(buf).map_err(|e| io_err(format!("OS RNG unavailable: {e}")))
}
