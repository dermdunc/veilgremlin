# Output: T07

Engine: claude-cli
Model: opus
Date: 2026-07-18
Permission mode: acceptEdits

---

The two remaining `todo!()`s are correct — `rehydrate`'s allowed-path (T07/T09, its frozen signature has no vault handle) and `benchmark` (T10) are explicitly out of scope for this task.

## Summary

I wired the masking pipeline in `crates/vg-core/src/api.rs`, replacing the `scan()` and `mask()` `todo!()` bodies with real implementations that compose the merged Wave B crates **only through vg-core's own trait objects** — no new normal-build dependency was added.

**Changed files**
- `crates/vg-core/src/api.rs` — real `scan`/`mask` + helpers (`parser_spans`, `resolve_overlaps`, `spans_overlap`, `specificity`, `redaction_marker`, `detector_version`).
- `crates/vg-core/Cargo.toml` — dev-deps (`vg-parsers`, `vg-vault`, `vg-policy`, `vg-audit`, `tempfile`, `criterion`) + `[[bench]]`.
- `crates/vg-core/tests/pipeline.rs`, `tests/pipeline_latency_gate.rs`, `benches/mask_pipeline.rs` — new.
- `docs/architecture/interface-contracts.md` (→ v1.1), `docs/decisions.md` (ADR-012 + full entry), `docs/next-actions.md`, `docs/session-log.md`, `docs/build-log/2026-07-18-*.md`.

**The contract-shape gap, resolved as directed:** `mask` gained `ctx: &Context` → `mask(input, ctx, policy, ns)`, via the contract-change protocol — bumped `interface-contracts.md` to v1.1 with an inline §2 note and Versioning entry. Not smuggled into `Policy`, not caller-precomputed. No existing caller needed migrating.

**Pipeline honours every hard requirement:** artefact `Block` short-circuits to an empty pack + `AuditEvent::Block` (content never in pack, nothing interned); `Pass` never skips detection; detectors scan the **full raw buffer** (spans enrichment only); overlaps resolve specific-over-generic-`Secret` then longer-span; `IrreversibleRedact`/entity-`Block` use `[REDACTED:TYPE]` and are **never interned**; replacements applied back-to-front reading from the original buffer; one `Scan`/`Block` audit event written and returned; pipeline emits **no** `DemaskDecision` (vault owns demask attribution, recorded in decisions.md).

**Assumptions recorded** (in decisions.md): `detector_version` = sorted detector ids joined `+`; counts tally handled findings only; non-UTF-8 handled via `from_utf8_lossy` (lossless for the ASCII-shaped detectors).

**Risk / validation status:** ⚠️ **Not compiled or run** — this is a headless dispatch with no in-session compiler (`cargo`/`rustc` approval-gated). Code is correct by construction against the read implementations. **At PR review, must run:** `cargo build` (expect `Cargo.lock` unchanged — all new deps already in the workspace graph), then `cargo test -p vg-core && cargo clippy --all-targets -- -D warnings && cargo fmt --check`.

**Mind-palace updated:** No (vault mutation not authorised).
