# Decisions: VeilGremlin

Condensed mirror of the repo's own `docs/decisions.md` (canonical, full rationale there).

| Date | Decision | Rationale |
|---|---|---|
| 2026-06-30 | Initial scaffold as factory-output (Hekton) | Local-first privacy shield; no `hekton-` prefix per taxonomy |
| 2026-06-30 | Repo under coderturtle GitHub account, private initially | Superseded 2026-07-04, see below |
| 2026-06-30 | ADR-001 Core language = Rust | Memory/thread safety, small trusted core |
| 2026-06-30 | ADR-002 Local vault = SQLCipher SQLite | Encrypted, local, queryable |
| 2026-06-30 | ADR-003 Detector mix = deterministic hot path + optional GLiNER warm path | Latency + explainability + recall balance |
| 2026-06-30 | ADR-004 First integration = Claude Code wrapper + hooks on Bedrock | Fastest enterprise proof |
| 2026-06-30 | ADR-005 Masking = typed placeholders, not synthetic values | Transparent, stable, auditable |
| 2026-06-30 | ADR-006 Demasking = explicit, local, policy-gated | Prevents re-exposure to cloud models |
| 2026-06-30 | ADR-007 Policy = native YAML/TOML now; Cedar later | Low dependency now; strong auth later |
| 2026-06-30 | ADR-008 Gateway = LiteLLM later; core stays separate | Hardened small core + provider-independence |
| 2026-06-30 | ADR-009 Supply chain = sign + SBOM + reproducible builds + no telemetry | Trust prerequisite for a privacy binary |
| 2026-06-30 | ADR-010 Placeholder key = salted HMAC over canonicalised value+type+namespace | Stable consistency without leaking structure |
| 2026-06-30 | Build method = agent factory, contract-first | Squad-per-crate ownership; interfaces frozen end of Wave A |
| 2026-07-03 | Build driven through Hekton's task-DAG orchestrator (`agentic-control-tower`) | Not manual per-task dispatch; see ADR-013 there |
| 2026-07-04 | Repo ownership moved from coderturtle (private) to dermdunc (public) | Enterprise architecture/governance/risk tool belongs under the professional-identity account |
