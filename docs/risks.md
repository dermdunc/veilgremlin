# Risks: VeilGremlin

## Risk Register

Machine-readable risk state lives in `.hekton/risk-register.yaml`. Keep this
Markdown file as the human-readable explanation of material risks and mitigations.

| ID | Date | Risk | Impact | Likelihood | Mitigation | Status |
|---|---|---|---|---|---|---|
| RISK-0001 | 2026-06-30 | Initial governance baseline needs first human/agent review | Medium | Medium | Run governance preflight and end-session review during the first material session | Open |
| RISK-0002 | 2026-06-30 | Hot-path latency regresses past budget (kills adoption) | High | Medium | Criterion benches gated in CI; p95<25ms assembly / <50ms e2e; arena alloc; compiled regex sets. **Status update 2026-07-14 (T01):** bench harness exists and compiles in CI (`cargo bench --no-run`); there is no hot-path code yet to benchmark, so p95 gating/baseline management is not enforced today — lands with the real detector/parser code in T03/T04, not before. | Open |
| RISK-0003 | 2026-06-30 | Detector recall insufficient (raw PII leaks) | High | Medium | Layered deterministic detectors + warm-path GLiNER; red-team corpus; secret recall ≥99% / PII ≥95% gates; conservative artefact blocking | Open |
| RISK-0004 | 2026-06-30 | False positives degrade utility; developers disable the tool | High | Medium | FP budget <3% reviewed; allow-lists; `vg inspect`/`diff` transparency; <50ms overhead | Open |
| RISK-0005 | 2026-06-30 | Demask re-exposes values to a cloud model | Critical | Low | Destination-typed deny; `remote_model_prompt` hard-deny; model has no vault handle; rehydration is a separate local gate | Open |
| RISK-0006 | 2026-06-30 | Vault key mishandling / plaintext at rest | Critical | Low | OS-keychain key wrap; `zeroize`; never persist key plaintext; SQLCipher AES-256 | Open |
| RISK-0007 | 2026-06-30 | Supply-chain compromise of the privacy binary | Critical | Low | Signed releases, SBOM, reproducible builds, no telemetry, `cargo-deny`/`cargo-audit`, signed policy packs | Open |
| RISK-0008 | 2026-06-30 | Multi-agent build: squads diverge on shared types | Medium | Medium | Contract-first freeze (Wave A); contract-change protocol; ownership rule (edit only your crate) | Open |
| RISK-0009 | 2026-06-30 | Over-claiming GDPR/AI-Act compliance in public copy | High | Low | Binding positioning note in spec/README; "supporting control" language only; review gate | Open |
| RISK-0010 | 2026-06-30 | coderturtle SSH key registered to dermdunc — pushes to coderturtle repos fail over SSH | Low | High | Push over HTTPS+gh token now; register `id_ed25519_coderturtle.pub` on coderturtle, or standardise HTTPS remote | Resolved 2026-07-04 — moot: repo transferred to `dermdunc/veilgremlin` (public); no longer pushed as coderturtle at all |
