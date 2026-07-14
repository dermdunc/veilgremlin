//! `vg-vault` — encrypted placeholder store implementing `vg_core::VaultStore`.
//!
//! AES-256 at rest (SQLCipher); DB key wrapped by the OS keychain, never persisted
//! plaintext; `Secret` zeroizes on drop; stable placeholders via salted HMAC over
//! `(canonical(value), ty, ns)` (`docs/architecture/interface-contracts.md` §5).
//! `IrreversibleRedact` values are never passed to `intern`.
//!
//! Scaffolded in Task T01; implementation lands in Task T05.
