//! E.164-ish phone number detector.
//!
//! ITU-T E.164 itself is just `+` followed by 7-15 digits with no separators, but almost
//! no real-world text writes numbers that way — they're grouped with spaces, dots,
//! hyphens, and/or parentheses. The regex below matches the loosely-grouped form (a
//! leading digit group, optionally parenthesised, followed by more digit groups joined
//! by common separators), and `detect` then filters candidates down to:
//!
//! 1. a total digit count in E.164's 7-15 range, and
//! 2. at least one phone-typical marker (`+`, parens, or a separator) — a bare
//!    unseparated run of digits is far more likely to be a generic numeric ID than a
//!    phone number, so it's excluded unless it starts with `+`.
//!
//! This is a heuristic by construction — the task spec calls for "E.164-ish", not a
//! validator — and it will still false-positive on separator-shaped numeric strings
//! that happen to fall in the digit-count range. This detector's confidence is set
//! below the fixed-format detectors' (email/IP/IBAN) to reflect that extra ambiguity;
//! see `docs/decisions.md`.
//!
//! **2026-07-16 census finding:** a real 197-file scan of Hekton's own repos found the
//! `YYYY-MM-DD` case above wasn't hypothetical — it was the dominant false-positive
//! class (783 of the census's phone findings), since dates are common in prose and an
//! 8-digit, 4/2/2-grouped, calendar-valid date is common enough to swamp real numbers.
//! Fixed narrowly via `looks_like_iso_date`: excludes only the strict ISO date shape,
//! not arbitrary grouped numbers, per the Codex-reconciled decision in
//! `docs/decisions.md` to fix detector-local false positives now rather than wait for
//! Task T10's formal precision measurement.

use std::sync::OnceLock;

use regex::bytes::{Regex, RegexBuilder};
use vg_core::{Detector, DetectorId, EntityType, Finding, Span};

const ENTITY_TYPES: [EntityType; 1] = [EntityType::Phone];
const CONFIDENCE: f32 = 0.7;
const MIN_DIGITS: usize = 7;
const MAX_DIGITS: usize = 15;

fn pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        RegexBuilder::new(r"\+?\(?[0-9]{1,4}\)?(?:[-. ]?[0-9]{2,4}){1,5}")
            .unicode(false)
            .build()
            .expect("phone regex pattern is a valid, tested literal")
    })
}

fn has_phone_marker(matched: &[u8]) -> bool {
    matched
        .iter()
        .any(|b| matches!(b, b'+' | b'(' | b')' | b'-' | b'.' | b' '))
}

/// Excludes matches shaped like an ISO-ish calendar date (`YYYY-MM-DD` or
/// `YYYY.MM.DD`) rather than a phone number: 8 digits split 4/2/2 with a
/// plausible year/month/day is indistinguishable from a short local number by
/// digit-count and marker alone, but real phone numbers essentially never fall
/// into exactly this grouping with a calendar-valid month and day. Deliberately
/// narrow (2026-07-16 census finding, see `docs/decisions.md`): only the strict
/// 4/2/2 date shape is excluded, not arbitrary grouped numbers, so this doesn't
/// eat real short local numbers that happen to share a digit count.
fn looks_like_iso_date(matched: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(matched) else {
        return false;
    };
    let groups: Vec<&str> = s.split(['-', '.']).collect();
    let [year, month, day] = groups.as_slice() else {
        return false;
    };
    if year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return false;
    }
    if !year.bytes().all(|b| b.is_ascii_digit())
        || !month.bytes().all(|b| b.is_ascii_digit())
        || !day.bytes().all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    (1900..=2099).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PhoneDetector;

impl Detector for PhoneDetector {
    fn id(&self) -> DetectorId {
        DetectorId("phone".to_string())
    }

    fn detect(&self, buf: &[u8], _spans: &[Span]) -> Vec<Finding> {
        let detector_id = self.id();
        pattern()
            .find_iter(buf)
            .filter_map(|m| {
                let matched = m.as_bytes();
                let digit_count = matched.iter().filter(|b| b.is_ascii_digit()).count();
                if !(MIN_DIGITS..=MAX_DIGITS).contains(&digit_count)
                    || !has_phone_marker(matched)
                    || looks_like_iso_date(matched)
                {
                    return None;
                }
                Some(Finding {
                    entity_type: EntityType::Phone,
                    span: Span {
                        start: m.start(),
                        end: m.end(),
                        node_kind: None,
                    },
                    confidence: CONFIDENCE,
                    detector: detector_id.clone(),
                })
            })
            .collect()
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
    fn detects_an_international_number_with_country_code() {
        let buf = b"call +1-415-555-2671 now";
        let findings = PhoneDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::Phone);
    }

    #[test]
    fn detects_a_parenthesised_local_number() {
        let buf = b"office: (020) 7946 0958";
        assert_eq!(PhoneDetector.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn detects_a_short_hyphenated_local_number() {
        let buf = b"reach me at 555-1234";
        assert_eq!(PhoneDetector.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn ignores_too_few_digits() {
        assert!(PhoneDetector.detect(b"dial 911", &[]).is_empty());
        assert!(PhoneDetector.detect(b"room 12-34", &[]).is_empty());
    }

    #[test]
    fn ignores_a_bare_unseparated_digit_run_without_a_plus() {
        // No separators and no leading '+' -- too ambiguous with a generic numeric ID.
        assert!(PhoneDetector
            .detect(b"order id 415555267188", &[])
            .is_empty());
    }

    #[test]
    fn accepts_a_bare_digit_run_with_a_leading_plus() {
        assert_eq!(PhoneDetector.detect(b"+14155552671", &[]).len(), 1);
    }

    #[test]
    fn ignores_too_many_digits() {
        assert!(PhoneDetector
            .detect(b"ref-123456789012345678", &[])
            .is_empty());
    }

    #[test]
    fn ignores_an_iso_date() {
        assert!(PhoneDetector
            .detect(b"session started on 2026-07-16 and ran late", &[])
            .is_empty());
        assert!(PhoneDetector.detect(b"logged 2026.07.16", &[]).is_empty());
    }

    #[test]
    fn ignores_iso_dates_across_the_full_calendar_year() {
        for (y, m, d) in [(2026, 1, 31), (2026, 12, 1), (1999, 6, 15), (2099, 2, 28)] {
            let s = format!("{y:04}-{m:02}-{d:02}");
            assert!(
                PhoneDetector.detect(s.as_bytes(), &[]).is_empty(),
                "expected {s} to be excluded as a date"
            );
        }
    }

    #[test]
    fn still_detects_a_real_number_with_the_same_digit_count_as_a_date() {
        // 8 digits, 3 groups, but month/day out of calendar range -- not a date shape.
        assert_eq!(PhoneDetector.detect(b"call 9999-99-99 now", &[]).len(), 1);
    }

    #[test]
    fn satisfies_the_detector_contract() {
        assert_detector_contract(&PhoneDetector, b"+1-415-555-2671 and 555-1234", &[]);
    }
}
