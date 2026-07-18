//! `vg-policy` — 3-layer policy engine implementing `vg_core::PolicyEngine`.
//!
//! [`LayeredPolicyEngine`] resolves up to three policy packs
//! (session-overrides-repo-overrides-global) into one validated policy, then answers the
//! `PolicyEngine` queries: entity/artefact classification, the masked-only send gate, and
//! the demask authorisation gate.
//!
//! Policy packs are serde-deserialised config (JSON in Phase 1 — see
//! `docs/decisions.md` 2026-07-17 for why JSON rather than TOML/YAML, and how to swap the
//! format later). Example packs live in `fixtures/`.
//!
//! Two contract points from `docs/architecture/interface-contracts.md` §6:
//!
//! - **Signed-pack verification** is a deliberate Phase 1 stub
//!   ([`config::verify_signature`]) that always accepts; Phase 2 replaces it.
//! - **`demask_allowed` hard-denies `RemoteModelPrompt` and `ObservabilitySink`** in code,
//!   regardless of actor or pack contents. This is the one security-load-bearing rule
//!   here; everything else is configuration plumbing. It is checked in the test suite via
//!   `vg_core::conformance::assert_policy_engine_denies_hard_deny_destinations`.
//!
//! Scaffolded in Task T01; implemented in Task T06.

mod config;
mod engine;

pub use config::{entity_key, parse_class, verify_signature};
pub use engine::LayeredPolicyEngine;
