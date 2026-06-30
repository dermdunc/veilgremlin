# Next Actions: VeilGremlin

Repo source-of-truth for the work queue. Tasks T01–T11 are defined in [`architecture/work-breakdown.md`](architecture/work-breakdown.md); the build method is in [`architecture/agent-factory-plan.md`](architecture/agent-factory-plan.md).

## Immediate (Wave A — foundation, must merge before parallel work)

- [ ] **Confirm first push** to GitHub: `cd /Users/hekton/Development/hekton/factory-output/veilgremlin && git push -u origin main` (see note in session-log re: SSH key — coderturtle key is currently registered to dermdunc; HTTPS+gh token works)
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
- [ ] Decide repo visibility flip (private → public) when ready to open-source
- [ ] Fix coderturtle SSH key registration (or standardise on HTTPS remote) so pushes don't need the gh-token workaround
