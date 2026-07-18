# Output: T05b

Engine: claude-cli
Model: Claude Code default
Date: 2026-07-17
Permission mode: acceptEdits

---

T05b is implemented and left as an uncommitted working-tree diff on `gateway/run-20260717-T05b` — `git add`/`commit` are permission-blocked in this dispatch (same as every toolchain command), so the tollgate applies the diff, matching the T04 flow.

## Session closeout

**What was built:** `vg_core::traits::AuditSink` implemented in the previously empty `vg-audit` crate as `JsonlAuditSink` — an append-only JSON Lines file, one fsynced record per line, opened `O_APPEND` and never truncated, with an in-memory index replayed at open for `get`.

**Changed files**
- `crates/vg-audit/src/lib.rs` — the sink: open/replay, torn-write recovery (skip + count + heal-newline), hard refusal on unknown `schema_version`, fsynced `write`, `get` from the index. `OpenError` is crate-local since `open` isn't part of the frozen trait.
- `crates/vg-audit/src/record.rs` (new) — versioned storage schema: `RecordV1`/`EventV1` plus mirrors of every contract enum, explicit conversions both ways (fallible toward storage for every `#[non_exhaustive]` contract enum so a future variant fails loudly instead of being silently dropped), the v1 wire shape pinned by test, and `Destination` serialized as the stable `DestinationId` strings.
- `crates/vg-audit/tests/sink.rs` (new) — conformance roundtrips for all six `AuditEvent` variants, reopen durability, append-not-rewrite, torn-write recovery, unknown-schema-version refusal, 8-thread concurrency, and the acceptance property test: adversarial raw values (newlines, tabs, quotes, IBAN, API key, unicode) must not appear in the persisted bytes verbatim **or** JSON-escaped, plus a negative control proving the conformance helper catches a deliberately leaky event.
- `crates/vg-audit/Cargo.toml`, `Cargo.lock` — deps added; lockfile **hand-edited** (see below).
- `docs/decisions.md`, `docs/session-log.md`, `docs/next-actions.md`, `docs/build-log/2026-07-17-an-audit-log-sized-to-fit-its-lockfile.md`, mind-palace mirror session-log.

**Key decisions** (full record in `docs/decisions.md`, 2026-07-17 T05b entry)
- **JSONL over SQLite.** Append-only maps directly onto append+fsync, and — the deciding factor — this dispatch had no runnable `cargo` (every toolchain command permission-blocked, the same factory gap T04 hit; now a pattern). CI builds `--locked`, so the lockfile had to be updated by hand, which is only safe as a dependency-edge edit between already-locked packages. serde/serde_json/thiserror/uuid are already locked; rusqlite isn't.
- That same constraint pruned `tempfile` (replaced with a std-only tempdir in tests) and uuid's `serde` feature (replaced by a `#[serde(with)]` Display-based adapter, identical wire bytes). `Cargo.toml` carries a comment so nobody "cleans this up" into a broken build.
- Storage schema is a deliberate mirror with `schema_version` on every record, not serde derives on frozen `vg-core` types.

**Risks / validation status — the honest part:** nothing was compiled or tested in this session; it was impossible, not skipped by choice. The line-by-line self-review caught three genuine would-be compile errors before handoff (`PathBuf` has no `Display` in a thiserror format string; an exhaustive match on non-exhaustive `Destination`; a moved-while-borrowed `path`), but the reviewer must run `cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo test -p vg-audit` before merge — recorded in `docs/next-actions.md` as a blocking item. Residual risk is concentrated in hand-guessed rustfmt canonicality and the hand-edited lockfile.

**Next actions:** reviewer runs the verify chain; T07 consumes this sink in `mask()` pipeline assembly; serial-vs-concurrent for remaining Wave B (T05/T06/T08) still open.

**Mind-palace updated:** yes (repo-local mirror only; vault untouched, `vault_mutation_allowed: false`).
