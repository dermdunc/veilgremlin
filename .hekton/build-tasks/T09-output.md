# T09 output — `vg` CLI + Claude Code adapter (contract v1.1 → v1.2 → v1.3)

**Run:** run-20260718-T09 (engine: claude-cli / Opus). The dispatch died mid-response
(`API Error: Connection closed mid-response` — dispatch failure mode #4) after authoring
the adapter crate; the run was rescued in place per the T02 precedent. This output file is
written by the rescue session; the per-change narrative lives in `docs/decisions.md`
(2026-07-18 T09 entries, including both doubt-pass rounds).

## What was built

- **`crates/vg-adapters-claude`** (Opus-authored, rescued): `runtime.rs` (`Engine` over
  `StatePaths`, default-policy bootstrap, repo namespace), `state.rs` (`.veilgremlin/`
  layout, resolution precedence, self-gitignore), `hook.rs` (the three hook events →
  `mask`, §8 **v1.3** protocol), `pack.rs` (`StoredPack` with schema + binding
  validation), `wrapper.rs` (hook settings JSON, quoted commands, Bedrock passthrough).
- **`crates/vg-cli`** (rescue-authored): `run`, `hook`, `inspect`, `diff --masked`,
  `demask`, `audit`, `policy check`, `vault stats`; fail-closed hook path at the process
  boundary.
- **Contract:** v1.2 (`MaskedPack.bindings` + `rehydrate` re-sign) and v1.3 (§8 corrected
  to the platform's real exit-code/JSON semantics) — `docs/architecture/
  interface-contracts.md` amended, both via the contract-change protocol.
- **`vg-core::rehydrate`** wired: hard-deny first (no policy/vault consult before the
  decision), denial-only DemaskDecision audit, boundary-aware single-pass substitution.
- **`vg-vault`**: 5s busy timeout (one-process-per-hook is now the normal mode).
- **Docs:** `docs/runbook-hooks.md` (human UX-invisibility session walkthrough).

## Review posture

Two adversarial rounds per the doubt-driven-development skill: Fable fresh-context
(18 findings, 6 High — including the §8 inversion that voided the hook mechanism
end-to-end) and Codex cross-model (7 findings — including the fail-open seam around the
round-1 fix), plus a round-2 addendum fix (hard-deny denial re-audited). All findings
reconciled in `docs/decisions.md`; open trade-offs routed to T10/T11.

## Validation

Full workspace: build, `clippy -D warnings`, `fmt`, **218 tests / 28 binaries /
0 failures**. Live smoke: mask → placeholders; demask byte-perfect restore; hard-deny
DENIED + audited; `.env` write blocked exit 2.

## Not done here (routed)

- Human UX-invisibility session (runbook) — T09 acceptance, human-run after landing.
- Eval-harness metrics (`benchmark` still `todo!`) — T10.
- State-dir trust, demask authn, pack TTL/purge, display-collision-avoiding minting — T11
  candidates (see decisions trade-off list).
