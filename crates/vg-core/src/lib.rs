//! `vg-core` — shared types and library API for VeilGremlin.
//!
//! Owns the canonical definitions described in `docs/architecture/interface-contracts.md`
//! (`EntityType`, `HandlingClass`, `Namespace`, `Finding`, `MaskedPack`, `scan`/`mask`/
//! `rehydrate`/`benchmark`) and the trait seams other crates implement against
//! (`Detector`, `Parser`, `VaultStore`, `PolicyEngine`, `AuditSink`).
//!
//! Scaffolded in Task T01 (workspace + CI). Frozen in Task T02, per the contract-change
//! protocol in `docs/architecture/agent-factory-plan.md` §6.
