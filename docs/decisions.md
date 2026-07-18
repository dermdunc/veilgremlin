# Decisions: VeilGremlin

## ADR Log

| Date | Decision | Rationale |
|---|---|---|
| 2026-06-30 | Initial scaffold as factory-output (Hekton) | Local-first privacy shield; built by the Hekton factory; no `hekton-` prefix per taxonomy |
| 2026-06-30 | Repo under **coderturtle** GitHub account, **private** initially | Factory-output routing chose coderturtle; private is reversible — flip to public when ready to open-source |
| 2026-06-30 | ADR-001 Core language = **Rust** | Memory/thread safety, no GC, small trusted core, enterprise reviewability |
| 2026-06-30 | ADR-002 Local vault = **SQLCipher SQLite** | Encrypted, local, queryable; supports normalisation/TTL/audit |
| 2026-06-30 | ADR-003 Detector mix = **deterministic hot path + optional GLiNER warm path** | Latency + explainability + recall balance |
| 2026-06-30 | ADR-004 First integration = **Claude Code wrapper + hooks on Bedrock** | Fastest enterprise proof, no central platform dependency |
| 2026-06-30 | ADR-005 Masking = **typed placeholders, not synthetic values** | Transparent, stable, auditable, debug-safe |
| 2026-06-30 | ADR-006 Demasking = **explicit, local, policy-gated** | Prevents re-exposure to cloud models; supports oversight |
| 2026-06-30 | ADR-007 Policy = **native YAML/TOML now; Cedar later** | Low dependency now; strong auth later |
| 2026-06-30 | ADR-008 Gateway = **LiteLLM later; core stays separate** | Hardened small core + provider-independence |
| 2026-06-30 | ADR-009 Supply chain = **sign + SBOM + reproducible builds + no telemetry** | Trust prerequisite for a privacy binary |
| 2026-06-30 | ADR-010 Placeholder key = **salted HMAC over canonicalised value+type+namespace** | Stable consistency without leaking original structure |
| 2026-06-30 | **Build method = agent factory, contract-first** | Squads own one crate each; interfaces frozen end of Wave A to enable safe parallel agent work (see `architecture/agent-factory-plan.md`) |
| 2026-07-03 | Build driven through Hekton's task-DAG orchestrator (`agentic-control-tower`), not manual per-task dispatch | `.hekton/veilgremlin-dag.toml` is now the machine source of truth for the T01-T11 DAG (transcribed from `architecture/work-breakdown.md`); `.hekton/build-tasks/*.md` are generated engine-gateway-lab task specs (regenerate via `dag gen-specs`, don't hand-edit). `.control-tower/` tracks each task's lifecycle. All build tasks route through `claude-cli`/`codex-cli` (cloud, V1 scope — no local-model build capability exists yet) at `privacy: vendor-allowed` (this repo's own source isn't privacy-sensitive; see `.hekton/project.yaml`'s `privacy_boundary: internal`). See `agentic-control-tower`'s root `decisions.md` ADR-013 for the full orchestrator design. |
| 2026-07-04 | Repo ownership moved from **coderturtle** (private) to **dermdunc** (public) | VeilGremlin is an enterprise architecture/governance/risk tool, not agentic-engineering tooling — belongs under the professional-identity account per Hekton's new domain-based GitHub routing decision (see `~/hekton/docs/decisions.md`, 2026-07-04). Supersedes the 2026-06-30 "coderturtle, private" decision above. |
| 2026-07-17 | ADR-011 (T05) `vg-vault` = **SQLCipher via `rusqlite` (vendored OpenSSL), OS-keychain-wrapped DB key, per-install salt in an encrypted `meta` table; `Keyer` ordinal counters reseeded from persisted rows at open** | Encrypted-at-rest reversible mapping store; keychain wrap keeps the key off disk; reseed prevents display-ordinal collision/drift across process restarts. Added an additive `Keyer::seed_ordinal` to `vg-core` (not a frozen-contract change). See the 2026-07-17 T05 entry below. |

Full reasoning and the Mermaid-illustrated design are in [`spec/requirements-and-design-spec.md`](spec/requirements-and-design-spec.md).

## 2026-07-04 - Repo ownership moved to dermdunc; visibility flipped to public

### Context

The original 2026-06-30 scaffold routed VeilGremlin to `coderturtle` (private) via the standard
factory-output "prompt, default coderturtle" routing rule. coderturtle (the human) determined
this was the wrong account for this specific project: VeilGremlin is an enterprise
architecture/governance/risk tool (a privacy shield for agentic coding workflows), which belongs
under the `dermdunc` professional-identity account, not the `coderturtle` agentic-engineering
demo/workshop account. This prompted a wider Hekton policy addition — see
`~/hekton/docs/decisions.md`'s 2026-07-04 entry adding a domain heuristic to factory-output
GitHub routing.

### Decision

- Transferred the GitHub repo from `coderturtle/veilgremlin` to `dermdunc/veilgremlin` via
  `gh api repos/coderturtle/veilgremlin/transfer -f new_owner=dermdunc`, then human-accepted the
  transfer as `dermdunc` (GitHub requires the receiving account to accept manually — no API path
  for that step).
- Flipped visibility to public (`gh repo edit --visibility public --accept-visibility-change-consequences`)
  in the same session — not deferred to a later "ready to open-source" milestone as the original
  scaffold decision assumed.
- Updated the local `origin` remote to `git@github.com:dermdunc/veilgremlin.git` (dermdunc's SSH
  host, keyed to `~/.ssh/id_ed25519`) and verified reachability with `git ls-remote origin`.
- Updated all current-state metadata to match: `.hekton/project.yaml` (`owner`, `github_account`,
  `github_remote_url`, `privacy_boundary: public`, `architecture.owner`), `.hekton/governance.yaml`
  and `.hekton/risk-register.yaml` (`owner`), the repo-local mind-palace mirror
  (`mind-palace/.../index.md`), and the `Owner:`/`Privacy boundary:` headers in `README.md`,
  `CLAUDE.md`, `AGENTS.md`, `CODEX.md`, and `docs/spec/requirements-and-design-spec.md`.
- Closed `docs/risks.md`'s RISK-0010 (coderturtle SSH key registered to the wrong account) as
  moot — the repo no longer pushes as coderturtle at all.
- Left historical entries alone: `docs/session-log.md` and this file's own 2026-06-30 entries
  describe what was true at the time and are not rewritten.

### Consequences

- VeilGremlin is now publicly visible at `github.com/dermdunc/veilgremlin` — the code, docs, and
  full history (including this decision) are world-readable from this point forward.
- No build work has happened yet (T01/T02 dispatch remains deliberately deferred, per
  `docs/next-actions.md`), so this move happened before any real implementation existed to review
  for accidental sensitive content — the safer order, rather than flipping visibility after code
  exists.
- Future factory-output projects should get the coderturtle-vs-dermdunc domain call made
  explicitly at scaffold time, per the new Hekton-wide routing guidance, rather than needing a
  post-hoc move like this one.

## 2026-07-14 - GO-LIVE dispatch (real) and T01 built directly after a dispatch-mechanism gap

### Context

The "no Rust toolchain" blocker (2026-07-04, above) was re-checked this session against
`agentic-control-tower/docs/go-live-dependencies.md`, which had already found the toolchain
installed on 2026-07-07 (re-checked 2026-07-13) — the local claim was stale. Independently
re-verified: `cargo`/`rustc`/`cargo-audit` were already present via Homebrew; only `cargo-deny`
was missing, installed this session. With the toolchain question resolved, the human authorized
the real GO-LIVE dispatch: `dag dispatch T01` through `agentic-control-tower` +
`engine-gateway-lab`.

### What happened

The dispatch mechanism itself worked correctly end to end: DAG state transition, worktree
isolation (`engine-gateway-lab/.worktrees/run-20260714-T01`, branch
`gateway/run-20260714-T01`), routing to `claude-cli`, and the verify gate all fired as designed.
But the nested `claude -p --output-format json --permission-mode acceptEdits` headless call
tried to run a Bash command (checking whether the Rust toolchain was actually installed — the
same stale claim this session had just corrected) and stalled on a tool-use permission prompt
with no human present to approve it in a one-shot `-p` invocation. Rather than erroring, it
returned "The tool is waiting on your approval for this command..." as its final result. The
gateway adapter treated this as a normal completion (no error, non-empty result text), wrote it
to `.hekton/build-tasks/T01-output.md`, and the DAG's verify step correctly failed on the
missing `Cargo.toml` — but nothing had actually been built, and the run's own timestamps
(dispatch and verify within the same second) confirm no real work occurred.

### Decision

Given three options (retry with `bypassPermissions`, block the run and stop, or build T01
directly), the human chose to build T01 directly in the existing worktree rather than debug or
loosen the nested dispatch's permission mode — the toolchain question was already settled, and
building it directly avoided disabling the nested agent's own confirmation safety net just to
work around a one-off stale-data trip-up. Built: a 9-crate Cargo workspace (`vg-core`,
`vg-detectors`, `vg-parsers`, `vg-vault`, `vg-policy`, `vg-audit`, `vg-cli`,
`vg-adapters-claude`, `vg-bench` — matching `agent-factory-plan.md`'s squad-to-crate mapping),
empty skeleton crates per `interface-contracts.md`'s note that Squad 0/Task T02 owns the
canonical trait and type definitions, `.github/workflows/ci.yml` (fmt, clippy -D warnings,
cargo-deny, cargo-audit, build --locked, bench compile-check), `deny.toml`, and a release
skeleton (`release/README.md`) with SBOM/signing explicitly stubbed, not silently omitted.

Verified locally before committing: the DAG's own verify command
(`cargo build --locked && cargo fmt --check`) passes; also `cargo clippy --workspace
--all-targets --locked -- -D warnings`, `cargo deny check`, and `cargo audit` all pass;
`cargo bench --workspace --locked --no-run` compiles. One real fix needed along the way:
`cargo-deny`'s bans check flags intra-workspace `path` dependencies with no `version` as
wildcard dependencies — added `version = "0.1.0"` alongside every `path = "../vg-*"` dependency
to satisfy it, a standard pattern, not a workaround.

### Consequences

- This is VeilGremlin's actual first line of code and the factory's first real end-to-end build
  through the DAG orchestrator → engine-gateway → adapter chain — the evidence both
  `agentic-control-tower` and `engine-gateway-lab` need for their own Transformer/platform
  promotions.
- The unattended-dispatch gap (headless `-p` mode cannot approve a Bash tool call) is real and
  not VeilGremlin-specific — flagged in `docs/next-actions.md` for `engine-gateway-lab`/
  `agentic-control-tower` to pick up; not fixed here, since fixing the dispatch mechanism itself
  is out of scope for a VeilGremlin build task.
- T02 (freeze `vg-core`'s shared types) is next; per `agent-factory-plan.md`, Wave B does not
  dispatch until T01 **and** T02 both merge.

## 2026-07-14 - Doubt-driven-development on the T01 PR: CI was actually red, docs overclaimed

### Context

Before merging the T01 PR (#2), ran a fresh-context adversarial review against the actual task
spec, `interface-contracts.md`, and `agent-factory-plan.md`, with instructions to independently
verify every claim rather than trust the text — including by reading the real GitHub Actions
run, not just the local terminal output this session had already captured.

### Findings and disposition

- **Confirmed, fixed:** the `deny` job in `.github/workflows/ci.yml` was `runs-on: macos-latest`.
  `EmbarkStudios/cargo-deny-action@v2` is a Docker container action — container actions only run
  on Linux-hosted GitHub runners. The actual PR's CI run failed on this job on every push
  ("Container action is only supported on Linux"), while every doc this session wrote —
  including the entry above, `docs/session-log.md`, and the PR body — claimed "cargo deny check:
  PASS." That claim was true for the local Homebrew binary, a different execution path from the
  Docker-based CI job; nobody had checked the real CI run before writing "PASS." Fixed:
  `runs-on: ubuntu-latest` for that job only.
- **Confirmed, fixed:** this project is explicitly bound to `~/hekton`'s Hekton Documentation
  Contract (`CLAUDE.md:18`), which requires `.hekton/agent-run-log.yaml` and
  `.hekton/change-log.yaml` updated every session with a structural/build change — T01 is exactly
  that kind of change (a from-scratch build engine), and neither file was touched. Added
  `RUN-0003`/`CHG-0003`.
- **Confirmed, fixed:** `docs/project-walkthrough.md` still read "No Rust code yet... session is
  design and scaffolding only" and "Implementation: not started" after T01's real code existed —
  the Plain-English Walkthrough Contract requires this file stay current and a dated entry land
  under `docs/walkthroughs/` after a meaningful build session. Updated the walkthrough file and
  added `docs/walkthroughs/2026-07-14-t01-workspace-scaffold.md`.
- **Confirmed, not fixed (flagged instead):** the branch (`gateway/run-20260714-T01`, the ACT/
  engine-gateway dispatch tooling's own naming convention) doesn't match
  `agent-factory-plan.md`'s documented `feat/<squad>-<task-id>-<slug>` convention. Not renamed —
  the branch is already pushed with an open PR, and renaming it costs more (force-push, PR
  re-target) than the mismatch itself does. Recorded as a real, unreconciled gap between two
  repos' git conventions for a future session to resolve, not silently accepted.
- **Verified genuinely correct** (reviewer ran the actual commands, not just read the code): the
  9-crate layout matches the squad ownership table exactly; `[lints] workspace = true` is present
  in every crate and clippy's `-D warnings` genuinely passes with zero suppressions; the `vg`
  binary name and `vg-bench`'s criterion wiring are both correct; no crate is missing.

### Why this matters

This is the second time this session that an independent, fresh-context check (first the lab-
readiness eval's own doubt-driven-development pass, now this one) caught a claim of "all pass"
that wasn't true for the environment that actually matters, not the one that was convenient to
check locally. The pattern is the same both times: verify against the real target (a live
GitHub Actions run here; a 0-byte YAML file there), not against what ran cleanly on this one
machine.

## 2026-07-14 - Doubt-driven-development round 2 (Codex cross-model): six more findings, fixed

### Context

After round 1's fix was independently confirmed (real GitHub Actions run green, all 6 jobs
passing), ran a second, cross-model review (Codex, `codex exec --sandbox read-only`) against the
same T01 PR, seeded with round 1's findings and instructed to focus on what round 1 might have
missed rather than re-confirm it.

### Findings and disposition

- **Confirmed, fixed:** no `cargo test` CI job existed. T01 itself has no logic to test, but the
  job should exist now so T02 onward's first real tests are gated in CI from day one rather than
  retrofitted later. Added a `test` job to `.github/workflows/ci.yml`.
- **Confirmed, fixed:** the Rust toolchain was unpinned — every CI job used
  `dtolnay/rust-toolchain@stable` (a floating channel, not a pinned version), and no
  `rust-toolchain.toml` existed, contradicting this project's own reproducibility standard
  ("documented → scripted → idempotent-ish → logged → reproducible on a blank machine"). Added
  `rust-toolchain.toml` (channel `1.96.1`, matching this machine's installed toolchain) and
  pinned every CI job's `dtolnay/rust-toolchain` ref to the same version.
- **Confirmed, fixed:** `docs/risks.md`'s RISK-0002 mitigation read as if bench-based p95 gating
  were already enforced in CI. It isn't — there's no hot-path code yet to benchmark; only the
  harness compiles. Added a status-update note to the mitigation cell rather than rewriting the
  aspirational target, which is still correct for later tasks.
- **Confirmed, fixed — and the most on-the-nose finding of the whole session:**
  `scripts/check-prereqs.sh`, `docs/local-assumptions.md`, and `scripts/verify-project.sh` still
  didn't check for the Rust toolchain at all, meaning they'd pass clean on a machine with no
  Cargo whatsoever — the *exact* gap flagged in `docs/next-actions.md` on 2026-07-04
  ("`check-prereqs.sh` doesn't check for it either... needs: Rust toolchain... and
  `check-prereqs.sh` updated to check for all three so this doesn't get silently rediscovered
  again") and prepared as an unapplied diff in `~/hekton`'s VeilGremlin dogfood runbook earlier
  this same session — but never actually applied to this repo until this fix. Applied now, to
  all three files, plus made `verify-project.sh` actually run `cargo build --locked && cargo fmt
  --check` rather than only check file presence.
- **Confirmed, fixed:** every crate hardcoded `version = "0.1.0"` on its intra-workspace `path`
  dependencies (added in round 1 to satisfy `cargo-deny`'s wildcard-dependency check). This
  works today but would silently drift from `[workspace.package].version` on the next bump —
  8 places to remember to update by hand. Refactored to the idiomatic Cargo pattern:
  `[workspace.dependencies]` declares each `vg-*` crate once (path + version), and every
  consuming crate uses `{ workspace = true }`. A version bump now only touches two places
  (`workspace.package.version` and `workspace.dependencies`), not every crate.
- **Confirmed, fixed — ironic given the whole exercise:** round 1's own fix to
  `docs/project-walkthrough.md` (correcting the stale "No Rust code yet" claim) introduced a new
  overclaim: "T01 workspace/CI merged 2026-07-14" and "T01 is merged" — the PR was (and still
  is, pending human review) open, not merged. Fixed to "built... PR open — not yet merged."
  Same overclaim did not appear in the dedicated walkthrough doc, only in the updated
  `project-walkthrough.md` sections. Also corrected `README.md`'s stale "Next: Wave A" status
  line (still described T01 as not-yet-started) and `docs/session-log.md`'s unqualified `cargo
  deny check: PASS` line (now notes it was local-only and wrong for the real CI run).

### Why this matters

Two real lessons, not one: first, the same "verify the real target, not the convenient one"
lesson as round 1 (the check-prereqs.sh gap in particular — a fix sitting *written and ready* in
a different repo's runbook for hours before actually being applied here, because nobody closed
that loop until an adversarial review asked "does this script still lie about what's required?").
Second, a fix session can introduce its own overclaim while correcting someone else's — the
"T01 is merged" line proves that doubt-driven-development needs to re-examine its own prior
output, not just the original artifact, on each cycle.

## 2026-07-15 - Task T02: vg-core's shared types, trait seams, and conformance stubs

### Context

With T01 merged, T02 (Squad 0) was next: freeze `vg-core`'s shared types and library API,
define the trait seams every Wave B crate implements against, and provide contract-conformance
test stubs — per `docs/architecture/interface-contracts.md` and `T02.md`'s acceptance
("interface-contracts.md v1 frozen; types compile; conformance test scaffold exists").

### What happened with the dispatch

Retried the real ACT GO-LIVE dispatch mechanism (`dag dispatch T02`) rather than building
directly from the start, since T01's stall was specifically triggered by a stale-toolchain
Bash check that no longer exists. First attempt failed on a transient API connection error
("Connection closed mid-response") — not the same permission-mode issue as T01. Second attempt
ran for the full tool timeout (~10 minutes) and was killed mid-run — but unlike T01, this was
genuine progress being cut short, not an instant stall: the worktree
(`engine-gateway-lab/.worktrees/run-20260715-T02`) had 7 new files and 787 lines of real Rust
written (`types.rs`, `traits.rs`, `api.rs`, `error.rs`, `ids.rs`, `audit.rs`, `conformance.rs`,
plus `tests/conformance_stubs.rs`), with no still-running process and no further file changes —
consistent with the work having actually finished writing but the outer process being killed
before the adapter's trailing steps (verify, output-artifact write, ACT session close) could run.

### Decision

Picked up the work in place rather than re-dispatching from scratch (which would have discarded
real, substantial progress) or discarding it. Verified it independently before trusting it:
`cargo build` (needed to update `Cargo.lock` for three new dependencies — `thiserror`, `uuid`,
`zeroize` — that `--locked` alone can't add), then the actual T02 `verify_command` — `cargo build
--locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo test` — which
passed clean after one `cargo fmt --all` pass (the interrupted run hadn't reached the formatting
step). Read the generated code directly (not just trusted the test pass) given this is the
*frozen* interface contract every later task builds against — found it faithfully matches
`interface-contracts.md`: `Secret` zeroizes on drop with a redacting `Debug` impl; `rehydrate`'s
destination hard-deny gate (`RemoteModelPrompt`/`ObservabilitySink`, regardless of actor) is
implemented for real, not stubbed, since correctly identified as the one piece of T02 logic that
doesn't depend on any Wave B crate; everything else pipeline-related is an explicit `todo!()`
naming the task that wires it (T07/T09/T10), not a silently-incomplete stub.

### Consequences

- This is the second piece of VeilGremlin's real business logic (after T01's empty scaffold) and
  the first real security-relevant invariant with a test proving it.
- T01 + T02 both need to merge before Wave B (five parallel squads) can dispatch, per
  `agent-factory-plan.md`.
- Not yet run: a doubt-driven-development pass on this PR. T01 got two rounds before merging;
  given `rehydrate`'s hard-deny logic is genuinely security-relevant (not just scaffolding),
  the same discipline probably applies here — left as an explicit next action, not assumed.

## 2026-07-15 - Doubt-driven-development on the T02 PR: contract left in DRAFT, a real vault bug

### Context

Ran the same two-round process as T01 (single-model fresh-context review, then Codex
cross-model) against the T02 PR before merging, given `rehydrate`'s hard-deny gate and the
vault/audit conformance helpers are genuinely security-relevant, not just scaffolding.

### Findings and disposition

- **Confirmed, fixed — the most severe finding:** `docs/architecture/interface-contracts.md`
  was never touched by this PR, despite T02's literal acceptance criterion being "interface-
  contracts.md v1 frozen." The doc still read "Status: DRAFT until end of Wave A (Task T02),
  then FROZEN" in present tense. Worse, its illustrative code never defined at least 11 types
  the actual implementation needed and had to invent (`ArtefactHint`, `ArtefactKind`,
  `NodeKind`, `Destination`, `DestinationId`, `Context`, `Input`, `Policy`, `Actor`, `Corpus`,
  `CorpusSample`, `Metrics`), and had two concrete deviations from the real code:
  `EntityType::Custom(String)` wasn't in the doc's literal enum, and `PolicyLayers` used the
  doc's literal `Path` — which isn't valid Rust for an owned field (`Path` is `?Sized`) — where
  the real code correctly used `PathBuf`. The entire point of freezing this document is so Wave
  B squads can build in parallel without reading each other's (or `vg-core`'s) internals; a
  document that's still in draft with 11 missing types defeats that purpose. Fixed: added a new
  §0 (supporting types), reconciled every deviation, flipped Status to FROZEN with today's date,
  and updated the Versioning section to record what changed at freeze time — all in this same
  PR, since nothing has consumed the "frozen" contract yet (Wave B hasn't dispatched), so a
  separate contract-change PR would have been process for its own sake.
- **Confirmed, fixed — the most severe *code* finding:** `MockVault::resolve` in
  `conformance_stubs.rs` **ignored its `ns` parameter entirely** (it was already prefixed
  `_ns`) — a value interned under one `Namespace` would resolve successfully under any other
  namespace. `interface-contracts.md`'s own `Namespace` design exists specifically to scope
  placeholder stability per session/repo/org; silently ignoring that on resolve is a real
  security-relevant defect in the template every Wave B squad (specifically `vg-vault`, Task
  T05) will read as the reference shape. Fixed: `MockVault` now stores the namespace alongside
  each mapping and returns `VaultError::NotFound` on a namespace mismatch (indistinguishable
  from "doesn't exist" to the caller, per the now-documented contract); `VaultStore`'s trait
  doc and `interface-contracts.md` §5 both now state namespace isolation as an explicit,
  required invariant, not an implied one; `assert_vault_roundtrip` now takes a second,
  distinct namespace and asserts cross-namespace resolution fails.
- **Confirmed, fixed:** no reusable `PolicyEngine` conformance helper existed — the hard-deny
  check was only an ad-hoc inline test in `conformance_stubs.rs`, not part of
  `vg_core::conformance` the way every other trait's check was. Added
  `assert_policy_engine_denies_hard_deny_destinations`; the existing test now calls it.
- **Confirmed, fixed:** `assert_audit_event_excludes_raw_values` checked only the literal raw
  string against `{event:?}`'s output — a raw value containing control characters (e.g. a
  newline) renders `Debug`-escaped, so the unescaped literal search would false-negative on
  exactly that leak. Fixed to also check the value's escaped form.
- **Confirmed, fixed:** `assert_masked_pack_excludes_raw_values` only checked `.text`, not
  `.policy_version` (also a `String` field the contract's invariant covers). Fixed; documented
  that `mapping_refs` needs no check since `MappingRef` is type-enforced to hold only an opaque
  `Uuid`, never a real key.
- **Confirmed, fixed:** `assert_detector_contract` didn't validate that returned `Span`s are
  in-bounds (`start <= end <= buf.len()`) — later pipeline code slices by these spans, so an
  out-of-bounds span from a "conformant" detector is a real latent panic/bug source downstream.
  Added the bounds check.
- **Confirmed, fixed:** every conformance helper required `T: Sized` (e.g. `D: Detector`), but
  the library API holds detectors/parsers/vault/audit as trait objects (`Context`, `Policy`) —
  a Wave B crate testing a `Box<dyn Detector>` registry couldn't call these helpers without
  extra wrapper plumbing. Relaxed every bound to `?Sized`.
- **Confirmed, fixed (documentation, not a code change):** the parser conformance test used a
  `MockParser` that can never panic by construction, so it only proved the harness call itself
  didn't crash, not that any real parser is panic-safe. Added a second test running the helper
  against a small battery of adversarial buffers (empty, invalid UTF-8, large/unbalanced) and a
  doc comment on `assert_parser_never_panics` warning that a trivial mock proves nothing about
  a real implementation — squads copying the template need their own equivalent battery against
  actual parsing logic.
- **Confirmed, documented as an inherent contract limitation, not fixed as a bug (matches T01's
  precedent for exactly this class of finding):** `Secret::expose_secret() -> &str` lets a
  caller copy the value out before `Drop` zeroizes it, and `rehydrate`'s own frozen signature
  (`-> Result<String, RehydrateDenied>`) requires returning an owned, non-zeroizing `String` at
  the one exit point that matters — so the zeroize-on-drop guarantee is cosmetic by the
  contract's own shape, not something T02 introduced. Documented explicitly in `traits.rs`,
  `types.rs` (`MaskedPack`'s doc comment no longer overclaims enforcement it doesn't have
  either), and `interface-contracts.md` §5, rather than left as a silent gap.

### Why this matters

The standout lesson here is different from T01's: last time, the bugs were in *scaffolding*
(CI config, docs) — real, but not security-relevant. This time, the most severe finding
(`MockVault` ignoring namespace) is a template defect in code whose entire purpose is to be
copied by four future crates, one of which (`vg-vault`) implements the exact trait this bug was
in. A conformance stub that doesn't test the invariant it's supposed to enforce isn't neutral —
it's actively worse than no stub, since it gives a false sense that the pattern has been
validated. The interface-contracts.md gap reinforces a lesson from the v1 phased plan: a
"frozen" artifact that nobody actually re-reads before building against it (Wave B hasn't
started yet, so nothing depended on it being current — but soon will) drifts from day one
unless updating it is part of the same PR that changes what it describes, not a follow-up.

## 2026-07-15 - Task T03: five deterministic detectors, plus a two-round doubt-driven-development pass

### Context

First genuinely successful unattended `code-implement` dispatch in the factory. The initial
attempt (terse one-line task description) got a clarifying question back from `claude -p`
instead of any code — headless one-shot mode has no follow-up channel, so that question was
the entire session. Root-caused and fixed at the task-spec level (both `.hekton/veilgremlin-dag.toml`,
the source of truth, and the regenerated `.hekton/build-tasks/T03.md`): rewrote the description
with concrete file/module/trait guidance and an explicit "use your judgment, don't ask" instruction.
Re-dispatched; this time claude wrote all five detectors for real (`email.rs`, `phone.rs`, `ip.rs`,
`iban_sortcode.rs`, `entropy.rs`, ~800 lines, plus a criterion bench) matching the prompt closely.
`Cargo.lock` needed a manual regenerate (new `regex`/`criterion` deps never locked) and one clippy
unused-import — both mechanical, fixed in the worktree, not logic changes.

### Findings and disposition (Codex cross-model doubt-pass, gpt-5.5, read-only sandbox)

Reviewed against the `Detector` trait contract (span validity, no panics, determinism,
confidence bounds) with an explicit security lens (false negatives in a PII/secret detector
mean real leaks). 9 findings, reconciled:

- **Fixed — real bug, not just a documented limitation:** `ip.rs`'s IPv6 pattern's own comment
  claimed it "deliberately does not attempt" IPv4-mapped addresses (`::ffff:192.0.2.1`), implying
  a clean miss — but the generic hex-group alternatives actually produced a truncated PARTIAL
  match (`::ffff:192` out of the full address), which is worse than a miss for a redaction tool:
  most of the real address (`.168.1.1`) was left sitting unredacted next to a redaction marker.
  Added an explicit IPv4-mapped alternative, ordered first (this crate's regex alternation is
  leftmost-first, not leftmost-longest, so ordering matters for which alternative wins).
- **Fixed — real gap in the entropy detector's core purpose:** `is_token_byte` excluded common
  password special characters (`!@#$%^&*`), so a genuinely high-entropy password like
  `aB3!xY7@qR2#nM8$pL5%zK` was silently split into sub-20-byte fragments at every special
  character, each individually below the length floor — a systematic miss of a realistic secret
  shape, not a hypothetical one, for the one detector whose whole job is catching secrets with no
  fixed format. Added those bytes; deliberately did NOT add `:`/`;`/`(`/`)` (too common as genuine
  field/prose/timestamp delimiters — see the next item).
- **Found only because the IPv6 fix now works correctly, then fixed:** the IPv4-mapped fix exposed
  that the embedded `192.168.1.1` inside a matched `::ffff:192.168.1.1` ALSO independently matches
  the standalone `ipv4_pattern()`, producing two overlapping findings for one real address.
  `detect()` now drops an ipv4 finding when it's fully contained inside an ipv6 finding's span.
- **Confirmed, documented as an accepted residual, not fixed:** colon-delimited compound secrets
  (`user:token`) are still split and individually missed — `:` is too common a genuine delimiter
  (timestamps, URLs, "key: value" idioms) to safely add to `is_token_byte` without risking spurious
  merges elsewhere; the narrower punctuation-only fix above was judged the better trade-off.
- **Confirmed, documented as an accepted residual, not fixed:** a phone number like
  `+1 (415) 555-2671` only matches from the parenthesised group onward, leaving the leading `+1`
  (just a country code, not the subscriber number) outside the redacted span. Low severity;
  fixing it safely would need a real regex restructure, judged not worth the risk for this gap.
- **Already documented in the code's own comments, not new bugs:** bare unseparated phone numbers
  and 6-digit sort codes in JSON/API shapes (`{"phone":"02079460958"}`) are excluded by design
  (phone.rs/iban_sortcode.rs already discuss this ambiguity); `YYYY-MM-DD` dates matching the phone
  heuristic is the exact example phone.rs's own docstring already calls out.
- **Real but out of `vg-detectors`' own scope, not fixed here:** independent detectors (e.g. IP vs
  phone) can produce overlapping findings for the same span with different entity types when a
  value's digit/separator shape satisfies more than one detector (`192.168.100.42` reads as both).
  Cross-detector arbitration isn't part of the `Detector` trait's contract; likely T04/T09 territory
  (typed-placeholder keying / cross-cutting policy), not a `vg-detectors` bug.
- **Refuted:** IBAN mod-97 checksum validation is NOT missing scope — `docs/architecture/work-breakdown.md`
  assigns "Luhn/mod-97 checksum validators" specifically to **T04**, not T03; `iban_sortcode.rs`'s own
  comment correctly describes this as regex-only by design, matching the actual task breakdown, not
  the broader aspirational spec the reviewer initially read it against.

### Round-2 verification pass (same Codex model, fresh session): checked the three fixes above for new bugs

Found 5 more, none requiring a code change — reconciled:

- **Confirmed, accepted, documented (2, both caused by the `is_token_byte` fix above):**
  including `@` makes an ordinary email (`jane.doe@example.com`) sit right at the entropy
  threshold, so it can ALSO get tagged `Secret` alongside `EmailDetector`'s own more specific
  `Email` finding for the same span — a precision cost (over-flagging), not a leak, and
  consistent with this detector's own stated philosophy of flagging *any* sufficiently
  random-looking token. More seriously: `@` can also merge a real secret with a long
  low-entropy suffix (e.g. Basic-Auth-in-URL style `<secret>@internal.example.com`) and
  dilute the merged token's entropy below threshold — a genuine false-negative shape, but
  removing `@` would just reopen the original multi-special-character password gap this fix
  closed. Judged: keep `@`, document both as accepted trade-offs (comments added to
  `entropy.rs`), don't chase further — each further tweak to a coarse byte-classification
  heuristic trades one realistic secret shape for another rather than strictly improving
  coverage.
- **Confirmed, pre-existing (not introduced by this session's fixes), documented, not fixed:**
  `=` is a token byte, so a long low-entropy key name merged with `=<secret>` can similarly
  dilute a real secret's entropy below threshold. Same class of trade-off as above.
- **Confirmed, accepted, documented:** the IPv4-mapped-IPv6 fix only covers the specific
  `::ffff:` prefix (the common, still-valid form); rarer/deprecated embedded-IPv4 shapes
  (`2001:db8::192.168.1.1`, an IPv4-compatible form RFC 4291 itself deprecated in 2006; a
  malformed `::ffff:0:192.168.1.1`) still produce the same partial-match behavior the fix
  was meant to close, just for a rarer input. Judged not worth generalizing further, matching
  the original scope call on embedded-IPv4 notation.
- **Same already-documented class as the first pass, not a new issue:** the phone detector
  matching a dotted IPv4 address (`10.10.10.10`) as a phone number is the identical
  cross-detector-overlap class already documented above (IP vs phone), independently
  rediscovered rather than a distinct new finding.
- **All three fixes verified correct** for their stated purpose (regex alternation ordering,
  overlap-dedup logic, tokenizer broadening) — no regressions found in the fixes themselves,
  only in their interaction with adjacent, unrelated inputs (emails, `=`-delimited keys),
  which is the residual surface documented above, not a defect in the fix logic itself.

Two full cross-model review cycles on this one crate is the stop point (per the
doubt-driven-development skill's own guidance: escalate rather than grind a third cycle
alone) — the remaining residual surface is a property of a coarse, deliberately simple
byte-classification heuristic, not a bug queue to keep chasing.

### Why this matters

Two genuinely new, generalizable lessons from this one task: (1) headless one-shot dispatch is
extremely sensitive to prompt specificity — the exact same task, described tersely, produced a
clarifying question and nothing else; described concretely with file/module/trait guidance, it
produced ~800 lines of real, well-tested, well-documented work on the first attempt. (2) A
security detector's own documentation can be *wrong* about its own limitations in a way that
matters: this code's comment said a case was "not attempted" (implying a safe miss) when it was
actually mishandled (an unsafe partial match) — the fix isn't just adding a feature, it's
correcting a false safety claim the code made about itself.

## 2026-07-16 - Dogfooding plan (Codex) + a real CI-enforced latency gate + a real detector census

### Context

Reviewed the remaining fan-out (Wave B: T04/T05/T05b/T06/T08; Wave C: T07/T09; Wave D: T10/T11)
against the product's actual goal — mask PII by design, but be an invisible control with
trading-system-grade latency discipline (tail-latency awareness and real CI enforcement, not a
literal microsecond target for a human-interactive hook). Two concerns: the p95 latency budget
was compile-check-only in CI until T10 wires up baseline management, and each Wave B crate is
built and tested in isolation against the frozen contract with no cross-crate integration
exercised until T07, several tasks away.

### Decision 1: a real, CI-enforced latency gate now, not deferred to T10

Added `crates/vg-detectors/tests/latency_gate.rs` — a plain `#[test]` (runs on every PR via the
existing `cargo test` CI job, zero new CI config) asserting the whole detector suite's p95 stays
within 4x the interface contract's 25ms budget across 200 iterations. Deliberately coarse: a
tight bound would be flaky on noisy shared CI runners; this is a regression backstop (catches an
accidentally-uncompiled regex, a hot-path allocation, an O(n²) detector), not precise tracking —
Task T10 still owns real p95/p99 baseline management. Independently corroborated by a Codex
planning pass (below) reaching the identical design ("plain Rust test... not Criterion as the
hard gate... generous margin") before seeing this implementation.

### Decision 2: cross-crate integration requirements added to Wave B/C task specs now

`.hekton/veilgremlin-dag.toml` (source of truth) and `docs/architecture/work-breakdown.md`
updated:
- **T04** (placeholder/HMAC keying) must integration-test against real `Finding`s from
  `vg-detectors::all_detectors()` (T03, already closed), not only mock values.
- **T08** (parsers) must integration-test its real `Span` output against
  `vg-detectors::all_detectors()` on a realistic fixture, and explicitly record whether the fact
  that all five T03 detectors currently ignore their `spans` parameter (confirmed 2026-07-16,
  `_spans` in every `detect()` signature) is an accepted stage-appropriate gap or a real one —
  don't let it go unnoticed until T07.
- **T09** (CLI/hooks) gained an explicit UX-latency acceptance criterion: a human runs a real
  interactive session with the hooks wired in and confirms no perceptible friction, recorded
  honestly in this file — the first point the "invisible control" goal is even testable.

### Decision 3: a Codex planning pass on dogfooding, ahead of implementing any of it

Asked a fresh Codex session (read-only, planning only, no code) to plan how to dogfood VeilGremlin
incrementally as it's built, rather than only at T10. Its plan (full transcript not reproduced
here) independently converged on several of the decisions above, and added:
- The highest cross-crate integration risks beyond T03↔T08: T04↔T05 (placeholder/vault behavioral
  disagreement — "types compile, behavior disagrees"), T06↔T08 (policy only sees `ArtefactHint`,
  parser returns `ArtefactKind` — a `.env` file could parse fine and never get blocked until T07
  notices), T05b↔everything (raw-value leak into audit events).
- A concrete, immediately-actionable step requiring no new pipeline: a read-only "detector
  census" — run the already-built `vg-detectors` against real Hekton artifacts to surface real
  edge cases and real-world latency, years before T10's formal eval harness exists. Explicit
  design constraint carried into the implementation: never print or store matched values, only
  counts/spans/detector-IDs/latency/paths.
- A concrete, Hekton-specific edge-case list the current 5 detectors would plausibly miss:
  absolute local paths, SSH aliases, API key *variable names* (vs. values), broker-auth token
  metadata fields, operational IDs (run-IDs, RISK-IDs, branch names — "no detector for factory
  control-plane identifiers"), hostnames (`EntityType::Hostname` exists in the type system, no
  detector implements it yet), and commit-SHA/HMAC ambiguity.
- Its structural recommendation: treat dogfooding and latency discipline as ongoing Wave B/C
  cross-cutting work, not something T10 owns alone — T10 becomes the *formal* eval harness, not
  the *first* time real data touches the product.

### Decision 4: ran the census for real — a significant, evidenced precision finding

Built `crates/vg-detectors/examples/census.rs` per the design above and ran it against both
VeilGremlin's own repo and `engine-gateway-lab`'s (197 files, ~1.1MB, real docs/YAML/logs — not
synthetic fixtures). Results: 11.2ms total scan time (0.057ms/file average) — confirms the
detectors are fast on real content, no latency surprise. But the finding-count breakdown is a
real precision concern, not a synthetic one:

```
email          findings=12
entropy        findings=2468   <- dominant, and mostly false positives (see below)
iban-sortcode  findings=28
ip             findings=69
phone          findings=783    <- also mostly false positives (see below)
```

Manually verified (not via the census tool, which deliberately never surfaces matched text) that
the entropy detector's dominant hit class is exactly the Codex-predicted "operational IDs" gap:
Hekton's own `run-YYYYMMDD-EG-NNN` run identifiers (e.g. `run-20260608-EG-012`) are ~19-20 bytes
of mixed digits/letters/hyphens — precisely the shape `is_token_byte` + the 20-byte floor +
3.5 bits/byte threshold was tuned to catch for real secrets, with nothing in the current design
to distinguish "high-entropy secret" from "high-entropy but totally benign structured
identifier." The phone detector's high count is the already-documented date/ID-as-phone
ambiguity, now empirically quantified at real scale rather than a single hand-built test case.

**This matters concretely for Task T06 (policy) and T07 (pipeline):** if masking goes live with
today's detector precision unchanged, the product would redact the overwhelming majority of
routine operational identifiers, dates, and structured IDs in ordinary agent-factory documents —
directly working against the "invisible control" goal by making output needlessly noisy. This is
a genuine design question, not a quick regex fix: whether to (a) add an allowlist/exclusion
mechanism at the policy layer for known-safe structured shapes, (b) tighten the entropy/phone
heuristics further (with the same risk/reward trade-offs already documented for T03), or (c)
accept and formally measure this as part of T10's already-tracked `false_positive_rate` Go/No-Go
metric. Not decided here — flagged as a real, evidenced open question for whoever builds T06/T07,
not guessed at. `census.rs` is kept in the repo (an `examples/` binary, not part of the normal
build/test path) so this can be re-run cheaply as each Wave B/C task lands, per the Codex plan's
ladder (detector-only now → parser+detector after T08 → stubbed mini-pipeline after
T04/T05/T06/T05b → real `mask()` after T07 → real dogfood after T09).

## 2026-07-16 - Fixed the entropy/phone false-positive finding (hybrid: detector patch now, T10 stays the gate)

### Context

Ran the census's open question (allowlist? tighter heuristics? defer to T10?) through a
Codex planning pass before deciding, per the human's request. Codex read the actual frozen
`PolicyEngine`/`Detector` contracts and the real detector code before answering, and
recommended a hybrid: fix the two dominant detector-level false-positive classes now
(`EntropyDetector`, `PhoneDetector`), keep Task T10 as the formal precision/recall gate,
and explicitly deprioritized a per-finding policy-layer allowlist for now — the frozen
`PolicyEngine::classify_artefact`/`classify_entity` contract has no per-finding-shape hook,
so building that properly is a real cross-cutting contract change, not a quick add, and a
regex-based allowlist would itself be a potential attacker-controlled bypass surface if not
carefully scoped. Human approved the hybrid.

### What was actually fixed

**`PhoneDetector`**: added `looks_like_iso_date`, excluding matches shaped like a strict
`YYYY-MM-DD`/`YYYY.MM.DD` calendar date (plausible year/month/day) rather than a phone
number. Narrow and generic — does not exclude arbitrary grouped numbers, only the exact
date shape.

**`EntropyDetector`**: added `is_structured_identifier`. **This required a correction
mid-session**: the first version assumed Hekton's own `run-YYYYMMDD-EG-NNN` run IDs were
the dominant false-positive shape (matching the census's original hypothesis) and only
excluded 3+-segment hyphen-delimited tokens with short alpha/bounded-digit segments.
Measuring it against real `engine-gateway-lab` content (via a temporary, never-committed
local debug print — not the census tool, which by design never surfaces matched text) showed
this removed only 1 of 1849 entropy findings on that fixed corpus. The actual dominant
classes were **file paths** (`scripts/gateway-run.sh`, `.hekton/risk-register.yaml`) and
**snake_case/kebab-case identifiers** (`requires_confirmation`, `local-coding-harness`) —
`is_token_byte` treats `/`, `.`, and `_` as part of a token, so a whole path or identifier is
scored for entropy as one blob, and the character-class mix clears the threshold even though
every piece is an ordinary word. Corrected to split on the token's own internal delimiters
(`/`, `.`, `_`, `-`) and exclude when every resulting segment is purely alphabetic (any
length) or purely numeric (<=8 digits) — catches paths/identifiers/operational-IDs
generically, not via a Hekton-specific dictionary.

**Accepted residual, not fixed**: a real secret that happens to be a dictionary-word
passphrase joined by delimiters (e.g. `correct-horse-battery-staple`) would also be excluded
— indistinguishable from a real identifier without a semantic/dictionary check this detector
doesn't have. A secret whose segments mix letters and digits (the vast majority of real
base64/hex/API-key shapes) is unaffected.

### Measured impact (isolated before/after on identical, untouched `engine-gateway-lab`
content — not the confounded combined-repo numbers, since this session's own doc edits also
grew VeilGremlin's own corpus)

```
                 before   after
entropy          1849     182    (-90%)
phone            618      54     (-91%)
```

Latency unaffected (10.19ms / 197 files across both repos, vs. 11.2ms before). Full
`cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check &&
cargo test` green throughout, including 5 new tests (2 phone, 3 entropy) built directly from
the real false-positive examples found.

### Decision

Ship the corrected detector-level fix now; T10 remains the formal `false_positive_rate`
gate for the residual ~10% and any future drift. The mid-session correction is itself
evidence for why "measure on real content, don't just theorize" (the whole point of the
census/dogfooding effort) matters — the first, unmeasured version of this exact fix would
have shipped almost no real improvement.

## 2026-07-17 - Added a build log, distinct from the session log

### Context

Human asked for a build log tracking "everything we're doing in this repo," similar to the
Hekton Workshop Gremlin's `docs/build-log/YYYY-MM-DD-*.md` convention and, to a degree, the
Hekton Field Journal. Since VeilGremlin already has a comprehensive internal
`docs/session-log.md`, the open question was what a build log adds that isn't already
covered — asked the human directly rather than assume scope: lightweight dated docs only,
a full publishable Astro-on-Pages site (like `terminal-velocity`/`borrow-native`), or just
writing about VeilGremlin from the Field Journal repo instead of building anything new here.
Human chose the lightweight option, explicitly to start.

### Decision

Added `docs/build-log/` — dated, deliberately written entries (`YYYY-MM-DD-<slug>.md`),
one per real event (a decision, a failure, a fix, a surprising result), not a mechanical
summary of `docs/session-log.md` and not one per session. Backfilled 7 entries covering the
repo's history to date: the initial scaffold, the coderturtle-to-dermdunc ownership move,
T01, T02, T03, the fan-out/latency-gate/census round, and the entropy/phone false-positive
fix (including its own mid-session self-correction, which became the best story of the
lot). Wired into the standing documentation contract: `AGENTS.md`, `CLAUDE.md`, and
`CODEX.md` all gained a rule to add an entry for future sessions with a real event, and
`AGENTS.md`'s traceability table now lists `docs/build-log/` alongside the existing
artefacts. Linked from `README.md`'s documentation table and `docs/project-walkthrough.md`
(which also got its own overdue refresh — it had gone stale claiming T01's PR was still
open and zero business logic existed, both no longer true).

No Astro site or GitHub Pages deploy yet — this repo is already public, so the files are
readable directly on GitHub without one. A site can be built later if the practice earns it
(see `docs/next-actions.md`), following the Workshop Gremlin's Build-log/Pages publisher
agent pattern rather than inventing a new one.

## 2026-07-17 - T04 typed-placeholder and HMAC keying

### Context

Headless one-shot dispatch (no follow-up channel — see `.hekton/veilgremlin-dag.toml`'s T04
entry). Implemented the formula `VaultStore`'s trait doc already names as authoritative:
"stable placeholder via salted HMAC over `(canonical(value), ty, ns)`". Several exact
naming/design choices were left to judgment per the dispatch instructions; recorded here
rather than asked, per that instruction.

### Judgment calls

1. **Case-folding in `canonicalize` is type-specific, not blanket.** Whitespace
   trim/collapse applies to every value; letter-case folding only applies to
   `Email`/`Hostname`/`InternalIp`/`Iban`/`SortCode`/`Postcode`/`TraceId` — types where case
   doesn't carry identity information. Secret-shaped types (`Password`, `PrivateKey`,
   `Secret`, `AccessToken`, `ApiKey`) and free-text/identifier types keep their case:
   lower-casing a secret would silently treat two genuinely different values as the same
   one, which is a correctness bug for exactly the class of value this tool most needs to
   keep distinct.

2. **HMAC salt is caller-supplied (`&[u8]`), not a hardcoded constant.** `vg-core` doesn't
   own persistent secret-key storage — `vg-vault` (Task T05) wraps the real key via the OS
   keychain per `interface-contracts.md` §5. A compiled-in salt would make "salted" a no-op
   (every install would key identically); `Keyer::new`/`placeholder_key` both take the salt
   as an argument so T05's `VaultStore::intern` impl can supply its own keychain-backed key.

3. **HMAC message uses an explicit `0x1F` (ASCII Unit Separator) between the canonical
   value, the entity-type tag, and the namespace tag**, rather than plain string
   concatenation — otherwise `value="ab", type="c"` and `value="a", type="bc"` would hash
   identically. Covered by
   `placeholder_key_has_no_naive_concatenation_collision` in `keying.rs`'s own tests.

4. **Ordinals are scoped per `(Namespace, EntityType)`, not globally per type.** The task
   description's wording ("the first time a distinct key is seen for a given `EntityType`
   within a `Namespace`...") was read as: each namespace gets its own independent
   `EMAIL_001, EMAIL_002, ...` sequence, matching `README.md`'s framing of placeholders as
   stable *within* a namespace and the acceptance criterion's own phrasing ("same value ->
   same placeholder within namespace"). A single shared global-per-type counter across
   unrelated sessions/repos/orgs seemed both harder to reason about for a user and not
   clearly what "within a `Namespace`" was asking for.

5. **Luhn/mod-97 validators are exposed as pure functions, not wired into `display`
   construction to synthesize a fake-but-checksum-valid card number or IBAN.** The task
   description's item 4 says these should let "a placeholder's own display value ... be
   checked (or constructed) to remain checksum-valid." Read literally, "constructed" could
   mean generating a full synthetic-looking replacement number. That would conflict with
   **ADR-005** (this file, 2026-06-30, frozen before this task): "Masking = typed
   placeholders, not synthetic values" — explicitly chosen over format-preserving fake data
   for Phase 1, for transparency/debuggability/audit reasons. Given the conflict, the
   frozen, earlier ADR was treated as authoritative: `display` stays `TYPE_TAG_NNN`
   (`EMAIL_001`, `ACCOUNT_ID_014`, matching `README.md`'s own example), and
   `luhn_is_valid`/`iban_mod97_is_valid` are exposed as standalone, independently useful
   validators (e.g. for a future detector-confidence booster or masking-quality check) —
   satisfying the "checked" half of item 4 without the "constructed" half's synthetic-value
   implication.

6. **Cross-crate integration test added per the 2026-07-16 acceptance-criterion addendum**
   (`crates/vg-core/tests/keying_integration.rs`): runs `vg-detectors::all_detectors()`
   (Task T03, already merged) over a fixture built from literal strings reused verbatim from
   each detector's own already-passing unit tests (so a detector regression shows up as a
   failed coverage assertion here, not silent no-op test), then feeds the real `Finding`
   spans/values through `Keyer`. Required adding `vg-detectors` as a **dev-only** dependency
   of `vg-core` (`crates/vg-core/Cargo.toml`) — not a real cycle in the normal build graph,
   since `vg-detectors`'s own (non-dev) dependency on `vg-core` is the only edge that matters
   for building either crate for real; Cargo resolves this pattern (a crate's own test
   binary dev-depending on one of its dependents) without issue.

### New dependencies

Added `hmac = "0.12"` and `sha2 = "0.10"` to `vg-core` (both RustCrypto crates, MIT/Apache-2.0
dual-licensed, matching `deny.toml`'s existing allowlist — same license family as the
already-present `zeroize`/`uuid`/`thiserror`).

### Validation status

**Not run in the dispatch session** — the sandboxed headless environment blocked all
`cargo`/`rustc` invocations pending an approval that never arrived (every attempt returned
"this command requires approval" with no prompt reachable in a one-shot headless run —
plain shell commands like `find`/`grep`/`git status` worked fine, so this looks like a
policy specifically gating toolchain execution, not a blanket Bash block). The module was
written and hand-traced carefully against the `hmac`/`sha2` crates' documented APIs and
existing crate conventions, including manual step-by-step verification of the Luhn and
mod-97 test vectors (`79927398713`/`79927398714`, `GB29NWBK60161331926819`/`...818`) by
hand.

**Verified post-dispatch, during PR review:** `cargo build --locked` compiled clean on the
first real attempt (only Cargo.lock needed regenerating for the two new dependencies,
`hmac`/`sha2`). `cargo clippy --workspace --all-targets --locked -- -D warnings` found one
trivial finding — a newer lint suggesting `.is_multiple_of(10)` over `% 10 == 0` in the
Luhn checksum — fixed. `cargo fmt --check` found routine reformatting (never run against
this file before) — applied. Full suite then green: 32 `keying` unit tests plus 5 real
cross-crate integration tests plus the existing 7 `vg-core` conformance tests, all passing,
including every hand-traced Luhn/mod-97 vector turning out correct. Confirms the "hand-trace
carefully, document what's unverified, don't guess" discipline worked as intended here — a
harder case than T01's silent stall or T03's clarifying question, since this time the agent
had no way to reach a compiler at all and still produced fully correct, review-ready code.

### Doubt-driven-development (Codex cross-model)

Passing tests don't substitute for reviewing logic the tests don't happen to cover, especially
in security-relevant keying code, so a fresh-context Codex review ran against the full diff
(`keying.rs`, `keying_integration.rs`, `lib.rs`, `Cargo.toml`) plus the frozen `VaultStore`
contract and this task's acceptance criteria, before this went to the human tollgate.

**Found and fixed (real bugs):**

1. **`EntityType::Custom` collision.** `Custom("foo-bar")`, `Custom("foo_bar")`, and
   `Custom("foo bar")` — three different policy-dictionary classes — all upper-snake-cased to
   the identical display tag `CUSTOM_FOO_BAR`, which was also being reused as the HMAC's
   entity-type input. Same value under three different `Custom` classes silently keyed
   identically, a direct violation of "type-sensitive by construction." Fixed by splitting a
   new `type_tag_for_keying` (embeds the raw, unmodified dictionary name) from the existing
   `type_tag_for_display` (cosmetic formatting only, safe to rename later).
2. **Compact vs. spaced/hyphenated IBAN and sort code keyed differently.** `canonicalize`
   collapsed whitespace but never stripped it, so the same real IBAN in its compact vs. spaced
   form (both of which `vg-detectors`' own IBAN detector already recognises as the same value)
   produced two different placeholders — a direct violation of this task's own "same value ->
   same placeholder within namespace" acceptance criterion. The same class of issue applied to
   sort codes and phone numbers. Fixed via a new `strip_cosmetic_separators` step, scoped
   narrowly to `Iban`/`SortCode`/`Phone` (not `Postcode`/`InternalIp`/`Hostname`, whose
   separators are structurally meaningful, not cosmetic).
3. **`PlaceholderKey`'s `Debug` impl leaked the real HMAC hex.** Any incidental `{:?}`
   formatting (a test failure message, a log line) would print the actual vault lookup key.
   Fixed to redact, matching `Secret`'s own `Debug` impl.

**Found and fixed (documentation only):** `iban_mod97_is_valid`'s doc comment now states
explicitly that it checks the mod-97 checksum only, not country-specific length or BBAN
structure — a caller could otherwise mistake it for a full IBAN format validator.

**Found, valid, not fixed here — a real cross-task interface gap for Task T05:** `Keyer`'s
per-`(Namespace, EntityType)` ordinal counters are session-only, in-memory state. `vg-vault`
(T05) doesn't exist yet, so this can't be fully resolved in T04, but it is a real requirement
T05 must satisfy: **when `VaultStore::intern` wraps a `Keyer`, it must reseed each namespace's
ordinal counters from the vault's own persisted records at construction time**, or two
different scenarios both produce a real bug — (a) a fresh `Keyer` after a process restart hands
out `EMAIL_001` to whatever value it sees first, which may not be the value the persisted vault
already calls `EMAIL_001`, and (b) if the vault ever holds more entries than the in-memory
counter has seen this session, a genuinely new value could collide with an already-assigned
display string. Recorded here and in `docs/next-actions.md` so T05 can't silently skip it.

**Found, valid, accepted as a trade-off — not fixed:**

- `Keyer::new` accepts an empty or low-entropy salt with no validation. Left unvalidated
  deliberately: this crate's own tests intentionally use short salts (`b"salt"`), and "salt
  strength" is a deployment/T05 concern (the real salt comes from the OS keychain per
  `interface-contracts.md` §5) that `vg-core` has no principled basis to enforce a threshold
  for.
- `Keyer::key_for` panics on a poisoned mutex rather than returning a `Result`. No code path
  currently panics while holding the lock, so this isn't reachable today, but if `vg-vault`
  wires `Keyer` directly into `VaultStore::intern` (which returns `Result<Placeholder,
  VaultError>`), a poisoned mutex would turn a recoverable error path into a process panic.
  Flagged for whoever builds that integration point in T05, not fixed speculatively against a
  failure mode that can't currently occur.
- The session cache is unbounded. Acceptable for Phase 1's in-process library/CLI lifetime
  (bounded by one masking invocation, not a long-running process); would need revisiting if a
  future daemon mode (deferred, see `interface-contracts.md`'s intro) keeps one `Keyer` alive
  indefinitely.

**Verification after fixes:** full workspace verify chain green — 31 `keying`-specific unit
tests (5 new, one per fixed/verified finding above) within `vg-core`'s 37-test unit binary (the
other 6 predate T04), 5 cross-crate integration tests, 7 conformance tests, 46 detector tests,
1 latency-gate test, all passing. Stopped at one cross-model cycle:
the findings were substantive and all got fixed or explicitly classified (contract requirement
for T05, or an accepted trade-off), not the "diminishing returns" pattern that would call for a
second round.

## 2026-07-17 - Merged T04 (PR #9); vault sync; fixed a real bug in sync-mirror-to-vault.sh

### Context

PR #9 (T04's implementation, folded together with the task-spec-guidance and T05-dependency
changes originally on the now-closed PR #8) merged to `main`. Human then asked to sync the
repo-local mind-palace mirror to the live Obsidian vault, and to confirm the build log is
genuinely tracking how/why/what this project is building, the way it will need to when
delivered as part of the final project (matching the coderturtle workshop build-log
practice).

### Vault sync

Backed up the vault (`~/hekton/scripts/backup-obsidian-vault.sh`) before syncing, per Hekton's
standing policy. Ran `scripts/sync-mirror-to-vault.sh`: it correctly copied the refreshed
`session-log.md` onto disk, but its own `git add` line staged nothing at all, silently. Root
cause: the `git add` command listed `index.md`, `decisions.md`, and `session-log.md` — but
`decisions.md` is deliberately never mirrored to the vault (per this same script's own
`SUMMARY_FILES` array and boundary-rule comment, only 2 files). `git add` aborts the *entire*
command when any one pathspec doesn't match a file, so nothing got staged, and the subsequent
`git commit` silently did nothing — while the script still printed a false-positive "Committed
scoped vault update" message, since it never checks either command's exit status. Fixed the
script (`git add` now lists only the two files `SUMMARY_FILES` actually mirrors) and manually
staged/committed the pending vault update.

### Build-log coverage audit

Reviewed `docs/build-log/` against the actual work done to confirm it holds up as a real
delivered artifact, not just a backfilled list. Found one real gap: T04's own build-log entry
(written by the dispatching agent, describing its no-compiler-available constraint) predates
the subsequent Codex doubt-pass and never mentions the three real bugs that pass found, or the
minor self-report inaccuracy the fact-check pass caught. Added a second T04 entry,
"Three bugs a compiler would never have caught," covering that story specifically — matching
this log's own rule that a clean success is worth one line, but a caught wrong assumption is
the actual story worth telling in full.

### Decision

The build log is confirmed to cover, for every material task to date (scaffold through T04):
what was built, why (the real decision/tension behind it), and how (the actual mechanism —
dispatch failure modes, review rounds, fixes). It lives directly in this public repo's
`docs/build-log/`, so it ships with the project automatically — no separate publish step
required, unlike the Workshop Gremlin's Astro/Pages pattern, since VeilGremlin doesn't (yet)
have or need a standalone site for it. Re-audit this coverage after each future task, not just
at the end.

## 2026-07-17 — T05 (`vg-vault`): SQLCipher-backed `VaultStore`, keychain-wrapped key, Keyer reseed

### Context

Task T05 implements `vg_core::traits::VaultStore` in `crates/vg-vault` (previously an empty
stub). The dispatch fixed several choices (SQLCipher via `rusqlite`, OS-keychain key wrap via
`keyring`, `Secret` for raw values, call into `vg-core`'s `keying.rs` rather than reimplementing
HMAC, namespace isolation on `resolve`, TTL purge, cached prepared statements) and asked that any
remaining ambiguity be resolved by best judgment and recorded here rather than by asking (headless
one-shot, no follow-up channel). As with T04, no Rust toolchain/compiler was reachable in the
dispatch environment (`cargo` is gated behind interactive approval that a headless run can't
satisfy), so the code is written against the verified `vg-core` interfaces and left for a
compile/clippy/test pass at PR-review time.

### Decisions and recorded assumptions

1. **Added a `Keyer::seed_ordinal` method to `vg-core`'s `keying.rs` (the one cross-crate edit).**
   The dispatch's hard requirement — the vault's `Keyer` must have its per-`(Namespace,
   EntityType)` ordinal counters reseeded from persisted `mapping` rows at construction — was
   impossible against T04's `Keyer` as merged: its ordinal state is private with no reseed hook.
   `Keyer` is an internal T04 helper, **not** part of the frozen `interface-contracts.md` surface
   (§0–§8), so adding an additive, monotonic, idempotent `seed_ordinal(&self, ns, ty, max_ordinal)`
   is not a contract change. This is the minimal, correct way to satisfy the requirement without
   the vault reimplementing ordinal assignment or display formatting (`type_tag_for_display` is
   private to `keying.rs`).

2. **`intern` computes the lookup key with `placeholder_key` (non-mutating), and only calls
   `Keyer::key_for` (which mints an ordinal) once a value is confirmed new to the DB.** Calling
   `key_for` first would advance the ordinal counter for values that already have a persisted
   ordinal, causing gaps and divergence after a restart. The DB (keyed by the HMAC hex) is the
   durable source of truth for "already interned?"; the reseeded `Keyer` is used only to assign
   the next ordinal/display for genuinely new values.

3. **"Cache prepared statements on the struct" is implemented via rusqlite's
   `Connection::prepare_cached`, not a self-referential `Statement` field.** A `rusqlite::Statement`
   borrows its `Connection`, so storing both in one struct is self-referential and not idiomatic;
   `prepare_cached` maintains an LRU of compiled statements on the `Connection` itself, which is
   exactly the "don't re-prepare per call on the hot lookup path" intent. The `Connection` is behind
   a `Mutex` (it is `Send` but `!Sync`; `VaultStore` requires `Send + Sync`).

4. **No app-level second cipher on the value column.** SQLCipher encrypts the entire DB file with
   AES-256 (`interface-contracts.md` §5), so the raw value stored in a column is encrypted at rest
   by that layer. Adding a separate application-level encryption of the value column would be
   redundant defense-in-depth for Phase 1 and was not done; the schema stores the value in a column
   protected by the SQLCipher-encrypted, keychain-wrapped DB.

5. **The keying salt is a per-install random 32 bytes stored in a `meta` table inside the encrypted
   DB** (generated on first open), not a compiled-in constant — a hardcoded salt would make "salted"
   a no-op across installs (per `keying.rs`'s own note). It is protected by the same SQLCipher/keychain
   layer as the values.

6. **`Vault::open_with_key(config, key)` exists alongside `Vault::open(config)`.** `open` fetches the
   DB key from the OS keychain (generating one on first use); `open_with_key` takes a caller-supplied
   key and bypasses the keychain. This is the seam the test suite uses (temp-file DB + fixed key) so
   tests never touch or mutate the real macOS keychain, and it is also the hook for a future
   alternative key custodian. The "never persisted plaintext" guarantee is a property of the `open`
   keychain path; with `open_with_key` the caller owns the key's secrecy.

7. **`resolve` reports both a namespace mismatch and an expired mapping as `VaultError::NotFound`**,
   never distinguishing "exists in another namespace" from "doesn't exist" (the §5 isolation
   contract, checked by `assert_vault_roundtrip`). Every `resolve` attempt (success or not) appends
   one row to the append-only `demask_event` table, holding only the opaque `mapping_ref` and
   namespace — never the value.

8. **`rusqlite`'s `bundled-sqlcipher-vendored-openssl` feature** was chosen over `bundled-sqlcipher`
   so the build vendors both SQLCipher and OpenSSL from source rather than depending on a system
   OpenSSL with dev headers (macOS ships LibreSSL without them). This makes the crate build heavier
   (needs a C toolchain + perl) but self-contained; flagged as a build-environment consideration for
   the PR-review compile pass.

9. **`EntityType`/`Namespace` are stored in structured, round-trippable columns** (`ns_kind`/`ns_id`,
   `entity_kind`/`entity_custom`) owned by `vg-vault`'s `codec` module — deliberately *not* the
   private one-way keying tags from `keying.rs` — so the construction-time reseed can reconstruct the
   real `(Namespace, EntityType)` to feed `seed_ordinal`. `Custom(name)` stores its raw dictionary
   name so two classes that format identically for display remain distinct.

### Validation status

Not compiled/tested in the dispatch environment (no reachable toolchain, as in T04). Code and tests
(`crates/vg-vault/tests/vault.rs`, unit tests in `codec.rs`/`keychain.rs`, and `seed_ordinal` tests in
`vg-core/keying.rs`) were written against the verified `vg-core` interfaces.

**Verified during PR review (2026-07-17):** the `bundled-sqlcipher-vendored-openssl` build compiled
clean (SQLCipher + OpenSSL from source, ~30s). Two fixes applied: the same trivial
`.is_multiple_of` clippy lint as T04, and a missing `Debug` on `Vault` (a test formats
`Result<Vault, _>` with `{:?}`) — added as a **redacting** manual impl rather than a derive,
since `Vault` holds the HMAC salt and a derive would print it. Full chain green:
`cargo build --locked && cargo clippy --workspace --all-targets --locked -- -D warnings &&
cargo fmt --check && cargo test` — 40 vg-core unit tests (+3 for `seed_ordinal`), 6 vg-vault
unit + 14 vault integration tests, plus all existing.

### Doubt-driven-development (Codex cross-model, 2026-07-17)

A fresh-context Codex review (given the diff + the frozen `VaultStore` contract + the
encryption/keying/namespace/TTL requirements) ran out of its turn budget mid-investigation
without a final synthesised verdict, but surfaced three concrete concerns I then chased down
in the code myself:

- **Fixed (real correctness bug): `intern` could return an expired-but-unpurged placeholder that
  `resolve` immediately rejects.** `resolve` filters on expiry (returns `NotFound` for an expired
  mapping) but `intern`'s `lookup_by_key` did not, so re-interning a value whose TTL had lapsed
  (before `purge_expired` ran) returned the stale placeholder — one that would then fail to
  resolve. Since `key_hex` is the `PRIMARY KEY`, minting a divergent new row is impossible, so the
  fix renews the expired row's TTL in place (re-minting `expires_at` exactly as a fresh intern
  would) and returns the same stable placeholder. New regression test
  `re_interning_an_expired_but_unpurged_value_renews_it_and_stays_resolvable`.
- **Reconciled (accepted design, documented): `resolve` writes a `demask_event` row on every
  attempt, including a failed namespace probe.** This is deliberate and correct — the schema
  comment and this task's own decision #7 both state a reversal (success or denial) must be
  attributable, and the row holds only the opaque `mapping_ref` + namespace, never the value.
  Forward note for **T07**: when the pipeline also drives the `vg-audit` `AuditSink`
  (`DemaskDecision`), avoid double-logging the same demask — decide which layer owns it.
- **Reconciled (round 1): the `UNIQUE (ns, ty, ordinal)` index is a *defense* against a
  cross-process ordinal race** (a duplicate-ordinal insert fails loudly rather than silently
  colliding). Cross-process openers are documented out of Phase 1 scope. **This claim was
  partly WRONG and was corrected in round 2 — see below.**

Namespace isolation on `resolve` was verified present and correct (returns `NotFound` on a
namespace mismatch, never distinguishing it from "doesn't exist" — `assert_vault_roundtrip`
passes). Round 1's Codex pass did not deliver a full verdict.

### Doubt-driven-development, round 2 (Codex cross-model, complete verdict, 2026-07-17)

Re-ran a tighter, exploration-forbidden Codex critique (round 1 had exhausted its budget
reading the wider repo). It delivered one concrete finding, and it invalidated round 1's own
reconciliation above:

- **Fixed (real bug): the ordinal `UNIQUE` guard did NOT actually fire for fixed entity types.**
  SQLite treats `NULL` as *distinct* in a `UNIQUE` index, and every fixed entity type (Email,
  Iban, …) stores `entity_custom = NULL`. So the index `(ns_kind, ns_id, entity_kind,
  entity_custom, ordinal)` never rejected a duplicate `EMAIL_001` for two different secrets in
  the same namespace — the exact collision round 1 claimed it "made fail loudly." Only
  `Custom(name)` rows (non-null `entity_custom`) were ever covered. Fixed by keying the index on
  `COALESCE(entity_custom, '')`, so all fixed-type rows share one key value and the constraint
  applies uniformly. New regression test
  `the_ordinal_unique_guard_fires_for_a_fixed_entity_type_null_entity_custom` (two `Vault`
  instances race the same ordinal; the second insert is now rejected). This is a good example
  of why a *complete* cross-model verdict was worth re-running for: round 1's partial pass left
  a plausible-but-false safety claim standing.

Full chain green after the fix: 40 vg-core + 6 vg-vault unit + 15 vault integration tests.

## 2026-07-17 - T05b audit sink: JSONL storage, versioned schema mirrors, and dependencies chosen to fit a hand-editable lockfile

### Context

T05b (Squad 5) dispatched headlessly to implement `vg_core::traits::AuditSink` in the
empty `vg-audit` stub crate. The task spec left the storage technology open (JSONL file
or SQLite, "pick one and record the choice, don't ask") and required versioned record
types plus the no-raw-values property test. Like T04's dispatch, this session had **no
reachable compiler**: every `cargo`/`rustc`/script invocation was permission-blocked in
the headless environment, which ended up shaping one real decision (below), not just the
validation story.

### Decisions

- **Storage = append-only JSON Lines file, fsynced per write** (`JsonlAuditSink`), not
  SQLite. Append-only maps directly onto `O_APPEND` + `fsync` with no schema/connection
  machinery; the audit log is a sequential record of events, not a queryable mapping
  store, so T05's SQLCipher choice solves a different problem and "one storage tech
  across both" bought nothing here. The deciding constraint was dependency-light-ness
  taken literally: serde/serde_json/thiserror/uuid are all already in `Cargo.lock` (via
  criterion's tree and `vg-core`), while `rusqlite`/SQLCipher are not — and with no
  runnable `cargo` in-session, a lockfile entry that can't be generated can't be added
  honestly (see next point).
- **`Cargo.lock` was updated by hand** — safe only because the change is a single
  dependency-edge list (`vg-audit`'s own entry) between packages already locked with
  checksums. Two would-be dependencies were dropped to keep it that way: `tempfile`
  (dev-dep; replaced by a 10-line std-only tempdir in the test file) and uuid's `serde`
  feature (replaced by a `#[serde(with)]` adapter over `Display`/`parse_str`, same wire
  format). The reasoning is recorded in `vg-audit/Cargo.toml` itself so the next agent
  doesn't "clean it up" into a broken `--locked` build.
- **The storage schema is a deliberate mirror, not serde derives on `vg-core` types.**
  `vg-core` is frozen and serialization is vg-audit's concern, so `record.rs` defines
  `RecordV1`/`EventV1` (+ mirrors of `EntityType`, `ArtefactKind`, `HandlingClass`,
  `Destination`) with explicit conversions both ways. Every record carries
  `schema_version` (currently 1); the exact v1 wire shape is pinned by a unit test that
  says, in its own doc comment, "a change here means a version bump and a new record
  type, not an edit". Conversions toward storage are fallible (`TryFrom`) for every
  `#[non_exhaustive]` contract enum, so a future contract variant fails loudly at write
  time instead of being silently dropped. `DestinationV1` serializes to exactly the
  stable `DestinationId` strings (`"remote-model-prompt"`, ...) so the audit log and
  policy dictionaries share one destination vocabulary — also pinned by test.
- **Recovery semantics, chosen and tested:** an unparseable line at open is skipped and
  counted (`skipped_lines()`) — that's what a torn write from a crashed writer looks
  like — and an unterminated final line is healed with a lone newline so the next append
  starts clean (the file is never truncated or rewritten). But a *well-formed* record
  with an unknown `schema_version` refuses to open (`OpenError::UnknownSchemaVersion`):
  silently skipping real records written by a newer build would make the audit trail
  quietly lossy, which is worse than failing.
- **`get` serves from an in-memory index** built by replaying the file at open —
  acceptable for Phase 1's in-process, per-invocation lifetime (same trade-off already
  accepted for T04's session cache). The index stores what the storage schema
  *round-trips to*, not the caller's original value, so a lossy conversion would fail
  the conformance roundtrip test immediately rather than hiding until the first restart.
- **The acceptance property test checks the persisted bytes, not just the Debug form.**
  `tests/sink.rs` writes every event variant "about" a table of adversarial raw values
  (newlines, tabs, quotes, backslashes, a realistic IBAN and API key, unicode) and
  asserts none appear in the file either verbatim or JSON-escaped — the exact leak class
  `assert_audit_event_excludes_raw_values`'s own doc warns about — plus a negative
  control proving the helper actually catches a deliberately leaky event.

### Assumptions (recorded, not asked — one-shot dispatch)

- Type names `JsonlAuditSink`/`OpenError` and the file layout (`lib.rs` + `record.rs`)
  were free choices; nothing in the contract names the concrete impl type.
- `OpenError` is a crate-local error type: `open()` isn't part of the frozen `AuditSink`
  trait, and `AuditError`'s single frozen `Write` variant is the wrong shape for it.
- Snake_case/kebab-case wire naming follows serde convention and the `DestinationId`
  precedent; nothing else in the repo had established a JSON naming style yet.

### Validation status

**Not compiled or tested in the dispatch session** — all toolchain access was
permission-blocked (same constraint as T04's dispatch; recorded there as a factory gap).
Mitigations: a line-by-line self-review pass that caught three real would-be compile errors
before handoff (`PathBuf` has no `Display` in thiserror format strings; an exhaustive match
on `#[non_exhaustive]` `Destination`; a moved-while-borrowed `path` in `open`), plus
rustfmt-canonical formatting written deliberately.

**Verified during PR review (2026-07-17):** the standard verify chain ran clean on the
first real attempt after a single `fmt` pass — `cargo build --locked && cargo clippy
--workspace --all-targets --locked -- -D warnings && cargo fmt --check && cargo test`,
with 8 sink tests + 3 record tests + the whole existing suite green.

### Doubt-driven-development (Codex cross-model, 2026-07-17)

A fresh-context Codex review, given the diff + the frozen `AuditSink` contract + the
redaction-safety requirement, found a strong set on this security-critical persistence
layer. Reconciliation:

**Fixed (real robustness/security):**
- **A crash mid-multibyte-UTF-8 in the torn final line bricked the whole log.** `open()`
  read the file with `read_to_string`, which fails entirely on any invalid UTF-8 —
  contradicting the documented "torn write tolerated" guarantee. Now reads raw bytes and
  decodes per line, so invalid UTF-8 is confined to (and tolerated in) the torn tail.
- **Any unparseable line was silently skipped, not just the torn final one.** A complete
  (newline-terminated) interior line that fails to parse is corruption/tampering, not a
  torn write, and silently dropping it made the index no longer represent the append log —
  the exact "quietly lossy" failure the `UnknownSchemaVersion` path was already written to
  avoid. Now only a genuinely torn *final* line is tolerated; every complete line must
  parse or `open()` returns the new `OpenError::CorruptLine`. This also closes the
  bypass where a malformed `schema_version` (e.g. the string `"2"`) failed `VersionProbe`
  and was skipped as torn instead of refused.
- **Recovery changed from newline-heal to truncation.** The old heal appended a `\n` to the
  torn tail, immortalising a garbage fragment that was re-skipped on every future open — and
  under the stricter rule above, that healed fragment would then read as a fatal
  complete-corrupt line. Truncating the torn tail back to the last complete record is the
  honest recovery (this sink fsyncs `record + '\n'` per write, so a tail without a trailing
  newline was never a committed record) and keeps the file all-complete-lines.
- **A duplicate `AuditId` on replay silently shadowed the earlier record** (`index.insert`
  overwrote it; `len()` under-counted). IDs are internal UUIDv4s, so a duplicate means the
  append-only log was spliced/tampered — now a hard `OpenError::DuplicateId`.
- **Error text could leak a raw value.** `unsupported()` Debug-formatted the *value* of an
  unknown future variant into `AuditError::Write` — a string a caller may log. For the one
  tool whose whole job is keeping raw values out of side channels, that is exactly the wrong
  failure. Now names only the *type*, never the value. Added a parent-directory fsync on
  first create (file-level fsync alone doesn't make a new dirent durable).
- Four regression tests added, one per fixed behaviour.

**Reconciled as accepted trade-offs (documented, not changed):**
- **The sink cannot enforce "no raw value persisted" — and by design does not try.** It has
  no oracle for what is "raw" (it never sees the vault or the original detected values), so
  it faithfully persists whatever event it is handed; keeping events clean is the
  *constructing* code's contract (Task T07), enforced at construction by
  `assert_audit_event_excludes_raw_values`. The struct doc now states this boundary
  explicitly. The property test proves well-constructed events don't leak; it cannot prove a
  buggy caller won't hand the sink a leaky `Block.reason`, because nothing at this layer
  could.
- **Single-live-sink coupling:** `get`'s in-memory index only reflects this sink's own
  writes; a second sink on the same file, or another process appending, needs a reopen to
  be seen. Made explicit in the struct doc; multi-opener coordination is out of Phase 1
  scope.

Round 1 stopped after one cross-model cycle — every finding was fixed or explicitly classified.

### Doubt-driven-development, round 2 (Codex cross-model, 2026-07-17)

A second, tighter Codex critique (re-run for a complete verdict across all Wave B tasks) found
one real bug that round 1's fixes had actually *introduced*:

- **Fixed (High): a valid-JSON final line with no trailing newline was indexed, then truncated
  off disk.** Round 1's recovery reads the torn tail, and if it happened to be a complete,
  parseable record, `index.insert`ed it — but the post-loop truncation then removed it from
  disk. So `get()`/`len()` reported an event that a reopen would lose: an index/disk
  inconsistency. Fixed by skipping the torn tail *unconditionally* (before parse/index), which
  is correct anyway — a record whose terminating `\n`+fsync never landed was never a
  durably-committed record for this sink, so discarding it is honest, and now index and disk
  agree. New regression test `a_valid_record_without_a_trailing_newline_is_not_indexed_then_lost`.
  Full chain green after: 13 sink tests + 3 record tests + the existing suite.

## 2026-07-17 - T06: `vg-policy` PolicyEngine implemented (`LayeredPolicyEngine`)

### Context

Implemented `vg_core::PolicyEngine` in `crates/vg-policy` (previously an empty stub). The
engine resolves up to three layered policy packs (session-over-repo-over-global) into one
validated policy and answers the six trait methods. See `crates/vg-policy/src/{config,engine}.rs`
and the fixtures/tests. Decisions taken during the task (recorded here rather than asked, per
the one-shot headless dispatch instruction):

### Decision — policy-pack format is **JSON** in Phase 1, not TOML/YAML (deviates from ADR-007)

ADR-007 (2026-06-30) said "native YAML/TOML now". The T06 spec restated that as "YAML
(`serde_yaml`) or TOML (`toml`) are both reasonable". I chose **JSON via `serde_json`** instead,
for one concrete, environment-driven reason: **cargo cannot run in this dispatch sandbox**, so
`Cargo.lock` cannot be regenerated. `serde`, `serde_core`, `serde_derive`, and `serde_json` are
*already fully resolved* in the workspace `Cargo.lock` (pulled in transitively by `criterion`),
whereas neither `toml` nor `serde_yaml` is present. Adding `toml`/`serde_yaml` would introduce
new registry packages (winnow, serde_spanned, toml_edit, indexmap, …) that I cannot resolve or
checksum by hand, which would break the `cargo build --locked` acceptance criterion. JSON adds
**zero** new locked packages — the only `Cargo.lock` change is adding the `serde`/`serde_json`
edges to `vg-policy`'s own dependency list.

This is a deliberate, reversible, Phase-1-scoped deviation, not a rejection of ADR-007. The
on-disk schema is format-agnostic serde structs (`RawPack` etc.), so switching the format crate
(e.g. to `toml`) is a one-line change in `LayeredPolicyEngine::read_layer` plus renaming the
fixtures — no schema or engine-logic change. **Follow-up (see next-actions):** when the build
environment can regenerate the lock, reconcile format with ADR-007 (TOML) or amend ADR-007 to
accept JSON. Neither `serde_yaml` (unmaintained/deprecated upstream) nor a hand-rolled parser
was considered a good option.

### Decision — the hard-deny rule is enforced *in code*, above the config layer

`demask_allowed` returns `false` for `Destination::RemoteModelPrompt` and
`Destination::ObservabilitySink` via an explicit `matches!` guard *before* the pack is consulted
— a malicious or misconfigured pack that sets `demask_allowed: true` for them cannot override it
(regression-tested in `malicious_pack_cannot_unlock_hard_deny_destinations`). This is the one
security-load-bearing part of the task; everything else is configuration plumbing. Verified with
`vg_core::conformance::assert_policy_engine_denies_hard_deny_destinations`. As defence in depth,
`destination_allows_masked_only` also forces `true` for those two destinations regardless of
pack contents (the send-side mirror of the same invariant — the contract only mandates the
`demask_allowed` half, this strengthens it at no cost).

### Decision — signed-pack verification is a clearly-marked always-accept stub

`config::verify_signature` returns `Ok(())` unconditionally in Phase 1 (interface-contracts.md
§6: "stub in Phase 1, enforced later"). The `signature` field is threaded through `RawPack` now
so Phase 2 can add real verification without a load-flow or schema change. Marked **PHASE 1
STUB — must be replaced before loading untrusted packs** in the doc comment.

### Smaller decisions (schema/semantics)

- **Entity/handling-class keys are stable kebab-case strings** (`config::entity_key`,
  `parse_class`) since `EntityType`/`HandlingClass` live in `vg-core` without serde derives and
  are `#[non_exhaustive]`. `Custom(name)` keys on its dictionary name directly; a future
  `EntityType` variant falls back to a lower-cased debug name (no breaking change here).
- **Unknown handling-class strings fail at load** (`ResolvedPolicy::from_raw`), not lazily at
  first `classify_*` — a pack typo is a load error.
- **Layer merge is key-by-key** (a repo/session layer overrides only the keys it names);
  destination rules merge *field by field* so one layer can flip `demask_allowed` without
  restating `masked_only`.
- **Fail-safe defaults:** unclassified entity → `Mask`; unconfigured destination →
  masked-only `true` and demask `false`; artefact default → `Pass`.
- **Optional role-gating:** a destination may list `demask_roles`; if non-empty the actor must
  hold one. This makes the `actor` parameter meaningful without over-building auth (Cedar is
  still ADR-007's later target).

### Validation status + doubt-driven-development (2026-07-17)

**Verified during PR review:** compiled clean (no new registry deps — `serde`/`serde_json`
already in the lockfile, chosen deliberately so `--locked` stays green); one `fmt` pass
applied; full chain green — 4 config + 9 engine tests plus the whole existing workspace suite.

**Codex cross-model doubt-pass** (given the diff + the frozen `PolicyEngine` contract + the
hard-deny requirement) ran out of its turn budget before a full verdict, but flagged the crux
(hard-deny bypass) which I then verified directly, along with the other fail-safe properties:

- **Hard-deny is unbypassable — verified.** `demask_allowed` returns `false` for
  `RemoteModelPrompt`/`ObservabilitySink` via a direct enum `matches!` checked *before* any pack
  rule is consulted, so no global/repo/session pack (malicious, misconfigured, or mistaken) can
  flip them. `destination_allows_masked_only` mirrors this on the send-side gate
  (`is_hard_deny_id` → forced masked-only). Confirmed against
  `assert_policy_engine_denies_hard_deny_destinations`.
- **Layering verified:** `load` merges global → (repo over global) → (session over repo),
  so session > repo > global. A malformed layer (or an unknown handling-class string) makes the
  *entire* `load()` fail (`PolicyError::Load`) — no partially-loaded, silently-wrong engine.
  Fail-safe.
- **Corrected the framing of one "fail-safe default" bullet above.** `artefact_default = Pass`
  is NOT fail-safe in the same sense as `entity_default = Mask`, and the review rightly flagged
  the asymmetry. It is nonetheless the *correct* default, for a real reason now documented in a
  code comment at `config::ResolvedPolicy::from_raw`: artefact class is a whole-file decision
  (`Block` refuses a file, `Pass` sends it), and defaulting unknown file types to `Block` would
  refuse everything not allow-listed and make the tool unusable — while their detected PII
  entities are STILL masked, because entity classification defaults to `Mask` independently.
  **Hard requirement recorded for T07:** artefact-`Pass` must mean "send after entity masking,"
  never "skip detection/masking for this file"; if T07 lets an artefact class short-circuit
  entity scanning, this default becomes a fail-open leak.

Because Codex did not deliver a full verdict, this counts as one partial cross-model cycle; the
concrete concern it raised (hard-deny bypass) is verified closed, and the asymmetry it would
have reached is documented and flagged forward.

### Doubt-driven-development, round 2 (Codex cross-model, complete verdict, 2026-07-17)

Re-ran a tighter, exploration-forbidden Codex critique (given the diff + the hard-deny/layering
contract inline) to get a complete verdict where round 1 had run out of budget. Result: **"no
issues found after thorough examination."** This is the only Wave B task whose second-round
critique surfaced nothing — consistent with the manual verification above (hard-deny
unbypassable, layering fail-safe, defaults documented). No code change from round 2.


## 2026-07-17 - T08 (`vg-parsers`) built (headless dispatch, unable to reach a compiler)

### Context

T08 (Squad 2): implement `vg_core::Parser` in `crates/vg-parsers/` — one module per format
(`json`, `yaml`, `toml`, `csv`, `log`, `diff`, `env`) plus tree-sitter for one source
language. Hard contract: **never panic on malformed input**, return best-effort spans.
Cross-crate requirement (interface-contracts.md, 2026-07-16): feed real `Span` output into
`vg_detectors::all_detectors()` and record whether the detectors' `_spans` no-op is an
expected gap. One-shot headless dispatch — ambiguities resolved by judgment and recorded
here rather than asked.

### Judgment calls recorded (no follow-up channel)

1. **Source language for tree-sitter = Rust** (`rust.rs`, `ArtefactKind::SourceCode("rust")`).
   The task said pick something simple and common if ambiguous; Rust is this project's own
   language (ADR-001) and the grammar crate is well maintained. Tree-sitter is error-tolerant
   by construction (produces `ERROR`/`MISSING` nodes, never fails), which matches the
   never-panic contract directly.

2. **JSON is hand-rolled, not `serde_json`.** `serde_json`'s tree parser gives no byte offsets
   and aborts at the first syntax error — the opposite of "best-effort spans over malformed
   input." `json.rs` is a single-pass tolerant byte tokenizer that classifies each string as an
   object `Key` (followed by `:`) or a `StringLiteral`/`Value`, and degrades an unterminated
   final string to a span clamped at end-of-buffer.

3. **YAML and TOML: third-party parser as a well-formedness gate, hand-rolled line scanner for
   spans.** `serde_yaml` and the `toml` crate both parse into an owned value tree with **no byte
   offsets**, so neither can answer "where in the buffer is this key." They are still
   load-bearing: (a) exercised on every parse, including the adversarial never-panic battery, so
   the third-party parsers' own panic-safety is verified alongside ours; and (b) for YAML, a
   *valid* document with no block-style `key:` structure (i.e. flow style `{a: 1}`, which is
   JSON-shaped) falls back to the JSON tokenizer for spans. Block-style YAML / TOML `key = value`
   / `[table]` / comments come from the quote-aware line scanners. This is why `serde_yaml` and
   `toml` are dependencies even though the spans don't come from them — recorded so a reader
   doesn't mistake them for dead weight. (Only `yaml.rs` was contractually required to "add
   serde_yaml or similar"; the `toml` crate is used the same way by choice, not mandate.)

4. **`.env` inline `#` is not a comment.** dotenv tools disagree on whether `KEY=val # x` has a
   comment; a value like `p#ssw0rd` or a URL fragment must never be truncated (and secrets live
   in exactly these values). Only a whole-line-leading `#` is a comment. TOML/YAML `#` handling
   *is* quote-aware because their grammars define it.

5. **Span structural tags** use the frozen `NodeKind` variants: object/map keys → `Key`, scalar
   values → `Value`/`StringLiteral`, CSV body cells → `Field(header_name)` (column-aware),
   log timestamp/level and diff added/removed/path → `Field(...)`, hunk headers → `Other("hunk")`,
   comments → `Comment`, tree-sitter identifiers → `Identifier`. All spans route through a single
   `util::span` helper that clamps `end` to buffer length and `start ≤ end`, so the Parser
   span-bounds invariant holds unconditionally even if a format's own scanning logic has an
   off-by-one.

### Cross-crate integration finding: the detectors' `_spans` no-op is an EXPECTED, stage-appropriate gap

**Required classification (interface-contracts.md, 2026-07-16).** `crates/vg-parsers/tests/detector_integration.rs`
feeds this crate's real `Span` output into `vg_detectors::all_detectors()` on realistic JSON,
`.env`, YAML, and CSV fixtures, and additionally pins the observed behaviour: feeding real
parser spans, an empty slice, and deliberately *wrong* spans all yield **identical** findings —
because every T03 detector's signature is `detect(&self, buf, _spans)`, ignoring `spans` today.

**This is an expected gap, not a real one**, for three reasons:
- **No contract is violated.** The detectors still satisfy their contract (valid findings,
  in-bounds spans, determinism) without consuming `spans`. The T03 detectors are regex/entropy
  scanners over the whole buffer; whole-buffer scanning is a *superset* of structure-scoped
  scanning, so nothing is missed by ignoring structure — it can only over-scan, never under-scan.
- **The pipeline that threads parser spans into detectors is T07** (masking-pipeline wiring in
  `vg-core`, Wave C), which does not exist yet. Parsers (T08) *produce* spans; T07 *wires* them
  in; span-aware detection is a deliberately later enrichment, consistent with the phased plan
  and interface-contracts.md §3–§4.
- **The no-op is now pinned by a test.** If a future change (T07, or a span-aware detector) makes
  `detect` actually consume `spans`, `detectors_currently_ignore_the_spans_parameter` breaks
  first and loudly, forcing this classification to be revisited rather than the no-op being
  silently assumed still true.

**Deferred opportunity, flagged for T07 (not a defect now):** the 2026-07-16 census found the
entropy/phone detectors' dominant false positives were on file paths and snake/kebab identifiers.
Span-awareness is the natural structural fix — e.g. scanning only `Value`/`StringLiteral` spans
and skipping `Key`/`Identifier`/`Comment` spans would cut exactly that false-positive class. That
is a T07-era enhancement (wire spans through, then let detectors opt into structure), not
something T08 can or should do inside `vg-detectors` (ownership rule: edit only your crate).

### Verification status — NOT built in-session (compiler unreachable), and a Cargo.lock action required

As with T04, this headless dispatch could not reach a compiler: every `cargo`/`rustc`/`python`
invocation (and any shell wrapper around one) is gated behind an approval prompt with no human
in a one-shot dispatch. The code is written for correctness and panic-safety by inspection, with
a thorough per-module adversarial `assert_parser_never_panics` battery (empty, truncated UTF-8,
unbalanced delimiters, all-NUL, every-byte-value, binary-masquerading-as-format) plus a shared
registry-wide battery in `lib.rs`. **These must be run at PR review**, exactly as T04 was:
`cargo build --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check &&
cargo test -p vg-parsers`.

**Two review-time actions specific to T08 (distinct from T04, which added no dependencies):**
- **`Cargo.lock` MUST be regenerated.** T08 adds four dependencies (`serde_yaml`, `toml`,
  `tree-sitter`, `tree-sitter-rust`) but `Cargo.lock` could not be updated without running cargo.
  Until a maintainer runs `cargo build` (or `cargo update -p vg-parsers`) and commits the
  refreshed `Cargo.lock`, **every `--locked` CI job (build, test, clippy, bench) will fail
  immediately** with "lock file needs to be updated." This is the top handoff item.
- **Verify the tree-sitter version pair resolves and compiles.** `tree-sitter = "0.22"` +
  `tree-sitter-rust = "0.21"` were chosen for the `set_language(&Language)` / `language()` API
  used in `rust.rs`. This pairing is the single most likely thing to need adjustment on first
  real build (the tree-sitter grammar crates changed `language()` → `LANGUAGE` in later
  versions); if it doesn't resolve, pin the matching pair rather than changing the call site
  blindly. `cargo fmt` at review will also absorb any residual formatting the hand-write missed.

**Verified during PR review (2026-07-17).** `Cargo.lock` regenerated (the four deps resolved;
the `tree-sitter 0.22` / `tree-sitter-rust 0.21` pairing compiled without adjustment). Fixed
during review: two trivial clippy items (`int_plus_one` in `env.rs`; unused `Span`/`Detector`
imports); two borrowed-temporary compile errors in tests (`let x = &Parser.parse(buf).spans`
dangles — bound the `ParseResult` first, in `env.rs` and `yaml.rs`); and **one real correctness
bug** — `yaml::falls_back_to_json_tokenizer_for_flow_style` failed because the block line-scanner
greedily matched the first `:` in a flow-style line `{"host": ...}` and emitted `{"host"` as a
bogus "key", so `spans` was never empty and the JSON-tokenizer fallback never fired. Fixed by
skipping block key-extraction for a segment that opens with a flow indicator (`{`/`[`), leaving
it to the JSON tokenizer (a line like `config: {host: db}` opens with `config`, so it is
unaffected). Full chain green after: 36 `vg-parsers` lib tests + 3 cross-crate integration tests
+ the whole existing suite.

### Doubt-driven-development (Codex cross-model, 2026-07-17)

A fresh-context Codex review ran (given the diff + the `Parser` never-panic/span-bounds
contract, focused on panics, out-of-bounds spans, and false-negative "missed value" spans). It
ran out of its turn budget before a full verdict, but its final assessment was explicit and
matches an independent read: **no out-of-bounds / panic risks found** (the hard contract holds —
backed by the per-module `assert_parser_never_panics` batteries and `every_span_is_within_bounds`
tests, all green), and the only residual is **partial spanning** — a syntactically valid value
covered by only part of a span, which *could* let a span-aware detector miss part of a secret.

Reconciled as a **documented, stage-appropriate concern flagged forward to T07**, not a current
bug: the T03 detectors currently *ignore* the `spans` parameter entirely (the `_spans` no-op,
asserted and explained in `tests/detector_integration.rs` and the section above), so today every
detector scans the raw buffer, not the spans — a partial span cannot cause a missed detection
until T07 wires spans into the pipeline. **Hard note for T07:** when spans start gating what a
detector scans, partial/underspanning becomes a real false-negative (missed-secret) risk;
before relying on parser spans to scope detection, add coverage that a value spanning multiple
tokens/lines (a multi-line string, a wrapped base64 blob, a quoted value with embedded
delimiters) is fully covered, or have the pipeline fall back to scanning the raw region. The one
real underspanning bug that *did* exist at this layer (the yaml flow-style mis-parse above) is
fixed.

### Doubt-driven-development, round 2 (Codex cross-model, complete verdict, 2026-07-17)

A second, tighter Codex critique confirmed **no panic / out-of-bounds-span risks** and named two
concrete instances of the partial-spanning class above. Both are Medium and, per the `_spans`
no-op, cannot cause a missed detection until T07 — so both are recorded here as **specific known
under-spanning limitations for T07 to close**, not fixed now (quote-aware YAML scanning in the
best-effort line scanner is real parser work with regression risk, and the well-formedness
gate + JSON fallback already back-stop structure):

1. **A `#` inside a quoted YAML value is treated as a comment.** `password: "abc #def"` yields a
   Value span over only `"abc` and a Comment span over `#def"`. `comment_start` requires a `#`
   preceded by whitespace but is not quote-aware. (The `.env` parser deliberately handles the
   analogous `pass#word` case; YAML's block scanner does not.) A secret containing ` #` would be
   split. **T07 fix direction:** make comment detection quote-aware, or have the pipeline scan
   the raw line for a value the parser under-spanned.
2. **YAML flow scalars that aren't JSON double-quoted strings are under-spanned.** The flow-style
   fallback reuses the JSON tokenizer, which only recognises `"…"` strings and bare scalars;
   single-quoted (`{token: 'sk-live-abc'}`) or unquoted flow scalars get no complete span.
   **T07 fix direction:** a YAML-aware flow tokenizer, or raw-region fallback.

Both are the same "detector could miss part of a value once spans gate detection" risk already
flagged above — now with concrete reproducers to test against at T07.
