//! IPv4 and IPv6 address detector.
//!
//! **Judgment call (recorded in `docs/decisions.md`):** `interface-contracts.md`'s
//! `EntityType` enum has `InternalIp` but no generic "IP address" variant, and this
//! detector has no way to tell an internal address from a public one from shape alone.
//! Rather than invent a new fixed variant (which needs the contract-change protocol,
//! not a T03 judgment call) or misuse `Custom(String)` for something that isn't
//! policy-dictionary-defined, both IPv4 and IPv6 findings are tagged `InternalIp` here —
//! it's the closest existing fixed classification, and downstream policy already treats
//! it as a maskable network identifier either way.

use std::sync::OnceLock;

use regex::bytes::{Regex, RegexBuilder};
use vg_core::{Detector, DetectorId, EntityType, Finding, Span};

use crate::util::regex_findings;

const ENTITY_TYPES: [EntityType; 1] = [EntityType::InternalIp];
const CONFIDENCE: f32 = 0.85;

const IPV4_OCTET: &str = r"(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])";

fn ipv4_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let pattern = format!(r"\b{IPV4_OCTET}(?:\.{IPV4_OCTET}){{3}}\b");
        RegexBuilder::new(&pattern)
            .unicode(false)
            .build()
            .expect("ipv4 regex pattern is a valid, tested literal")
    })
}

/// The standard fully-general IPv6 literal pattern (all 9 valid `::`-compression forms,
/// alternated longest/most-specific first), plus IPv4-mapped IPv6 (`::ffff:192.0.2.1`,
/// RFC 4291 §2.5.5.2) as the FIRST alternative.
///
/// **Codex cross-model doubt-pass fix (2026-07-15):** this used to say "deliberately does
/// not attempt embedded IPv4-in-IPv6 notation" on the theory that it would simply fail to
/// match and fall through as plaintext — that was wrong. The generic hex-group
/// alternatives below still partially match the leading `::ffff:` (`(?:[A-Fa-f0-9]{1,4}:)
/// {1,7}:` matches `::ffff:` itself, and — worse — a subsequent alternative can absorb
/// the IPv4 portion's first octet as if it were one more hex group, e.g. matching
/// `::ffff:192` out of `::ffff:192.168.1.1`). A partial match is worse than a clean miss
/// for a redaction tool: it leaves most of the real address (`.168.1.1`) sitting in the
/// buffer right next to a redaction marker, which is misleading about what was actually
/// masked. Regex alternation in this crate (`regex`) is leftmost-first, not
/// leftmost-longest, so the explicit IPv4-mapped pattern must come FIRST to be tried
/// before the generic hex-group alternatives can grab a partial prefix.
///
/// **Round-2 verification pass found this only covers the specific `::ffff:` prefix
/// (the still-common IPv4-mapped form), not other, rarer embedded-IPv4 shapes** --
/// `2001:db8::192.168.1.1` (an IPv4-compatible embedding, deprecated by RFC 4291 itself
/// since 2006) and a malformed `::ffff:0:192.168.1.1` still produce the same partial-
/// match/overlap behavior this fix was meant to close, just for a rarer shape. Accepted,
/// not fixed: the deprecated/malformed forms are rare enough that generalizing this
/// pattern further is judged not worth the added regex complexity, matching this
/// detector's original scope call on embedded-IPv4 notation generally.
fn ipv6_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let pattern = format!(
            concat!(
                r"::[Ff]{{4}}:{ipv4}|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{7}}[A-Fa-f0-9]{{1,4}}|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{1,7}}:|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{1,6}}:[A-Fa-f0-9]{{1,4}}|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{1,5}}(?::[A-Fa-f0-9]{{1,4}}){{1,2}}|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{1,4}}(?::[A-Fa-f0-9]{{1,4}}){{1,3}}|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{1,3}}(?::[A-Fa-f0-9]{{1,4}}){{1,4}}|",
                r"(?:[A-Fa-f0-9]{{1,4}}:){{1,2}}(?::[A-Fa-f0-9]{{1,4}}){{1,5}}|",
                r"[A-Fa-f0-9]{{1,4}}:(?:(?::[A-Fa-f0-9]{{1,4}}){{1,6}})|",
                r":(?:(?::[A-Fa-f0-9]{{1,4}}){{1,7}}|:)",
            ),
            ipv4 = IPV4_OCTET.to_string() + &format!(r"(?:\.{IPV4_OCTET}){{3}}"),
        );
        RegexBuilder::new(&pattern)
            .unicode(false)
            .build()
            .expect("ipv6 regex pattern is a valid, tested literal")
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IpDetector;

impl Detector for IpDetector {
    fn id(&self) -> DetectorId {
        DetectorId("ip".to_string())
    }

    fn detect(&self, buf: &[u8], _spans: &[Span]) -> Vec<Finding> {
        let detector_id = self.id();
        let ipv4_findings = regex_findings(
            ipv4_pattern(),
            buf,
            EntityType::InternalIp,
            &detector_id,
            CONFIDENCE,
        );
        let ipv6_findings = regex_findings(
            ipv6_pattern(),
            buf,
            EntityType::InternalIp,
            &detector_id,
            CONFIDENCE,
        );
        // The IPv4-mapped IPv6 fix (2026-07-15) means an address like
        // `::ffff:192.168.1.1` now correctly matches ipv6_pattern() in full -- but its
        // embedded `192.168.1.1` substring is ALSO a standalone, word-bounded match for
        // ipv4_pattern(), producing two overlapping findings for the same real address.
        // Drop an ipv4 finding whenever it's fully contained inside an ipv6 finding's
        // span, rather than double-flagging one address.
        let mut findings: Vec<Finding> = ipv4_findings
            .into_iter()
            .filter(|v4| {
                !ipv6_findings
                    .iter()
                    .any(|v6| v6.span.start <= v4.span.start && v4.span.end <= v6.span.end)
            })
            .collect();
        findings.extend(ipv6_findings);
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

    fn matched_text<'a>(buf: &'a [u8], finding: &Finding) -> &'a [u8] {
        &buf[finding.span.start..finding.span.end]
    }

    #[test]
    fn detects_a_plain_ipv4_address() {
        let buf = b"connect to 192.168.1.42 for the gateway";
        let findings = IpDetector.detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::InternalIp);
        assert_eq!(matched_text(buf, &findings[0]), b"192.168.1.42");
    }

    #[test]
    fn rejects_octets_over_255() {
        // 999 is not a valid octet; the bounded regex must not match the whole run.
        assert!(IpDetector.detect(b"code 999.999.999.999", &[]).is_empty());
    }

    #[test]
    fn detects_a_full_ipv6_address() {
        let buf = b"host 2001:0db8:85a3:0000:0000:8a2e:0370:7334 online";
        assert_eq!(IpDetector.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn detects_a_compressed_ipv6_address() {
        let buf = b"loopback ::1 here";
        assert_eq!(IpDetector.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn detects_both_versions_in_one_buffer() {
        let buf = b"v4 10.0.0.1 and v6 fe80::1";
        assert_eq!(IpDetector.detect(buf, &[]).len(), 2);
    }

    #[test]
    fn ignores_a_non_numeric_dotted_hostname() {
        assert!(IpDetector.detect(b"see www.example.com", &[]).is_empty());
    }

    #[test]
    fn detects_a_full_ipv4_mapped_ipv6_address_not_a_truncated_prefix() {
        // Codex cross-model doubt-pass finding (2026-07-15): the generic hex-group
        // alternatives used to partially match just "::ffff:192" out of this address,
        // leaving ".168.1.1" sitting unredacted right next to the match -- worse than a
        // clean miss for a redaction tool. Must match the FULL literal now.
        let buf = b"remote=::ffff:192.168.1.1 connected";
        let findings = IpDetector.detect(buf, &[]);
        assert_eq!(
            findings.len(),
            1,
            "expected exactly one finding, got {findings:?}"
        );
        assert_eq!(
            matched_text(buf, &findings[0]),
            b"::ffff:192.168.1.1",
            "must match the full address, not a truncated prefix"
        );
    }

    #[test]
    fn a_dotted_version_string_is_a_known_false_positive() {
        // Four-dot-separated small numbers are shape-identical to an IPv4 address --
        // this is an accepted heuristic limitation (see docs/decisions.md), not a bug:
        // "1.2.3.4" out of "chapter 1.2.3.4.5" is indistinguishable from a real address
        // by regex alone.
        let findings = IpDetector.detect(b"chapter 1.2.3.4.5", &[]);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn satisfies_the_detector_contract() {
        assert_detector_contract(&IpDetector, b"10.0.0.1 and fe80::1 and nonsense", &[]);
    }
}
