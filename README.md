# VeilGremlin

**Classification:** factory-output (Hekton) · **Owner:** dermdunc · **Status:** design + scaffold (Phase 0 → Phase 1)

> A **local-first privacy shield for agentic engineering.** It keeps real PII and sensitive enterprise identifiers out of model context when developers use vibe-coding / agentic tools (Claude Code, Codex, Cursor, Continue, Cline, …). Masking is automatic and local; demasking is explicit, local, policy-gated, and auditable. The cloud model only ever sees placeholders.

VeilGremlin is **not** another guardrail or DLP scanner. It is the millisecond-sensitive, file-aware, **reversible-pseudonymisation** layer that sits in the hot path of every coding-agent turn and keeps the reversal material on your laptop.

> **Positioning:** a technical and governance control **supporting** data minimisation, privacy by design, auditability, and risk-based adoption. Not a GDPR or EU AI Act "compliance" guarantee.

## The one hard rule

**Unless the model is local and explicitly approved, it never receives real PII or sensitive enterprise identifiers.**

## How it works (Phase 1)

A small hardened **Rust core** runs entirely on the developer laptop: parse → detect → vault → policy → masked pack, on a deterministic hot path measured in milliseconds (no network, no LLM). Thin adapters wire it into **Claude Code on Amazon Bedrock** via hooks + a `vg run` wrapper. An encrypted **SQLCipher vault** holds reversible mappings; an audit log records what was masked, blocked, and demasked — without storing raw values.

```
vg run -- claude "Debug this incident and propose a regression test"
vg inspect incident.log          # preview what would be masked
vg demask --from result.patch --to local_patch
vg audit last
```

## Documentation

| Doc | What |
|---|---|
| [Requirements & Design Spec](docs/spec/requirements-and-design-spec.md) | Canonical Phase 0/1 spec (threat model, vault, policy, latency budget, 6 diagrams, Go/No-Go) |
| [Agent Factory Build Plan](docs/architecture/agent-factory-plan.md) | How teams of agents build it — squads, waves, gates |
| [Work Breakdown (T01–T11)](docs/architecture/work-breakdown.md) | Task DAG with owners + acceptance |
| [Interface Contracts](docs/architecture/interface-contracts.md) | Frozen crate seams for parallel build |
| [Architecture index](docs/architecture.md) · [Decisions](docs/decisions.md) · [Risks](docs/risks.md) · [Next Actions](docs/next-actions.md) | — |
| [Deep Research Report](docs/research/deep-research-report.md) | Source analysis |

## Status

- **Done:** Phase 0/1 design, agent-factory build plan, repo scaffold (Hekton factory-output).
- **T01 (Cargo workspace + CI + supply-chain skeleton):** built 2026-07-14, PR open
  (github.com/dermdunc/veilgremlin/pull/2), not yet merged.
- **Next:** merge T01, then freeze interface contracts (T02). Once T01 + T02 both merge, batch-dispatch Wave B squads.

## Project conventions

Built inside the Hekton agentic factory. Agents must follow `~/hekton/CLAUDE.md` and this repo's `CLAUDE.md` / `AGENTS.md`: inspect `.hekton/project.yaml` before structural changes, record decisions in `docs/decisions.md`, keep `docs/session-log.md` current, and use session-scoped commits. Vault mutation is not allowed by default.
