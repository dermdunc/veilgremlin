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
