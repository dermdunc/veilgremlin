# Next Actions: VeilGremlin

Repo source-of-truth for the live work queue. Tasks T01–T11 are defined in
[`architecture/work-breakdown.md`](architecture/work-breakdown.md); the build method is in
[`architecture/agent-factory-plan.md`](architecture/agent-factory-plan.md). The full history of
completed work lives in [`docs/session-log.md`](session-log.md), [`docs/decisions.md`](decisions.md),
and [`docs/build-log/`](build-log/README.md). This file is the forward queue only, not a second log.

## Build status

T01–T11 complete; interface contract v1.4; 221 tests pass. **T11 human sign-off returned NO-GO
(2026-07-19)** — the hook adapter is a validated proof-of-mechanism but does NOT ship: it does
not deliver "invisible governance / PII never leaves the machine" without an egress proxy, and
the keychain UX is poor. See the 2026-07-19 T11 sign-off entry in `docs/decisions.md`. The mask/
demask logic, vault, detectors, pipeline, and tool-path masking are all validated.

## Now — the next milestone (supersedes the prior sign-off blocker order)

- [ ] **Local masking proxy + daemon.** Intercept the actual request to the model endpoint,
      mask the entire assembled payload (prompt + context) via the vault, demask the response —
      invisible to the user; a long-lived daemon holds the vault key once (removing the keychain
      friction). This is what turns the proven mechanism into a product that actually solves the
      governance/risk/privacy problem. The already-deferred "route masked request to Bedrock" /
      LiteLLM-gateway warm path. **#1 — nothing above it.**
- [ ] **Precision NO-GO — fix implemented and verified GO locally; PENDING REVIEW, not
      merged.** Branch `agent/claude/t10-fp-detector-fixes` implements the targeted fix
      next-actions previously described (`EntropyDetector` git-SHA-context exclusion,
      `PhoneDetector` ISBN-13/10 checksum + ZIP+4 shape exclusion) **plus a third residual
      found while re-running the harness after those two**: `is_structured_identifier` now
      splits on `=` too, closing a `LICENSE_KEY=ACME-2026-DEMO-KEY`-shaped false positive
      (`docs/decisions.md`, 2026-07-21 entry has the full detail and the reasoning for each).
      `cargo test --workspace` (all green), `clippy -D warnings`, `fmt --check` all pass.
      **`vg bench` verdict: GO** — false-positive-rate is now **0.0%** (was 16.7%), all other
      gates unchanged/PASS, hot-path p95 15.0 ms. **This does not ship on its own —
      human review + this repo's usual doubt-pass discipline before merge, same as every
      other T10-adjacent change** (see the many precedents in `docs/decisions.md`); the
      masking-proxy milestone (#1 above) and the display-collision fix (below) are unrelated
      and still open regardless of this gate's status.
- [ ] **Fix the display-collision corruption** (1 of 3 mask→demask round-trips). Implement
      collision-avoiding minting at intern time (skip an ordinal whose display already occurs in
      the raw text), as the T09 doubt-round and T10 eval both recommended, now with data.
- [ ] **Resolve or drop the dead `artefacts.by_language [dotenv]` config path** confirmed
      unreachable by the T10 eval (classify-before-parse makes it unreachable). Fix the wiring or
      remove the config surface.

## T11 review scope (attribution/hardening items surfaced during the build)

- [ ] **F4, demask authorisation is attribution, not authentication.** `--actor`/`--role` are
      self-asserted and the wrapped agent can invoke `vg demask` via its own shell. Candidate
      hardening: hooks refuse to spawn `vg demask` from inside a wrapped session; packs get
      restrictive perms; the vault key never enters the wrapped environment.
- [ ] **F3, upward state-dir discovery trusts any ancestor `.veilgremlin/`.** Now warns; T11
      should decide whether to refuse a discovered-not-created state dir, plus policy-signature
      verification (already stubbed for Phase 2).
- [ ] **F5, packs accumulate masked-text plaintext, unbounded.** Gitignore mitigates
      exfil-via-commit; a TTL/purge command (`vg pack purge`) is still deferred here.
- [ ] **dotenv-without-hint residual:** one seeded value only an artefact Block would catch (no
      filename hint). Decide detection vs accepted-residual.

## Later phases (designed, not started)

- [ ] Warm-path local NER (GLiNER), designed but off by default.
- [ ] LiteLLM gateway, MCP server mode, CI/CD mode, cloud-agent packaging.
- [ ] Synthetic-data generation and quasi-identifier leakage scoring.

## Standing conventions

- [ ] Add a `docs/build-log/` entry as each future material task lands, per the standing rule in
      `AGENTS.md`/`CLAUDE.md`/`CODEX.md`.
- [ ] Re-audit build-log coverage against actual work after each task.
