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
