//! `vg-policy` — 3-layer policy engine implementing `vg_core::PolicyEngine`.
//!
//! Resolution order: session overrides repo overrides global. Signed-pack
//! verification is a stub in Phase 1, enforced later. `demask_allowed` must return
//! false for `RemoteModelPrompt`/`ObservabilitySink` in default policy
//! (`docs/architecture/interface-contracts.md` §6).
//!
//! Scaffolded in Task T01; implementation lands in Task T06.
