//! Mapping from this crate's failure modes onto `vg_core::VaultError`.
//!
//! The trait's frozen error type has three arms: `NotFound` (used by `resolve` for both an
//! unknown mapping *and* a namespace mismatch — the two are indistinguishable to a caller
//! by contract), `Crypto` (key wrap/unwrap, SQLCipher keying), and `Io` (SQL/filesystem).
//! Every fallible internal step maps onto one of these.

pub(crate) use vg_core::VaultError;

/// A crypto/keying failure (keychain access, SQLCipher `PRAGMA key`, malformed stored key).
pub(crate) fn crypto_err(msg: impl Into<String>) -> VaultError {
    VaultError::Crypto(msg.into())
}

/// An I/O / storage failure (SQL execution, filesystem, RNG).
pub(crate) fn io_err(msg: impl Into<String>) -> VaultError {
    VaultError::Io(msg.into())
}

/// Maps a `rusqlite` error onto `VaultError::Io`. SQLCipher key failures surface as
/// `NotADatabase`/`SqliteFailure` on the first query after a bad `PRAGMA key`; those are
/// handled explicitly at open time (see `lib.rs`) and mapped to `Crypto` there, so a plain
/// query error here is genuinely I/O-shaped.
pub(crate) fn sql_err(e: rusqlite::Error) -> VaultError {
    io_err(format!("sqlite error: {e}"))
}
