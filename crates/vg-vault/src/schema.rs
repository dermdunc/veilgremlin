//! SQLCipher schema for the vault.
//!
//! The whole database file is AES-256 encrypted by SQLCipher (`PRAGMA key`, set in
//! `lib.rs` before any of this runs), so a value stored in a column is encrypted at rest by
//! that whole-DB encryption — this crate does not add a second, app-level cipher on top of
//! it. `interface-contracts.md` §5's "AES-256 at rest (SQLCipher)" is satisfied by the
//! SQLCipher layer itself.

/// Idempotent DDL run on every open. `IF NOT EXISTS` throughout so re-opening an existing
/// vault is a no-op; there is only one schema version in Phase 1.
pub(crate) const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
    k TEXT PRIMARY KEY,
    v BLOB NOT NULL
);

-- One row per interned (value, entity type, namespace). `key_hex` is the salted HMAC
-- placeholder key (vg_core::keying::placeholder_key, hex-encoded) and is the stable lookup
-- key for `intern`'s "already interned?" check. `value` is the raw secret, protected by the
-- surrounding SQLCipher encryption. `ordinal`/`display` are the human-readable placeholder
-- (EMAIL_001) minted by the Keyer.
CREATE TABLE IF NOT EXISTS mapping (
    key_hex       TEXT PRIMARY KEY,
    mapping_ref   TEXT NOT NULL UNIQUE,
    display       TEXT NOT NULL,
    ordinal       INTEGER NOT NULL,
    ns_kind       TEXT NOT NULL,
    ns_id         TEXT NOT NULL,
    entity_kind   TEXT NOT NULL,
    entity_custom TEXT,
    value         TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    expires_at    INTEGER
);

-- Defends the per-(namespace, entity type) ordinal sequence: two different values must
-- never land on the same display ordinal within one namespace/type, even if a second writer
-- raced the in-memory counter.
--
-- `COALESCE(entity_custom, '')`, not the bare column, is deliberate (Codex critique,
-- 2026-07-17): SQLite treats NULL as DISTINCT in a UNIQUE index, and every fixed entity type
-- (Email, Iban, ...) stores entity_custom = NULL. With the bare column the guard therefore
-- did NOT fire for any fixed type — two racing writers could both insert EMAIL_001 for
-- different secrets in the same namespace, exactly the collision this index exists to stop.
-- COALESCEing NULL to '' makes all fixed-type rows share one key value, so the UNIQUE
-- constraint applies uniformly. (Custom(name) rows already had a non-null value and were
-- always covered.)
CREATE UNIQUE INDEX IF NOT EXISTS idx_mapping_ordinal
    ON mapping (ns_kind, ns_id, entity_kind, COALESCE(entity_custom, ''), ordinal);

-- `resolve`/`purge_expired` lookups.
CREATE INDEX IF NOT EXISTS idx_mapping_ref ON mapping (mapping_ref);
CREATE INDEX IF NOT EXISTS idx_mapping_expiry ON mapping (expires_at);

-- Append-only demask log: one row per `resolve` attempt (success or not), so a reversal is
-- always attributable. Holds only the opaque mapping_ref and namespace — never the value.
CREATE TABLE IF NOT EXISTS demask_event (
    id           TEXT PRIMARY KEY,
    mapping_ref  TEXT NOT NULL,
    ns_kind      TEXT NOT NULL,
    ns_id        TEXT NOT NULL,
    requested_at INTEGER NOT NULL,
    success      INTEGER NOT NULL
);
"#;
