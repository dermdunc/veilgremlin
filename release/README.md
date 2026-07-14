# Release skeleton (Task T01)

Placeholder for VeilGremlin's release pipeline. Nothing here runs yet — it exists so
the workspace has a named home for supply-chain hardening before real releases start,
per `docs/architecture/agent-factory-plan.md`'s Squad X scope ("workspace, cargo-deny,
SBOM, release").

## SBOM (stub)

Real releases will generate a CycloneDX SBOM via `cargo cyclonedx` (or `cargo auditable`)
per workspace member and attach it to the GitHub release. Not wired into CI yet — `deny.toml`
and the `cargo-deny`/`cargo-audit` CI jobs are the supply-chain gate for now.

## Signing (stub)

Real releases will sign build artifacts (e.g. via `cosign` or `minisign`) and publish the
signature alongside the binary. No signing key exists yet; this is a placeholder for the
decision, not an implementation.

## What's real today

- `deny.toml` — cargo-deny license/advisory/source policy, enforced in CI.
- `cargo-audit` CI job — RustSec advisory scan.
- `--locked` everywhere — no silent dependency drift between a local build and CI.

Both the SBOM and signing steps are explicitly deferred, not silently skipped — tracked in
`docs/next-actions.md`.
