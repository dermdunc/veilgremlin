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
