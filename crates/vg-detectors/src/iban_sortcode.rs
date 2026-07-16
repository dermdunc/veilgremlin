//! IBAN (ISO 13616) and UK sort-code detector.
//!
//! Regex-only, matching the other pattern-based detectors in this crate: ISO 13616
//! defines a checksum (mod-97 over a rearranged numeric form) that a full validator
//! would verify, but the task scope here is a pattern match like items 1-4, not a
//! checksum validator — false positives on shape-alike alphanumeric strings are an
//! accepted heuristic tradeoff, same as the other four detectors, not a gap specific to
//! this one. Confidence is set slightly below the fixed-format email/IP detectors to
//! reflect the lack of checksum verification.

use std::sync::OnceLock;

use regex::bytes::{Regex, RegexBuilder};
use vg_core::{Detector, DetectorId, EntityType, Finding, Span};

use crate::util::regex_findings;

const ENTITY_TYPES: [EntityType; 2] = [EntityType::Iban, EntityType::SortCode];
const CONFIDENCE: f32 = 0.75;

/// Matches IBANs printed either compact (`GB29NWBK60161331926819`) or in the
/// conventional SWIFT 4-character groups (`GB29 NWBK 6016 1331 9268 19`).
fn iban_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        RegexBuilder::new(concat!(
            r"\b[A-Z]{2}[0-9]{2}(?:[ ][A-Z0-9]{4}){2,7}(?:[ ][A-Z0-9]{1,3})?\b|",
            r"\b[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}\b",
        ))
        .case_insensitive(true)
        .unicode(false)
        .build()
        .expect("iban regex pattern is a valid, tested literal")
    })
}

/// UK sort code: 3 pairs of digits joined by a hyphen or space. A bare 6-digit run
/// (`123456`) is deliberately not matched — with no separator it's indistinguishable
/// from countless other 6-digit numeric fields, and sort codes are conventionally always
/// written with a separator.
fn sort_code_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        RegexBuilder::new(r"\b[0-9]{2}[- ][0-9]{2}[- ][0-9]{2}\b")
            .unicode(false)
            .build()
            .expect("sort code regex pattern is a valid, tested literal")
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IbanSortCodeDetector;

impl Detector for IbanSortCodeDetector {
    fn id(&self) -> DetectorId {
        DetectorId("iban-sortcode".to_string())
    }

    fn detect(&self, buf: &[u8], _spans: &[Span]) -> Vec<Finding> {
        let detector_id = self.id();
        let mut findings = regex_findings(
            iban_pattern(),
            buf,
            EntityType::Iban,
            &detector_id,
            CONFIDENCE,
        );
        findings.extend(regex_findings(
            sort_code_pattern(),
            buf,
            EntityType::SortCode,
            &detector_id,
            CONFIDENCE,
        ));
        findings
    }

    fn entity_types(&self) -> &[EntityType] {
        &ENTITY_TYPES
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vg_core::conformance::assert_detector_contract;

    #[test]
    fn detects_a_compact_iban() {
        let buf = b"send to GB29NWBK60161331926819 please";
        let findings = IbanSortCodeDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::Iban);
    }

    #[test]
    fn detects_a_spaced_iban() {
        let buf = b"iban: GB29 NWBK 6016 1331 9268 19";
        let findings = IbanSortCodeDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::Iban);
    }

    #[test]
    fn detects_a_hyphenated_sort_code() {
        let buf = b"sort code 12-34-56 account";
        let findings = IbanSortCodeDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::SortCode);
    }

    #[test]
    fn detects_a_space_separated_sort_code() {
        let buf = b"sort code 12 34 56 account";
        assert_eq!(IbanSortCodeDetector.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn ignores_a_bare_unseparated_six_digit_run() {
        assert!(IbanSortCodeDetector.detect(b"pin 123456", &[]).is_empty());
    }

    #[test]
    fn detects_both_kinds_in_one_buffer() {
        let buf = b"iban GB29NWBK60161331926819 sort code 12-34-56";
        assert_eq!(IbanSortCodeDetector.detect(buf, &[]).len(), 2);
    }

    #[test]
    fn satisfies_the_detector_contract() {
        assert_detector_contract(
            &IbanSortCodeDetector,
            b"GB29NWBK60161331926819 and 12-34-56 and nonsense",
            &[],
        );
    }
}
