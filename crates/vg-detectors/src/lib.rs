//! `vg-detectors` — hot-path detectors implementing `vg_core::Detector`.
//!
//! Deterministic, allocation-aware, no I/O, no network, no ML (warm-path NER lives
//! behind a separate `Enricher` trait, never here). Benchmarked: p95 < 25ms on the
//! reference buffer (`docs/architecture/interface-contracts.md` §3).
//!
//! Scaffolded in Task T01; detector implementations land in Tasks T03/T04.
