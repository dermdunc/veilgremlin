# VeilGremlin — Work Breakdown (Task DAG)

Companion to `agent-factory-plan.md`. Task IDs are stable references used in branches (`feat/<squad>-<id>-<slug>`), PRs, and the eval gates. "Blocks on" = must merge first.

## Legend
- **Wave**: A (foundation), B (parallel), C (integration), D (gate).
- **Owner**: squad from the topology table.
- **Hot path**: ✅ means a criterion bench + latency budget applies.

---

## Epic E1 — Foundation & supply chain

| ID | Task | Wave | Owner | Blocks on | Hot path | Acceptance |
|---|---|---|---|---|---|---|
| **T01** | Cargo workspace + crate skeletons; CI (`fmt`, `clippy -D warnings`, `cargo-deny`, `cargo-audit`, `--locked`); bench CI job; release skeleton (SBOM stub, signing stub) | A | Squad X | — | — | `cargo build` green on empty crates; CI runs on PR; `cargo-deny` config present |
| **T02** | Freeze shared types + library API in `vg-core`; define trait seams (Detector, Parser, VaultStore, PolicyEngine, AuditSink); write contract-conformance test stubs | A | Squad 0 | — | — | `interface-contracts.md` v1 frozen; types compile; conformance test scaffold exists |

## Epic E2 — Detection & masking primitives (parallel)

| ID | Task | Wave | Owner | Blocks on | Hot path | Acceptance |
|---|---|---|---|---|---|---|
| **T03** | Deterministic detectors: compiled regex sets (email/phone/IP/IBAN/sort-code), Aho-Corasick dictionaries, entropy detector | B | Squad 1 | T02 | ✅ | unit tests per detector; bench p95 < 25 ms on reference input. Done 2026-07-15: 41 tests, 0.62ms bench, a real CI-enforced latency-regression gate (`tests/latency_gate.rs`, generous CI-safe margin) added 2026-07-16 as a stopgap ahead of T10's precise baseline tracking |
| **T04** | Typed-placeholder + HMAC keying: canonicalisation, salted HMAC over `(value,type,namespace)`, per-type ordinals, Luhn/mod-97 checksum validators, session cache | B | Squad 1 + Squad 3 (shared contract) | T02 | ✅ | same value → same placeholder within namespace; checksum tests pass. **Added 2026-07-16:** must integration-test against real `Finding`s from `vg-detectors::all_detectors()` (T03, already built), not only mock values — catches a real interface/shape mismatch before T07, not only at it |
| **T08** | File-aware parsing: logs, git diffs, JSON/YAML, `.env`; tree-sitter for one source language; emit `Span` model to detectors | B | Squad 2 | T02 | partial | parser tests on fixtures; spans typed; error-tolerant on malformed input. **Added 2026-07-16:** must integration-test real `Span` output against `vg-detectors::all_detectors()` (T03) on a realistic fixture — note that all five T03 detectors currently ignore their `spans` parameter entirely (confirmed 2026-07-16); explicitly record whether that's expected at this stage or a gap, don't let it go unnoticed until T07 |

## Epic E3 — State, policy, evidence (parallel)

| ID | Task | Wave | Owner | Blocks on | Hot path | Acceptance |
|---|---|---|---|---|---|---|
| **T05** | SQLCipher vault: schema (`mapping`, `demask_event`), OS-keychain key wrap (macOS first), `zeroize`, TTL/purge, prepared-statement reuse | B | Squad 3 | T02 | lookup ✅ | encrypted at rest; key never plaintext; lookup bench within budget; TTL purge works |
| **T05b** | Audit sink: append-only structured events, redaction-safe (no raw values), versioned record types | B | Squad 5 | T02 | — | events written; property test: no raw value ever serialised |
| **T06** | Native policy engine: 3-layer load (global→repo→session), artefact/entity/destination/demask rules, signed-pack verification stub, `policy check` | B | Squad 4 | T02 | eval ✅ | example policy parses; layering resolves; deny rules enforced in tests |

## Epic E4 — Pipeline & integration

| ID | Task | Wave | Owner | Blocks on | Hot path | Acceptance |
|---|---|---|---|---|---|---|
| **T07** | Masking pipeline in `vg-core`: detectors → policy → vault → masked pack; `.env`/block path; irreversible-redact path; placeholder-consistency validation | C | Squad 0 | T03,T04,T05,T05b,T06,T08 | ✅ | end-to-end mask on fixture; irreversible class never vault-stored; e2e bench |
| **T09** | `vg` CLI + Claude Code adapter: `run`/`inspect`/`diff --masked`/`demask`/`audit`/`policy`/`vault` commands; hooks (`UserPromptSubmit`/`PreToolUse`/`PostToolUse`) + wrapper; Bedrock masked-request path; pre-send summary; explicit demask gate | C | Squad 6 | T07 | — | masked round trip to Bedrock path; demask gate denies `remote_model_prompt`; CLI help complete. **Added 2026-07-16:** a human runs a real interactive session with the hooks wired in and explicitly confirms no perceptible added latency/friction — the first point the "invisible control" goal is actually testable — recorded in `docs/decisions.md`, not quietly assumed |

## Epic E5 — Evaluation & gate

| ID | Task | Wave | Owner | Blocks on | Hot path | Acceptance |
|---|---|---|---|---|---|---|
| **T10** | Seeded corpus (labelled) + metrics harness (recall/FP/consistency/latency/zero-raw-PII); incident-log demo fixture; Go/No-Go report generator | D | Squad 7 | T09 | — | report emits all Go/No-Go metrics; demo scenario runs; thresholds evaluated |
| **T11** | Review + `/security-review` on full diff; privacy + supply-chain criteria; sign-off | D | Review Agent | T10 | — | review findings triaged; gates green; human approval recorded |

---

## Cross-cutting (continuous, all waves)

| ID | Task | Owner | Notes |
|---|---|---|---|
| **X-DOCS** | Keep `../spec/`, `decisions.md`, `project-walkthrough.md` current | Documentation/Walkthrough Agent | every material change |
| **X-SUPPLY** | Dependency review on each new crate; SBOM on release; reproducible-build check | Squad X | `cargo-deny` allowlist |
| **X-CONTRACT** | Steward `interface-contracts.md`; process contract-change requests | Squad 0 + Orchestrator | versioned |

---

## Dependency DAG (compact)

```
T01 ─┐
T02 ─┼─> T03 ─┐
     ├─> T04 ─┤
     ├─> T08 ─┤
     ├─> T05 ─┼─> T07 ─> T09 ─> T10 ─> T11
     ├─> T05b ┤
     └─> T06 ─┘
```

## Milestone definition (Phase 1 MVP)
Phase 1 is complete when **T01–T11** are merged, the Go/No-Go report (T10) meets every threshold in `../spec/requirements-and-design-spec.md` § Go/No-Go, and the human has approved the milestone.
