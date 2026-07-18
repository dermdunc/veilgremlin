# Output: T08

Engine: claude-cli
Model: opus
Date: 2026-07-17
Permission mode: acceptEdits

---

Done. Here's the session closeout.

## Summary

Implemented `vg_core::Parser` across `crates/vg-parsers/` (was an empty stub). Eight format modules, a shared util, a registry, and a cross-crate integration test.

### Changed / created files
- **`crates/vg-parsers/Cargo.toml`** — added `regex`, `serde_yaml`, `toml`, `tree-sitter`, `tree-sitter-rust`; dev-dep `vg-detectors`.
- **`src/{lib,util,json,yaml,toml,csv,log,diff,env,rust}.rs`** — new. `lib.rs` exposes `all_parsers()` (registry, mirrors `all_detectors()`).
- **`tests/detector_integration.rs`** — new, cross-crate.
- **Docs**: `docs/decisions.md` (T08 entry), `docs/session-log.md`, `docs/next-actions.md`, `docs/risks.md` + `.hekton/risk-register.yaml` (RISK-0011), `docs/build-log/2026-07-17-parsers-that-refuse-to-panic.md` (+README link).

### What / why
- **Never-panic is the design driver.** JSON is a hand-rolled tolerant byte tokenizer (serde_json gives no offsets and aborts on first error). YAML/TOML use `serde_yaml`/`toml` as a well-formedness gate but derive spans from quote-aware line scanners (those libs discard byte offsets); valid flow-style YAML delegates to the JSON tokenizer. CSV/log/diff/env are tolerant scanners. Rust source uses tree-sitter (error-tolerant by construction). Every span routes through `util::span`, which clamps to buffer length — so the span-bounds invariant holds even on adversarial input. Each module carries an `assert_parser_never_panics` battery (empty, truncated UTF-8, unbalanced delimiters, all-NUL, every-byte, binary-as-format), plus a registry-wide battery.

### Mandated cross-crate classification
`detector_integration.rs` feeds real parser `Span`s into `vg_detectors::all_detectors()` and pins that the detectors' `_spans` parameter is a **no-op** (real spans, empty, and bogus spans all yield identical findings). Classified in `docs/decisions.md` as an **expected, stage-appropriate gap**, not a defect: whole-buffer scanning is a superset of structure-scoped scanning; the span-threading pipeline is T07 (Wave C); and the no-op is now test-pinned so it can't silently change. Flagged span-awareness as the natural T07 fix for the 2026-07-16 entropy/phone false positives.

### Assumptions (no follow-up channel)
Rust as the tree-sitter language; hand-rolled JSON; serde_yaml/toml as gates + line-scan spans; `.env` inline `#` is not a comment. All recorded in `docs/decisions.md`.

### Validation status
**Not compiled/tested in-session** — `cargo`/`rustc`/`python` are approval-gated with no human in this one-shot dispatch (same constraint as T04). Written correct-by-inspection.

### Risks / required review actions (RISK-0011)
1. **Top item — regenerate `Cargo.lock`:** 4 new deps make the lockfile stale; **every `--locked` CI job (build/test/clippy/bench) fails** until a maintainer runs `cargo build` and commits it.
2. Confirm the `tree-sitter = "0.22"` / `tree-sitter-rust = "0.21"` pair resolves (chosen for the `set_language(&Language)`/`language()` API in `rust.rs`); adjust pins, not the call site, if not.
3. Run `cargo test -p vg-parsers && cargo clippy --all-targets -- -D warnings && cargo fmt --check`; the fmt pass will absorb any residual hand-formatting.

### Next actions
Verify per RISK-0011; after landing, re-run the census ladder's parser+detector rung; T07 to thread spans into detectors.

**Mind-palace updated:** no (vault mutation not authorised; repo-local docs updated).
