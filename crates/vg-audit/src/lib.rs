//! `vg-audit` — append-only, redaction-safe audit sink implementing `vg_core::AuditSink`.
//!
//! No raw values in any `AuditEvent` variant — refs/counts/versions only, property-tested
//! (`docs/architecture/interface-contracts.md` §7).
//!
//! Scaffolded in Task T01; implementation lands in Task T05b.
