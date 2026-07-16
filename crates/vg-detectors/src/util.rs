//! Shared helper for the regex-based detectors (email/phone/ip/iban_sortcode): each one
//! is "run a compiled pattern over the buffer, emit one `Finding` per non-overlapping
//! match" — this collapses that boilerplate into one place instead of repeating it four
//! times with the same field wiring.

use regex::bytes::Regex;
use vg_core::{DetectorId, EntityType, Finding, Span};

/// Runs `re` over `buf` and returns one `Finding` per non-overlapping match, tagged with
/// `entity_type` and `detector_id` at a fixed `confidence`.
///
/// Uses `regex::bytes::Regex` (not the `&str` API) deliberately: matches are byte spans
/// directly into `buf`, so they stay valid even when `buf` is not valid UTF-8 — the `str`
/// API would require a lossy conversion first, which can shift byte offsets relative to
/// the original buffer and violate the `Detector` contract's span-bounds invariant.
pub(crate) fn regex_findings(
    re: &Regex,
    buf: &[u8],
    entity_type: EntityType,
    detector_id: &DetectorId,
    confidence: f32,
) -> Vec<Finding> {
    re.find_iter(buf)
        .map(|m| Finding {
            entity_type: entity_type.clone(),
            span: Span {
                start: m.start(),
                end: m.end(),
                node_kind: None,
            },
            confidence,
            detector: detector_id.clone(),
        })
        .collect()
}
