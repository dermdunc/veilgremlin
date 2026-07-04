# VeilGremlin — Plain-English Project Walkthrough

## What this project is in one paragraph

VeilGremlin is a local-first privacy shield for "vibe coding". When a developer uses an AI coding agent (like Claude Code) that sends code, logs, and tickets to a cloud model, VeilGremlin sits on the laptop *in front of* that model. It automatically swaps real personal data and sensitive company identifiers for stable placeholders before anything leaves the machine, then lets the developer put the real values back locally — explicitly and auditably — after the model replies. The cloud model only ever sees placeholders.

## The simple analogy

It is a translation booth at the door. Outgoing messages get their secrets swapped for code-words (`EMAIL_001`, `ACCOUNT_ID_014`) on the way out; the reply comes back in code-words; and only you, standing inside the booth, hold the key to translate the code-words back — and only into local files, never back out to the cloud.

## What problem we are solving

AI coding agents pull in far more than a chat prompt: files, diffs, terminal output, logs, tickets, MCP resources. Any of those can contain real customer or employee data. Guardrails and DLP scanners inspect data *after* the provider already has it. VeilGremlin changes *what leaves the laptop in the first place* — which is what privacy and risk teams actually care about, and which aligns with data-minimisation and privacy-by-design expectations (without claiming "compliance").

## What we have built so far

- **2026-06-30** — Project scaffolded as a Hekton **factory-output** repo under the **coderturtle** GitHub account (private).
- **2026-07-04** — Repo ownership moved to **dermdunc** and made **public** — VeilGremlin is an
  enterprise architecture/governance/risk tool, not agentic-engineering tooling, so it belongs
  under the professional-identity account per Hekton's new domain-based GitHub routing decision.
- Authored the full **requirements & design specification** (`docs/spec/`), covering Phase 0 (discovery) and Phase 1 (laptop MVP): threat model, taxonomy, hot/warm/cold path design, token vault, policy-as-code, supply-chain model, six architecture diagrams, and Go/No-Go criteria.
- Authored the **agent factory build plan** (`docs/architecture/`): how teams of agents build this — squad-per-crate ownership, a contract-first method, the four build waves, the task DAG (T01–T11), and the frozen interface contracts that let squads work in parallel without colliding.
- Brought the source **deep research report** into the repo (`docs/research/`).

No Rust code yet — this session is design and scaffolding only (Hekton rule: stop before building unless asked to implement).

## How the pieces fit together

A small hardened **Rust core** does the work: parse → detect → vault → policy → masked pack, all locally on a deterministic "hot path" measured in milliseconds (no network, no LLM). Thin **adapters** wire it into Claude Code (hooks + a `vg run` wrapper) and, later, into LiteLLM gateways and cloud-agent worktrees. An encrypted **SQLCipher vault** holds the reversible mappings; an **audit log** records what was masked, blocked, and demasked — without storing raw values.

## What is deliberately not automated yet

- The actual Rust implementation (Phase 1 build, tasks T01–T11).
- Warm-path local NER (GLiNER) — designed but off by default.
- LiteLLM gateway, MCP server mode, CI/CD mode, cloud-agent packaging — all later phases.
- Synthetic-data generation and quasi-identifier leakage scoring — Phase 4.

## How this could connect to the wider Hekton factory

VeilGremlin is itself a demonstration of the Hekton "agent factory" model: a team of builder Gremlins, each owning one crate, coordinated by frozen contracts and gated by an eval harness. It also complements existing labs (e.g. the engine-gateway-lab routing work and local-llm-lab) by being the privacy boundary that decides what context is safe to route to which model.

## Current confidence level

Design: high (grounded in the research report and an explicit Go/No-Go bar). Implementation: not started. Update as evidence grows.

## Open questions

- Multilingual entity coverage; quasi-identifier leakage; screenshot OCR boundary; graph-based context preservation; exact Cedar/OPA choice for enterprise policy; repo- vs session-scoped placeholder stability default. (See spec § Open Questions.)
- ~~Repo visibility: stay private vs open-source publicly under coderturtle, and when.~~ —
  decided 2026-07-04: public, under dermdunc (see decisions log).

## Next recommended session

Confirm the first push, then dispatch Wave A: Squad X builds the Cargo workspace + CI (T01) and Squad 0 freezes the interface contracts (T02). Once both merge, batch-dispatch the five Wave B squads.
