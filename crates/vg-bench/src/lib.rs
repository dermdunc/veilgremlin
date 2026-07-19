//! `vg-bench` — the VeilGremlin eval harness: the labelled seeded corpus loader, the
//! Go/No-Go report (the six banked T10 measurements), and its redaction-safe rendering.
//!
//! Scaffolded in Task T01 (workspace + a bench CI job). Task T10 wired the real harness:
//! a labelled corpus ([`corpus`]) scored through the same pipeline the product runs, the
//! frozen `vg_core::benchmark` for the headline [`vg_core::Metrics`], and a report layer
//! ([`report`]) that adds the six banked measurements the frozen `Metrics` shape can't
//! carry — per-detector false-positive rates, the zero-raw-PII property, display-collision
//! incidence, dotenv-no-hint entity recall, cold-hook e2e latency, and dead-policy-branch
//! detection.
//!
//! **The harness satisfies the redaction discipline it measures:** no raw detected value
//! ever reaches report output (see [`render`]). The `vg bench` subcommand drives
//! [`Report::generate`] and prints [`render::render`].

pub mod corpus;
pub mod render;
pub mod report;

pub use corpus::{CorpusError, LabelledCorpus, LabelledSample};
pub use render::render;
pub use report::{Options, Report, ReportError, Verdict};

/// Convenience: run the full report over the embedded seeded corpus with `opts`.
pub fn run(opts: &Options) -> Result<Report, ReportError> {
    let corpus = LabelledCorpus::embedded()?;
    Report::generate(&corpus, opts)
}
