# Brief: VeilGremlin

> Local-first privacy shield that keeps real PII and sensitive enterprise identifiers out of model context in agentic coding workflows.

## Problem

AI coding agents (Claude Code, Codex, Cursor, Continue, Cline, cloud-hosted agents) pull rich context — files, diffs, terminal output, logs, tickets, MCP resources — into cloud model prompts. Any of it can carry real customer/employee data. Guardrails and DLP inspect data *after* the provider has it. Regulated enterprises need to constrain *what leaves the laptop* in the first place, while keeping the model useful for engineering tasks.

## Outcome

A small hardened Rust core on the developer laptop that automatically masks PII/enterprise identifiers into stable typed placeholders before cloud invocation (deterministic, local, millisecond hot path), stores reversible mappings in an encrypted local vault, and rehydrates only via an explicit, local, policy-gated, auditable demask. First reference workflow: Claude Code on Amazon Bedrock. Gateway-ready (LiteLLM) and cloud-agent-aware by design.

## Constraints

- Unless the model is local + approved, it never sees real PII.
- Masking automatic; demasking explicit/local/gated/auditable.
- Hot path: no network, no LLM, p95 < 25 ms (assembly), < 50 ms e2e.
- Supply-chain integrity first-class: signed releases, SBOM, reproducible builds, no telemetry.
- No GDPR / EU AI Act over-claiming — position as a supporting control.

## Canonical docs (repo is source of truth)

- Spec: `docs/spec/requirements-and-design-spec.md`
- Agent factory build plan: `docs/architecture/agent-factory-plan.md`
- Work breakdown + interface contracts: `docs/architecture/`
