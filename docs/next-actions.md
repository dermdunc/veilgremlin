# Next Actions: VeilGremlin

Repo source-of-truth for the work queue. Tasks T01–T11 are defined in [`architecture/work-breakdown.md`](architecture/work-breakdown.md); the build method is in [`architecture/agent-factory-plan.md`](architecture/agent-factory-plan.md).

## Immediate (Wave A — foundation, must merge before parallel work)

- [x] **Confirm first push** to GitHub — resolved: `origin` now uses the `github.com-coderturtle`
      SSH host alias (a real fix, not the HTTPS+gh-token workaround); `main` is pushed and
      confirmed via `git ls-remote --heads origin` (2026-07-04).
- [x] **Repo ownership move to dermdunc — complete (2026-07-04).** Transferred via
      `gh api repos/coderturtle/veilgremlin/transfer -f new_owner=dermdunc`, human-accepted as
      dermdunc, visibility flipped to public
      (`gh repo edit dermdunc/veilgremlin --visibility public --accept-visibility-change-consequences`),
      local `origin` repointed to `git@github.com:dermdunc/veilgremlin.git`, and confirmed
      reachable via `git ls-remote origin`. `.hekton/project.yaml`, `.hekton/governance.yaml`,
      `.hekton/risk-register.yaml`, the repo-local mind-palace mirror, and the `Owner:`/`Privacy
      boundary:` headers in `README.md`/`CLAUDE.md`/`AGENTS.md`/`CODEX.md`/`docs/spec/...` all
      updated to match. See `docs/decisions.md` for the full record.
- [x] **Corrected 2026-07-14: the "no Rust toolchain" blocker above was stale.** Re-checked
      against `agentic-control-tower/docs/go-live-dependencies.md` (which had already found the
      toolchain installed on 2026-07-07) and independently re-verified: `cargo`/`rustc` were
      already present via Homebrew. Only `cargo-deny` was actually missing — installed this
      session (`brew install cargo-deny`). `check-prereqs.sh` still needs the prereq-check
      update named below; not applied this session (prepared as a reviewed diff in
      `~/hekton`'s `docs/plans/veilgremlin-v1-dogfood-runbook-v1.md`).
- [x] **GO-LIVE dispatch, real, 2026-07-14.** `dag dispatch T01` ran for real through
      `agentic-control-tower` + `engine-gateway-lab` — the factory's first real end-to-end
      build event. **Finding:** the nested `claude -p --permission-mode acceptEdits` headless
      call stalled on a Bash-command permission prompt (checking for the Rust toolchain — the
      very blocker just corrected above) with no human to approve it, and returned a
      "waiting on your approval" message as its final answer instead of erroring — `dag status`
      showed `in-progress` with a clean exit and nothing built. This is a real unattended-dispatch
      gap (headless `-p` mode has no path to approve a Bash tool call), not a VeilGremlin-specific
      bug — worth flagging to `engine-gateway-lab`/`agentic-control-tower` for their own
      unattended-loop work. **Resolution this session:** built T01 directly instead of retrying
      the nested dispatch (human decision, given the toolchain question was already answered) —
      see the T01 entry below.
- [x] **T01 — done, 2026-07-14.** Cargo workspace (9 crates: `vg-core`, `vg-detectors`,
      `vg-parsers`, `vg-vault`, `vg-policy`, `vg-audit`, `vg-cli`, `vg-adapters-claude`,
      `vg-bench`) + CI (`.github/workflows/ci.yml`: fmt, clippy -D warnings, cargo-deny,
      cargo-audit, build --locked, bench compile-check) + `deny.toml` + release skeleton
      (`release/README.md`, SBOM/signing stubs). Crates are empty skeletons per
      `interface-contracts.md`'s note that Squad 0 (T02) owns the canonical trait/type
      definitions — this task scaffolds the workspace they'll land in. Verified locally:
      `cargo build --locked && cargo fmt --check` (the DAG's own verify command) passes;
      also `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo deny check`,
      and `cargo audit` all pass; `cargo bench --workspace --locked --no-run` compiles. PR
      opened against `main` from `gateway/run-20260714-T01`.
- [x] **Two rounds of doubt-driven-development on the T01 PR — done, 2026-07-14.** Round 1
      (single-model): found the `deny` CI job was actually failing on the real GitHub Actions
      run (`macos-latest` + a Docker container action); fixed, re-verified the real run is now
      green (all 6 jobs pass). Round 2 (Codex cross-model): found no `cargo test` CI job, an
      unpinned Rust toolchain, a stale bench-gating claim in `docs/risks.md`, stale
      reproducibility scripts (`check-prereqs.sh`/`local-assumptions.md`/`verify-project.sh`
      didn't check for the Rust toolchain at all — the exact same gap flagged back on
      2026-07-04 and never actually applied until now), hardcoded intra-workspace dependency
      versions that would drift on a workspace version bump, and — ironically — a "T01 is
      merged" overclaim introduced by round 1's own project-walkthrough.md fix (the PR isn't
      merged yet). All fixed; see `docs/decisions.md` for the full record.
- [x] **T01 PR merged, 2026-07-14** (github.com/dermdunc/veilgremlin/pull/2), plus its
      session-closeout PR #3.
- [x] **T02 — done, 2026-07-15.** Freeze shared types + library API in `vg-core`; trait seams
      (`Detector`, `Parser`, `VaultStore`, `PolicyEngine`, `AuditSink`); contract-conformance
      test helpers (`vg_core::conformance`) + a full worked example against mock impls
      (`crates/vg-core/tests/conformance_stubs.rs`). Real dispatch this time actually built the
      code (unlike T01's stall) but hit a ~10-minute tool timeout before it could close out
      formally — picked up the work in place: it compiled clean, and `rehydrate`'s
      destination hard-deny gate (`RemoteModelPrompt`/`ObservabilitySink`, regardless of
      actor) is real logic, not a stub, since it doesn't depend on any Wave B crate. Everything
      else (`scan`/`mask`/`benchmark`, `rehydrate`'s allowed-destination path) is `todo!()`
      pointing at the task that wires it (T07/T09/T10), matching `interface-contracts.md`'s own
      note that Phase 1 pipeline assembly happens later. Full T02 verify_command passes:
      `cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check
      && cargo test` (6 real tests, all green).

## This Week (Wave B — dispatch in parallel once T01+T02 merge)

- [ ] **T03/T04** detectors + placeholder/HMAC keying — *Squad 1 (+3)*
- [ ] **T08** parsers (logs, diffs, JSON/YAML, `.env`, tree-sitter) — *Squad 2*
- [ ] **T05** SQLCipher vault + keychain wrap + TTL — *Squad 3*
- [ ] **T05b** audit sink (append-only, redaction-safe) — *Squad 5*
- [ ] **T06** native 3-layer policy engine — *Squad 4*

## Later (Wave C/D)

- [ ] **T07** masking pipeline wiring in `vg-core` — *Squad 0*
- [ ] **T09** `vg` CLI + Claude Code adapter + Bedrock path + demask gate — *Squad 6*
- [ ] **T10** seeded corpus + eval harness + Go/No-Go report — *Squad 7*
- [ ] **T11** review + `/security-review` + milestone sign-off — *Review Agent + human*
- [x] Decide repo visibility flip (private → public) — decided 2026-07-04: public, as part of
      the dermdunc ownership move above, not deferred to a later open-source milestone.
- [ ] ~~Fix coderturtle SSH key registration~~ — superseded 2026-07-04: the repo is moving off
      coderturtle entirely, so this is moot for VeilGremlin (the SSH host alias itself
      (`github.com-coderturtle`) remains fine for other coderturtle-owned repos).

## Session Update: 2026-07-14 — T01 built + two doubt-driven-development rounds + PR merged

- [x] Dispatch/build T02 — done 2026-07-15, see the Wave A entry above.

## Session Update: 2026-07-15 — T02 built

- [x] Doubt-driven-development pass (two rounds: single-model + Codex cross-model) — done
      2026-07-15. Most severe: `interface-contracts.md` was never frozen/reconciled despite
      being T02's literal acceptance criterion — now fixed, 11 missing types added, two
      deviations reconciled. Most severe code finding: the conformance example's `MockVault`
      ignored its namespace parameter on resolve — real cross-namespace leak in the template
      every Wave B squad reads; fixed, and now covered by a test. Six more conformance-helper
      gaps fixed; one contract-shape limitation documented (not fixed — `Secret`'s zeroize is
      cosmetic given `rehydrate`'s own return type). Full record in `docs/decisions.md`.
- [ ] Human: review/merge the T02 PR.
- [ ] Once T01 + T02 are both merged, batch-dispatch the five Wave B squads (T03/T04, T05,
      T05b, T06, T08).
- [ ] Still open: the branch-naming mismatch between the ACT/engine-gateway dispatch
      tooling's convention and `agent-factory-plan.md`'s `feat/<squad>-<task-id>-<slug>`
      convention (flagged, not fixed, since T01).

## Session Update: 2026-07-15/16 — T03 built (first genuinely unattended completion), reviewed, approved

- [x] T03 dispatched (twice — first attempt got a clarifying question, prompt rewritten,
      re-dispatched), reviewed (my own pass + two Codex cross-model rounds, 3 real bugs
      fixed), tollgate-approved, session closed. Full record in `docs/decisions.md` and
      `docs/session-log.md`.
- [ ] Human: review/merge the T03 PR once opened.
- [ ] Decide serial-vs-concurrent for T04/T05/T05b/T06/T08 now that the rework loop and the
      RISK-0016 ledger fix are both proven for real. Default serial per the runbook unless a
      future task completes cleanly unattended.
- [ ] `engine-gateway-lab` RISK-0017 (gateway-review.sh's output-path resolution breaks for
      ACT-dispatched cross-repo worktree tasks) needs a real fix before the next Wave B
      tollgate — worked around for T03, not fixed at the root.

## Session Update: 2026-07-16 — Fan-out review, Codex dogfooding plan, real latency gate, real detector census

- [x] T03 PR merged; RISK-0017 documented and merged in `engine-gateway-lab`.
- [x] Added a real, CI-enforced latency-regression gate (`crates/vg-detectors/tests/latency_gate.rs`)
      instead of waiting for T10 — plain `#[test]`, runs on every PR already.
- [x] Added cross-crate integration requirements to T04, T08's acceptance criteria (both pair
      concretely with the already-closed T03) and a UX-latency human-verification criterion to
      T09's. See `.hekton/veilgremlin-dag.toml` and `docs/architecture/work-breakdown.md`.
- [x] Codex planning pass on dogfooding strategy — full reconciliation in `docs/decisions.md`
      2026-07-16 entries.
- [x] Built and ran `crates/vg-detectors/examples/census.rs` (read-only, no matched values ever
      printed/stored) against 197 real files across VeilGremlin + `engine-gateway-lab`. **Real
      finding, not yet resolved:** entropy (2468 hits) and phone (783 hits) detectors are
      dominated by false positives on Hekton's own operational IDs (`run-YYYYMMDD-EG-NNN` shapes)
      and dates — verified by hand against the actual repo content. Latency itself is fine
      (11.2ms / 197 files). Full detail and the open design question (allowlist? tighter
      heuristics? measure via T10's `false_positive_rate` metric?) in `docs/decisions.md`.
- [ ] **Human decision needed:** how to address the entropy/phone false-positive rate before
      Task T06 (policy)/T07 (pipeline) go live — not decided or guessed at this session,
      genuinely needs a product call.
- [ ] Re-run `cargo run --example census -- <paths>` as each Wave B/C task lands, per the Codex
      plan's ladder (detector-only now → parser+detector after T08 → stubbed mini-pipeline after
      T04/T05/T06/T05b → real `mask()` after T07 → real dogfood after T09).
- [ ] Decide serial-vs-concurrent for the remaining Wave B tasks (T04/T05/T05b/T06/T08) — still
      open from the prior session update above.
