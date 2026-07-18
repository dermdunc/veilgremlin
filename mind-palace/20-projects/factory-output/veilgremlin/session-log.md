# Session Log: VeilGremlin

## 2026-07-04 - Repo moved to dermdunc (public); build dispatch still deferred

GitHub ownership moved from coderturtle (private) to dermdunc (public) — VeilGremlin is an
enterprise architecture/governance/risk tool, belongs under the professional-identity account.
T01/T02 build dispatch reviewed as ready but deliberately deferred again pending the wider
Hekton factory-readiness pass; a Rust-toolchain gap was found and logged for whenever that
build session starts. See repo `docs/decisions.md` and `docs/session-log.md` for full detail.

## 2026-06-30 - Initial scaffold

Project scaffolded as **factory-output**. Local-first privacy shield that keeps real PII and sensitive enterprise identifiers out of model context in agentic coding workflows.
---

## Session: T01 built + two doubt-driven-development rounds + PR merged

**Date:** 2026-07-14 22:11

### What Changed

Built Task T01 (9-crate Cargo workspace + CI + supply-chain skeleton) after the real ACT GO-LIVE dispatch stalled on a headless permission-mode gap. Ran two doubt-driven-development rounds (single-model, then Codex cross-model) that found and fixed a red CI job (cargo-deny-action needs Linux runners), missing test/toolchain-pin CI, stale reproducibility scripts, hardcoded dependency versions, and several stale-doc overclaims (including one round 1 itself introduced). Rewrote all 3 commits to use the Hekton commit footer (Hekton-Engine/Harness/Model/Workflow) instead of the generic Claude Code co-author line. PR #2 merged to main.

### Decisions

Built T01 directly rather than retry/loosen the nested dispatch's permission mode. Fixed the CI red job (ubuntu-latest for cargo-deny) rather than remove the check. Refactored intra-workspace deps through [workspace.dependencies] instead of hardcoded per-crate versions. Rewrote (not amended-in-place) the 3 commits via reset+cherry-pick to apply the Hekton footer, since they were already pushed with an open PR.

### Assumptions

Assumed the Hekton footer's engine=claude/harness=claude-code/model=claude-sonnet-5/workflow=t01-workspace-scaffold values are the right ones for this interactive-session build, per hekton-cli-lab's hkt commit schema.

### Risks

Branch name (gateway/run-20260714-T01) never matched agent-factory-plan.md's feat/<squad>-<task-id>-<slug> convention -- flagged, not fixed (already merged now, moot going forward for this branch but the dispatch tooling's naming convention still doesn't match VeilGremlin's own git contract for future tasks).

### Next Actions

- [ ] Human: dispatch/build T02 (freeze vg-core's shared types + interface-contracts.md v1) -- Wave B doesn't start until T01 and T02 both merge; consider whether engine-gateway-lab/agentic-control-tower's branch-naming convention should be reconciled with agent-factory-plan.md's before T02 dispatches.

## Session: T03 built (first genuinely unattended code-implement completion), reviewed, tollgate-approved

**Date:** 2026-07-15

### What Changed

First T03 dispatch attempt got a clarifying question back from headless claude -p instead of
code (one-shot mode, no follow-up channel); root-caused, task prompt rewritten with concrete
file/module/trait guidance, re-dispatched. Second attempt: real work -- five detector modules
(email/phone/ip/iban_sortcode/entropy, ~800 lines) + criterion bench. Reviewed by two rounds
of Codex cross-model doubt-driven-development (9 then 5 findings; 3 real bugs fixed). Tollgate
approved by the human; a real gateway-review.sh bug surfaced along the way (worked around,
logged as engine-gateway-lab RISK-0017).

### Decisions

Full reconciliation of all findings across both doubt-driven-development rounds is in
`docs/decisions.md`, not repeated here.

### Assumptions

None beyond what the two review rounds directly verified.

### Risks

`engine-gateway-lab` RISK-0017 (new): `gateway-review.sh` resolves `output_artifact` relative
to its own repo root unconditionally, wrong for ACT-dispatched cross-repo worktree tasks --
worked around for T03, not fixed at the root.

### Next Actions

- [ ] Human: review/merge the T03 PR once opened.
- [ ] Decide serial-vs-concurrent for T04/T05/T05b/T06/T08.
- [ ] `engine-gateway-lab` RISK-0017 needs a real fix before the next Wave B tollgate.
- [ ] Note for a future mirror-sync session: this mirror was found missing T02's entire
      session entry before this update (pre-existing drift, not introduced this session) --
      flagged rather than silently backfilled.

## Session: Fan-out testing-strategy review + real CI latency gate + Codex dogfooding plan + real detector census

**Date:** 2026-07-16

### What Changed

Merged T03's PR and `engine-gateway-lab`'s RISK-0017 PR. Reviewed the fan-out phases
against VeilGremlin's actual goal (invisible PII masking, trading-system latency
discipline) and found 3 real testing-strategy gaps: T04/T08 had no requirement to
integration-test against T03's real `Finding`/`Span` output (mocks only), and T09 had no
point where a human confirms the "invisible control" goal is actually met. Added a real,
CI-enforced latency-regression gate (`tests/latency_gate.rs`) ahead of T10. Ran a Codex
planning pass (explicitly not a review) on a dogfooding strategy, reconciled in
`docs/decisions.md`. Built and ran a read-only detector census against 197 real Hekton
files.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-16 entry.

### Assumptions

None beyond what the census run and the manual false-positive verification directly
showed.

### Risks

Entropy (2468 findings) and phone (783 findings) detectors are dominated by false
positives on Hekton's own operational-ID and date shapes -- a real, evidenced precision
problem, not fixed or guessed at this session; needs a human product decision before
T06/T07.

### Next Actions

- [x] Human decision made: ran the three options through a Codex planning pass
      (hybrid recommended, approved); fixed and measured (entropy -90%, phone -91% on
      isolated identical content).
- [ ] Re-run the census as each Wave B/C task lands.
- [ ] Serial-vs-concurrent decision for the remaining Wave B tasks still open.

## Session: Fixed the entropy/phone false-positive finding (Codex-planned, hybrid, measured)

**Date:** 2026-07-16

### What Changed

Ran the census's open false-positive question through a Codex planning pass; Codex
recommended a hybrid (fix detectors now, keep T10 as the formal gate) after reading the
actual frozen contracts. Implemented `PhoneDetector`'s ISO-date exclusion and
`EntropyDetector`'s structured-identifier exclusion. The entropy fix needed a
mid-session correction: the first version assumed Hekton's own run-ID shapes were
dominant and barely helped when measured; the real dominant classes were file paths and
snake_case/kebab-case identifiers, fixed generically.

### Decisions

Full record, including the mid-session correction, in repo `docs/decisions.md`'s
2026-07-16 entry.

### Assumptions

None beyond what the isolated before/after measurement (via `git stash` on identical
`engine-gateway-lab` content) directly showed.

### Risks

Accepted residual: a dictionary-word passphrase joined by delimiters would also be
excluded by the entropy fix. Remaining ~10% of findings left for T10's formal gate.

### Next Actions

- [ ] Re-run the census as each Wave B/C task lands.
- [ ] Serial-vs-concurrent decision for the remaining Wave B tasks still open.

## Session: Merged PR #6; added a build log

**Date:** 2026-07-17

### What Changed

Merged PR #6, pulled `main` up to date. Added `docs/build-log/` — a dated,
deliberately-written build-in-public narrative distinct from `docs/session-log.md`, per
the Hekton Workshop Gremlin's convention. Backfilled 7 entries covering the repo's
history to date. Wired into `AGENTS.md`/`CLAUDE.md`/`CODEX.md` as a standing rule, and
linked from `README.md` and a refreshed `docs/project-walkthrough.md`.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 entry.

### Assumptions

None beyond the human's explicit choice of lightweight dated docs over a full
publishable site.

### Risks

None new — backfilled entries only narrate what `docs/decisions.md`/`docs/session-log.md`
already say publicly.

### Next Actions

- [ ] Add a build-log entry as each future task lands, per the new standing rule.
- [ ] Revisit whether the build log earns a publishable site later.

## Session: T04 — typed-placeholder + HMAC keying, tollgate-approved

**Date:** 2026-07-17

### What Changed

Dispatched T04 (`crates/vg-core/src/keying.rs` + `keying_integration.rs`). Headless
sandbox blocked all `cargo`/`rustc` execution, so the code was hand-traced carefully but
never compiled by the dispatching agent. Verified during review: compiled clean, one
trivial clippy fix, one fmt pass. A Codex cross-model doubt-pass then found and fixed 3
real bugs (`EntityType::Custom` HMAC collision, compact-vs-spaced IBAN/sort-code/phone
keying differently, `PlaceholderKey`'s `Debug` leaking the real HMAC hex) plus a
documentation gap. Also folded in T04's task-spec expansion and a new T05-on-T04
dependency (both originally on a separate branch, PR #8, now superseded and closed).
Tollgate-approved by the human via `gateway-review.sh`, which hit the same RISK-0017
output-path bug as T03 (worked around the same way, not fixed at the root).

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 T04 entry, including the
doubt-driven-development reconciliation.

### Assumptions

None beyond the judgment calls the dispatch instructions explicitly authorised (recorded
in `docs/decisions.md`, e.g. per-namespace ordinal scoping, Luhn/mod-97 exposed as
validators only, not wired into placeholder display).

### Risks

A real cross-task interface gap found during review: `Keyer`'s ordinal counters must be
reseeded from T05's persisted vault state at construction time, or ordinals can
collide/drift across process restarts. `vg-vault` (T05) doesn't exist yet, so this
couldn't be fixed in T04 itself — recorded as a hard requirement in T05's own acceptance
criteria so it can't be silently skipped.

### Next Actions

- [ ] Human: review/merge the T04 PR.
- [ ] T05 (`vg-vault`) must call into `Keyer`/`placeholder_key` and reseed ordinals from
      persisted state — see the risk above.
- [ ] Decide serial-vs-concurrent for the remaining Wave B tasks (T05/T05b/T06/T08).

### Validation status

- `cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check
  && cargo test`: PASS, 37 `vg-core` unit tests (31 keying-specific), 5 cross-crate
  integration tests, 7 conformance tests, 46 detector tests, 1 latency-gate test, all
  green — confirmed in both the worktree and the real repo after the tollgate applied
  the diff.

## Session: PR #9 merged; vault sync; build-log coverage audit

**Date:** 2026-07-17

### What Changed

Merged PR #9. Backed up and synced the Obsidian vault via `scripts/sync-mirror-to-vault.sh`
(this project's vault entry hadn't been touched since before T01) — found and fixed a real
bug in that script (a stale `git add` pathspec silently blocked every commit it ever tried
to make). Audited `docs/build-log/` against actual work done, per a human request to
confirm it will genuinely track how/why/what this project is building when delivered as
part of the final project. Found and fixed one real gap: T04's doubt-pass bugs never got
their own entry.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 entry.

### Next Actions

- [ ] Re-audit build-log coverage after each future task lands.
- [ ] Decide serial-vs-concurrent for the remaining Wave B tasks (T05/T05b/T06/T08).

## Session: T05 (`vg-vault`) built under Opus, reviewed, tollgate-approved

**Date:** 2026-07-17

### What Changed

Dispatched T05 (SQLCipher vault) under Opus after the default model hit a usage-credit
wall. It implemented `VaultStore` (keychain-wrapped DB key, AES-256 at rest, namespace
isolation, TTL/purge) and honored the T04-mandated `Keyer` ordinal reseed
(`seed_ordinal` + a `MAX(ordinal) GROUP BY` reseed at open). Verified during review; two
rounds of Codex cross-model critique found two real bugs, both fixed: an expired-row-reuse
bug (`intern` could return an unresolvable placeholder) and a `UNIQUE` ordinal guard that
silently didn't fire for fixed entity types (NULL `entity_custom` + SQLite NULL-is-distinct
semantics → fixed with `COALESCE`). Tollgate-approved by the human.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 T05 entry.

### Next Actions

- [ ] T05b/T06/T08 tollgates still to be completed (they did not land with T05).

## Session: T05b audit sink built (second no-compiler headless dispatch)

**Date:** 2026-07-17

### What Changed

Implemented the T05b audit sink in `crates/vg-audit/`: an append-only JSON Lines
`AuditSink` (`JsonlAuditSink`, fsynced per write, never truncated), a versioned on-disk
schema mirroring the frozen `vg-core` contract types, and the acceptance property test
that no raw value ever reaches the persisted bytes (verbatim or escaped). The dispatch
had no reachable compiler — same factory gap as T04, now a pattern — which drove the
storage and dependency choices: everything was selected so the required `Cargo.lock`
change stayed a hand-verifiable edit between already-locked packages.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 T05b entry.

### Next Actions

- [ ] Reviewer: run the full verify chain on the T05b branch — nothing compiled
      in-session.
- [ ] Decide serial-vs-concurrent for the remaining Wave B tasks (T05/T06/T08).

### Validation status

- Not compiled or tested in-session (toolchain permission-blocked); self-review caught
  three would-be compile errors before handoff.

## Session: T06 (`vg-policy`) built under Opus, reviewed, landed

**Date:** 2026-07-17

### What Changed

Dispatched T06 (policy engine) concurrently under Opus. Implemented `PolicyEngine` with
3-layer resolution (session > repo > global), a signed-pack verification stub, and the
security-load-bearing hard-deny: `demask_allowed` returns false for
`RemoteModelPrompt`/`ObservabilitySink` via a direct enum check *before* any pack rule, so
no pack can override it. Chose `serde_json` (already in the lockfile) to keep `--locked`
green with no new dependency.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 T06 entry. Two Codex critique rounds:
round 1 verified hard-deny unbypassability + layering fail-safety and prompted documenting
the `artefact_default = Pass` asymmetry (correct, flagged for T07); round 2 returned "no
issues found" — the only Wave B task whose complete second-round critique surfaced nothing.

### Next Actions

- [ ] T08 to land next (base-drift recovery, same as T05b/T06).

## Session: T08 (`vg-parsers`) built under Opus, reviewed, landed — Wave B complete

**Date:** 2026-07-17

### What Changed

Dispatched T08 (file-aware parsers) concurrently under Opus. Implemented `Parser` for
json (hand-rolled byte tokenizer for offsets + tolerance), yaml (serde_yaml gate + block
scanner + json fallback for flow style), toml, csv, log (regex shape-based), env/dotenv,
and rust (tree-sitter), plus a cross-crate integration test feeding real spans into the
T03 detectors. Verified during review — regenerated Cargo.lock (4 new deps; tree-sitter
pair compiled), fixed two clippy items, two borrowed-temporary test errors, and one real
yaml flow-style mis-parse bug. Closed RISK-0011 (the build gap) as mitigated.

### Decisions

Full record in repo `docs/decisions.md`'s 2026-07-17 T08 entry. Two Codex critique rounds:
no panic/out-of-bounds risks found (the hard never-panic + span-bounds contract holds); the
residual "partial spanning could miss a value" concern is documented with two concrete
reproducers (a `#` inside a quoted YAML value; single-quoted YAML flow scalars), flagged for
T07 — the T03 detectors currently ignore spans, so it can't bite yet.

### Next Actions

- [ ] **Wave B is complete** (T03/T04/T05/T05b/T06/T08 all merged). T07 (masking pipeline)
      is next — it wires detectors → policy → vault → audit and must honor the T07 notes
      carried forward from T05/T06/T08.

## Session: T07 (masking pipeline) built under Opus, doubted by Fable + Codex, tollgate-approved

**Date:** 2026-07-18

### What Changed

Dispatched T07 under Opus from a clean current `main` (the tollgate auto-apply finally
worked with no base-drift surgery). Opus wired `scan()`/`mask()` through all six Wave B
crates, honored every banked hard requirement, and executed the sanctioned contract change
(`mask` gained `ctx: &Context`, interface-contracts.md v1 → v1.1) properly.

### Decisions

Full doubt-driven-development record in repo `docs/decisions.md` (2026-07-18 T07 entry).
Two fresh-context review rounds: **Fable** (Opus authored, Fable doubted — a complete
13-finding verdict: 2 High fixed, incl. partial-overlap trimming and the `.env`-dotfile
fail-open; 4 Med + 4 Low fixed; 2 documented forward), then **Codex** on the post-fix diff
(1 Medium introduced by the fixes' own interaction — Scan counts per-fragment — fixed with
a mock-detector regression). Stop condition: diminishing returns. 199 tests / 0 failures.

### Next Actions

- [ ] T09 (CLI + adapter) is next — hard requirement banked: demask via `MappingRef`s
      only, never placeholder-pattern scanning. T10/T11 after.
