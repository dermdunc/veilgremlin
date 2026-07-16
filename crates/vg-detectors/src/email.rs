//! RFC-5322-ish email address detector.
//!
//! Matches the practical subset of RFC 5322's local-part grammar (unquoted `atext` plus
//! `.`) that shows up in real code/config/logs, not the full quoted-string/comment
//! grammar — no real-world scanner needs to match `"john..doe"@example.com`, and doing
//! so would only widen the false-positive surface.

use std::sync::OnceLock;

use regex::bytes::{Regex, RegexBuilder};
use vg_core::{Detector, DetectorId, EntityType, Finding, Span};

use crate::util::regex_findings;

const ENTITY_TYPES: [EntityType; 1] = [EntityType::Email];
const CONFIDENCE: f32 = 0.9;

fn pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        RegexBuilder::new(
            r"\b[A-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[A-Z0-9](?:[A-Z0-9-]{0,61}[A-Z0-9])?(?:\.[A-Z0-9](?:[A-Z0-9-]{0,61}[A-Z0-9])?)+\b",
        )
        .case_insensitive(true)
        .unicode(false)
        .build()
        .expect("email regex pattern is a valid, tested literal")
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EmailDetector;

impl Detector for EmailDetector {
    fn id(&self) -> DetectorId {
        DetectorId("email".to_string())
    }

    fn detect(&self, buf: &[u8], _spans: &[Span]) -> Vec<Finding> {
        regex_findings(pattern(), buf, EntityType::Email, &self.id(), CONFIDENCE)
    }

    fn entity_types(&self) -> &[EntityType] {
        &ENTITY_TYPES
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vg_core::conformance::assert_detector_contract;

    fn matched_text<'a>(buf: &'a [u8], finding: &Finding) -> &'a [u8] {
        &buf[finding.span.start..finding.span.end]
    }

    #[test]
    fn detects_a_plain_email_in_surrounding_text() {
        let buf = b"contact jane.doe+test@example.co.uk today";
        let findings = EmailDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::Email);
        assert_eq!(
            matched_text(buf, &findings[0]),
            b"jane.doe+test@example.co.uk"
        );
    }

    #[test]
    fn detects_multiple_distinct_emails() {
        let buf = b"cc: a@example.com, b@example.org";
        let findings = EmailDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn ignores_an_at_sign_without_a_dotted_domain() {
        assert!(EmailDetector.detect(b"user@localhost", &[]).is_empty());
        assert!(EmailDetector.detect(b"nothing here", &[]).is_empty());
    }

    #[test]
    fn is_case_insensitive() {
        let buf = b"JANE.DOE@EXAMPLE.COM";
        assert_eq!(EmailDetector.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn never_panics_on_invalid_utf8_or_empty_input() {
        assert!(EmailDetector.detect(b"", &[]).is_empty());
        let _ = EmailDetector.detect(&[0xFF, 0xFE, b'@', 0x00], &[]);
    }

    #[test]
    fn satisfies_the_detector_contract() {
        assert_detector_contract(
            &EmailDetector,
            b"jane@example.com and not-an-email and @nope",
            &[],
        );
    }
}
