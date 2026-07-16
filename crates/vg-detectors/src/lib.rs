//! `vg-detectors` — hot-path detectors implementing `vg_core::Detector`.
//!
//! Deterministic, allocation-aware, no I/O, no network, no ML (warm-path NER lives
//! behind a separate `Enricher` trait, never here). Benchmarked: p95 < 25ms on the
//! reference buffer (`docs/architecture/interface-contracts.md` §3).
//!
//! Scaffolded in Task T01; five deterministic detectors landed in Task T03:
//! [`email::EmailDetector`], [`phone::PhoneDetector`], [`ip::IpDetector`],
//! [`iban_sortcode::IbanSortCodeDetector`], [`entropy::EntropyDetector`].
//! [`all_detectors`] is the composition point downstream code enumerates.

pub mod email;
pub mod entropy;
pub mod iban_sortcode;
pub mod ip;
pub mod phone;
mod util;

use vg_core::Detector;

/// All deterministic detectors this crate provides, as trait objects — the composition
/// point downstream code (`vg-core::Context`, the CLI, benches) enumerates rather than
/// naming each detector individually.
pub fn all_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(email::EmailDetector),
        Box::new(phone::PhoneDetector),
        Box::new(ip::IpDetector),
        Box::new(iban_sortcode::IbanSortCodeDetector),
        Box::new(entropy::EntropyDetector::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_detectors_returns_all_five() {
        assert_eq!(all_detectors().len(), 5);
    }

    #[test]
    fn every_detector_id_is_unique() {
        let ids: Vec<_> = all_detectors().iter().map(|d| d.id()).collect();
        let mut deduped = ids.clone();
        deduped.sort_by(|a, b| a.0.cmp(&b.0));
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }
}
