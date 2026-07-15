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
