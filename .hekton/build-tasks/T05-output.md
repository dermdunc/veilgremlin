# Output: T05

Engine: claude-cli
Model: opus
Date: 2026-07-17
Permission mode: acceptEdits

---

I've implemented `vg_core::traits::VaultStore` in `crates/vg-vault`. Here's the summary.

## What was built

**`crates/vg-vault/` (SQLCipher-backed `VaultStore`):**
- `lib.rs` — `Vault` + `VaultConfig`; the `intern`/`resolve`/`purge_expired` impl; `Vault::open` (Keychain-wrapped key) and `Vault::open_with_key` (caller-supplied key, for tests); a compile-time `Send + Sync` assertion (Connection is `!Sync`, so it's behind a `Mutex`).
- `schema.rs` — SQLCipher DDL: `meta` (per-install salt), `mapping` (with a UNIQUE index guarding the per-namespace/type ordinal), `demask_event`.
- `keychain.rs` — OS-keychain wrap of the 32-byte DB key via `keyring` (generate-on-first-use, hex-encoded, never plaintext on disk).
- `codec.rs` — round-trippable `Namespace`/`EntityType` ↔ column encoding (so the reseed can reconstruct real keys).
- `random.rs`, `error.rs` — CSPRNG helper + `VaultError` mapping.
- `tests/vault.rs` — conformance (`assert_vault_roundtrip`), stable placeholders, ordinal continuity across reopen, namespace isolation, demask logging, TTL purge, wrong-key failure.

**`crates/vg-core/src/keying.rs`:** added `Keyer::seed_ordinal` — the one cross-crate change, needed because T04's `Keyer` had no reseed hook. It's additive to an internal helper, not a change to the frozen `interface-contracts.md` surface.

## How the hard requirements are met
- **Reseed (the flagged hard requirement):** at open, the vault reads `MAX(ordinal)` per `(namespace, entity type)` from its own `mapping` rows and reseeds the fresh `Keyer` — so after a restart a new value gets `EMAIL_003`, never a colliding second `EMAIL_001`.
- **Look-up-before-mint:** `intern` uses non-mutating `placeholder_key` for the durable lookup and only calls `Keyer::key_for` (which advances the counter) once a value is confirmed new — otherwise re-interning a known value would drift the counter.
- **No reimplemented keying**, `Secret` throughout, namespace mismatch → `NotFound` (indistinguishable from missing), `prepare_cached` for the hot path.

Nine judgment calls are recorded in `docs/decisions.md` (ADR-011 + full entry); session-log, build-log, and next-actions updated.

## Important caveat
**I could not compile or run any of this.** `cargo` is gated behind interactive approval that a headless dispatch can't satisfy (the same no-compiler condition T04 hit). The code is written against the verified `vg-core` interfaces and I hand-traced it — including fixing one real bug during review (a private `use` that wasn't re-exportable). But **nothing has been built, clippy-checked, fmt-checked, or tested.** The PR-review gate is `cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo test -p vg-core -p vg-vault`, which also needs a C toolchain + perl for the vendored-OpenSSL/SQLCipher build. I'd also suggest a cross-model doubt-pass on the reseed/ordinal logic, as was done for T04.
