# Output: T04

Engine: claude-cli
Model: Claude Code default
Date: 2026-07-17
Permission mode: acceptEdits

---

Both `cargo`/`rustc` execution and mutating `git` commands (`git add`, presumably `git commit`) require approval that isn't reachable in this headless session — only read-only commands (`git status`, `find`, `grep`, `cat`) went through. I've implemented everything in the working tree but can't compile-check or commit it myself. Here's the summary:

## What was built

**`crates/vg-core/src/keying.rs`** (new, ~520 lines incl. 25 unit tests) — implements the `VaultStore`-authoritative formula "stable placeholder via salted HMAC over `(canonical(value), ty, ns)`":
- `canonicalize(value, ty)` — whitespace trim/collapse always; case-folding only for types where case isn't identity-bearing (`Email`, `Hostname`, `InternalIp`, `Iban`, `SortCode`, `Postcode`, `TraceId`) — secrets/passwords/free-text keep case.
- `placeholder_key(salt, value, ty, ns) -> PlaceholderKey` — HMAC-SHA256, with an explicit separator byte between message fields to prevent concatenation collisions. Salt is caller-supplied (not hardcoded), since `vg-core` doesn't own persistent key storage.
- `Keyer` — `Mutex`-backed session cache assigning sequential per-`(Namespace, EntityType)` ordinals, producing `EMAIL_001`/`ACCOUNT_ID_014`-style `display` strings.
- `luhn_is_valid` / `iban_mod97_is_valid` — pure checksum validators, hand-verified digit-by-digit against known test vectors.

**`crates/vg-core/tests/keying_integration.rs`** (new) — feeds real `Finding`s from `vg-detectors::all_detectors()` through the keying logic, per the T04 acceptance criterion added 2026-07-16. Required a dev-only `vg-detectors` dependency on `vg-core` (not a real cycle — documented in the Cargo.toml comment).

**Docs updated:** `docs/decisions.md` (5 judgment calls recorded, notably: Luhn/mod-97 exposed as pure validators only, *not* wired into placeholder display, since synthesizing a fake-but-checksum-valid card number would contradict the already-frozen ADR-005 "typed placeholders, not synthetic values"), `docs/session-log.md`, `docs/next-actions.md`, and a `docs/build-log/` entry.

## What's unverified

I could not run `cargo build`/`test`/`clippy` — every toolchain invocation was blocked pending approval unavailable in this headless run. The code is hand-traced carefully but **not compiled**. I also couldn't `git add`/`commit` for the same reason, so **all changes are sitting uncommitted in the working tree** on branch `gateway/run-20260717-T04`.

Before this can be merged, someone needs to run, from this worktree:
```
cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo test -p vg-core
```
and then commit/push/open the PR — I wasn't able to do any of that myself this session.
