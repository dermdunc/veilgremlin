# Next Actions: VeilGremlin

Repo source-of-truth for the work queue. Tasks T01–T11 are defined in [`architecture/work-breakdown.md`](architecture/work-breakdown.md); the build method is in [`architecture/agent-factory-plan.md`](architecture/agent-factory-plan.md).

## Immediate (Wave A — foundation, must merge before parallel work)

- [x] **Confirm first push** to GitHub — resolved: `origin` now uses the `github.com-coderturtle`
      SSH host alias (a real fix, not the HTTPS+gh-token workaround); `main` is pushed and
      confirmed via `git ls-remote --heads origin` (2026-07-04).
- [x] **Repo ownership move to dermdunc — complete (2026-07-04).** Transferred via
      `gh api repos/coderturtle/veilgremlin/transfer -f new_owner=dermdunc`, human-accepted as
      dermdunc, visibility flipped to public
      (`gh repo edit dermdunc/veilgremlin --visibility public --accept-visibility-change-consequences`),
      local `origin` repointed to `git@github.com:dermdunc/veilgremlin.git`, and confirmed
      reachable via `git ls-remote origin`. `.hekton/project.yaml`, `.hekton/governance.yaml`,
      `.hekton/risk-register.yaml`, the repo-local mind-palace mirror, and the `Owner:`/`Privacy
      boundary:` headers in `README.md`/`CLAUDE.md`/`AGENTS.md`/`CODEX.md`/`docs/spec/...` all
      updated to match. See `docs/decisions.md` for the full record.
- [ ] **New blocker found 2026-07-04: no Rust toolchain on this machine.** `cargo`/`rustc` not
      found (checked `~/.cargo/bin`, `rustup`, and Homebrew — `rust` formula available but not
      installed). T01's own verify command is `cargo build --locked && cargo fmt --check`, so
      dispatch will fail immediately until this is fixed. `scripts/check-prereqs.sh` doesn't check
      for it either (still the generic scaffold default: git/bash/sed/date/mkdir/printf only).
      Needs: Rust toolchain (brew or rustup) + `cargo-deny` + `cargo-audit`, and
      `check-prereqs.sh` updated to check for all three so this doesn't get silently rediscovered
      again.
- [ ] **Human decision: dispatch T01 + T02 for real** through the new task-DAG orchestrator —
      `control-tower --root . dag dispatch T01` and `dag dispatch T02` (both show `ready` in
      `dag status`, confirmed 2026-07-04). The DAG (`.hekton/veilgremlin-dag.toml`) and generated
      specs (`.hekton/build-tasks/T01.md`, `T02.md`) are already in place and validated; this
      repo's own `.control-tower/` workspace is initialized. See `docs/decisions.md` (2026-07-03)
      and `agentic-control-tower`'s root `decisions.md` ADR-013 for the full mechanism.
      **Deliberately deferred again 2026-07-04**: coderturtle wants to hold off starting real
      VeilGremlin build work until the wider Hekton factory-readiness pass (see `~/hekton`'s
      2026-07-04 decision accepting the 8-layer OS model) is further along, not just T01/T02's own
      prerequisites. Resolve the Rust toolchain gap above whenever that build session actually
      starts, not before.
- [ ] **T01** — Cargo workspace + crate skeletons + CI (`fmt`, `clippy -D warnings`, `cargo-deny`, `cargo-audit`, `--locked`) + bench/release skeleton — *Squad X*
- [ ] **T02** — Freeze shared types + library API in `vg-core`; finalise `architecture/interface-contracts.md` v1 — *Squad 0*

## This Week (Wave B — dispatch in parallel once T01+T02 merge)

- [ ] **T03/T04** detectors + placeholder/HMAC keying — *Squad 1 (+3)*
- [ ] **T08** parsers (logs, diffs, JSON/YAML, `.env`, tree-sitter) — *Squad 2*
- [ ] **T05** SQLCipher vault + keychain wrap + TTL — *Squad 3*
- [ ] **T05b** audit sink (append-only, redaction-safe) — *Squad 5*
- [ ] **T06** native 3-layer policy engine — *Squad 4*

## Later (Wave C/D)

- [ ] **T07** masking pipeline wiring in `vg-core` — *Squad 0*
- [ ] **T09** `vg` CLI + Claude Code adapter + Bedrock path + demask gate — *Squad 6*
- [ ] **T10** seeded corpus + eval harness + Go/No-Go report — *Squad 7*
- [ ] **T11** review + `/security-review` + milestone sign-off — *Review Agent + human*
- [x] Decide repo visibility flip (private → public) — decided 2026-07-04: public, as part of
      the dermdunc ownership move above, not deferred to a later open-source milestone.
- [ ] ~~Fix coderturtle SSH key registration~~ — superseded 2026-07-04: the repo is moving off
      coderturtle entirely, so this is moot for VeilGremlin (the SSH host alias itself
      (`github.com-coderturtle`) remains fine for other coderturtle-owned repos).
