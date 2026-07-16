//! Shannon-entropy detector for high-entropy tokens (API keys, tokens, random secrets)
//! that have no stable format a regex could describe — unlike email/phone/IP/IBAN,
//! secrets vary per provider with no shared shape, so this detector flags *any*
//! sufficiently long, sufficiently random-looking token instead of matching a pattern.
//!
//! **Judgment call (recorded in `docs/decisions.md`):** entropy alone can't distinguish
//! an API key from a password from a generic secret — all it sees is "a long random
//! string" — so every finding is tagged the generic `EntityType::Secret` rather than
//! guessing `ApiKey` from a token shape this detector has no basis to assert.

use vg_core::{Detector, DetectorId, EntityType, Finding, Span};

const ENTITY_TYPES: [EntityType; 1] = [EntityType::Secret];

/// Default minimum token length (bytes) before entropy is even considered. Short
/// strings hit their alphabet's max entropy trivially (e.g. a 4-character token can't
/// exceed 2 bits/char), so a length floor is what actually keeps this detector from
/// flagging short identifiers.
const DEFAULT_MIN_LENGTH: usize = 20;

/// Default Shannon-entropy threshold, in bits per character. Sits between hex-encoded
/// secrets (~4 bits/char) and base64-encoded ones (~6 bits/char, capped by their
/// 64-symbol alphabet), in the same ballpark as common secret-scanner heuristics (e.g.
/// detect-secrets' generic high-entropy-string plugin).
const DEFAULT_THRESHOLD: f64 = 3.5;

/// Bytes treated as part of a token rather than a delimiter: ASCII alphanumerics plus
/// the punctuation that commonly appears *inside* API keys/tokens (base64url and
/// common key-encoding alphabets use these); everything else (whitespace, quotes,
/// commas, etc.) splits tokens apart.
///
/// **Codex cross-model doubt-pass finding (2026-07-15):** `!@#$%^&*` added after the
/// review found that a generated password like `aB3!xY7@qR2#nM8$pL5%zK` (well above the
/// entropy threshold as a whole) was being split at every one of those bytes into
/// sub-20-byte fragments, each individually rejected by `min_length` -- a systematic
/// miss of an entire realistic secret shape, not a hypothetical one, for the one
/// detector whose whole job is catching secrets with no fixed format. Deliberately NOT
/// adding `:`/`;`/`(`/`)` even though they also appear in some compound secrets
/// (`user:token`): those are far more common as genuine field/prose delimiters in logs,
/// timestamps, and URLs, and including them risks merging unrelated adjacent tokens into
/// spurious high-entropy strings (e.g. an ISO timestamp's own digit runs). That
/// narrower gap is accepted as a documented residual, not fixed here — see
/// docs/decisions.md.
///
/// **Round-2 verification pass found two more residuals from including `@`, both
/// accepted, neither fixed:** (1) `@` makes an ordinary email (`jane.doe@example.com`,
/// 20 bytes, ~3.5 bits/byte) sit right at the entropy threshold, so plain emails can
/// ALSO get tagged `Secret` alongside `EmailDetector`'s own, more specific `Email`
/// finding for the same span -- a precision cost (over-flagging), not a leak; downstream
/// can prefer the more specific classification. (2) `@` can merge a real secret with a
/// long low-entropy suffix (e.g. Basic-Auth-in-URL style `<secret>@internal.example.com`)
/// and dilute the merged token's entropy below threshold, a genuine false-negative shape
/// that removing `@` would not fix either (it would just reopen the original password
/// gap this fix closed). Also pre-existing, not introduced by this fix: `=` is a token
/// byte, so a long low-entropy key name merged with `=<secret>` can similarly dilute
/// below threshold. All three are accepted: this is a coarse byte-classification
/// heuristic by design, not a context-aware tokenizer, and each further tweak trades one
/// realistic shape for another rather than strictly improving coverage.
fn is_token_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'+' | b'/'
                | b'='
                | b'-'
                | b'_'
                | b'.'
                | b'!'
                | b'@'
                | b'#'
                | b'$'
                | b'%'
                | b'^'
                | b'&'
                | b'*'
        )
}

/// Excludes tokens that decompose entirely into word-like or short-numeric segments
/// when split on structural delimiters (`/`, `.`, `_`, `-`) -- covers file paths
/// (`scripts/gateway-run.sh`), snake_case/kebab-case identifiers
/// (`requires_confirmation`, `local-coding-harness`), and structured operational IDs
/// (`run-20260608-EG-012`) uniformly, rather than an unstructured secret.
///
/// **2026-07-16 census finding, corrected after measurement:** an initial version of
/// this exclusion assumed Hekton's own `run-YYYYMMDD-EG-NNN` run IDs were the dominant
/// false-positive shape. Measuring the *actual* matched tokens (via a temporary local
/// debug print against real `engine-gateway-lab` content, never committed) showed that
/// assumption was wrong: on that specific fixed content, the fix only removed 1 of 1849
/// entropy findings. The real dominant classes were file paths and
/// snake_case/kebab-case code identifiers -- `is_token_byte` treats `/`, `.`, and `_` as
/// part of a token (not a delimiter), so a whole path or identifier is scored for
/// entropy as one blob, and the resulting mix of letters/case/punctuation clears the
/// threshold even though every piece is an ordinary word. This version fixes that:
/// splitting on the token's own internal delimiters and requiring every resulting piece
/// to be purely alphabetic (any length -- these are word-like labels, not random blobs)
/// or purely numeric (<=8 digits -- dates/ordinals) catches paths and identifiers
/// generically, not via a Hekton-specific dictionary.
///
/// **Accepted residual, not fixed:** a real secret that happens to be a
/// dictionary-word passphrase joined by delimiters (e.g. `correct-horse-battery-staple`)
/// would also be excluded by this rule -- indistinguishable from a real identifier
/// without a dictionary/semantic check this detector doesn't have. A secret whose
/// segments mix letters and digits (the vast majority of real base64/hex/API-key
/// shapes) is unaffected, since a mixed segment is neither purely alphabetic nor purely
/// numeric and fails this exclusion.
fn is_structured_identifier(token: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(token) else {
        return false;
    };
    // Empty segments (a leading/trailing delimiter, e.g. the dotfile in
    // `.hekton/risk-register.yaml`, or a doubled delimiter) are common and not
    // themselves suspicious -- filtered out rather than treated as disqualifying.
    let segments: Vec<&str> = s
        .split(['/', '.', '_', '-'])
        .filter(|seg| !seg.is_empty())
        .collect();
    if segments.len() < 2 {
        return false;
    }
    segments.iter().all(|seg| {
        let bytes = seg.as_bytes();
        let all_alpha = bytes.iter().all(|b| b.is_ascii_alphabetic());
        let all_digit = bytes.iter().all(|b| b.is_ascii_digit());
        all_alpha || (all_digit && bytes.len() <= 8)
    })
}

fn shannon_entropy_bits_per_byte(token: &[u8]) -> f64 {
    let mut counts = [0u32; 256];
    for &b in token {
        counts[b as usize] += 1;
    }
    let len = token.len() as f64;
    counts
        .iter()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = f64::from(count) / len;
            -p * p.log2()
        })
        .sum()
}

/// Maps how far `entropy` sits above `threshold` onto a `0.5..=0.95` confidence range:
/// right at the threshold is a coin flip (0.5), and 2+ bits/char above it is treated as
/// as confident as this detector gets (0.95) — entropy is a continuous heuristic, so a
/// step function at the threshold would discard information the detector actually has.
fn confidence_for(entropy: f64, threshold: f64) -> f32 {
    let scaled = 0.5 + (entropy - threshold) / 2.0 * 0.45;
    scaled.clamp(0.5, 0.95) as f32
}

/// Tunable Shannon-entropy detector: flags whitespace/punctuation-delimited tokens at
/// least `min_length` bytes long whose Shannon entropy meets or exceeds `threshold`
/// bits per byte.
#[derive(Debug, Clone, Copy)]
pub struct EntropyDetector {
    pub min_length: usize,
    pub threshold: f64,
}

impl EntropyDetector {
    pub fn new(min_length: usize, threshold: f64) -> Self {
        Self {
            min_length,
            threshold,
        }
    }
}

impl Default for EntropyDetector {
    fn default() -> Self {
        Self::new(DEFAULT_MIN_LENGTH, DEFAULT_THRESHOLD)
    }
}

impl Detector for EntropyDetector {
    fn id(&self) -> DetectorId {
        DetectorId("entropy".to_string())
    }

    fn detect(&self, buf: &[u8], _spans: &[Span]) -> Vec<Finding> {
        let detector_id = self.id();
        let mut findings = Vec::new();
        let mut idx = 0;
        while idx < buf.len() {
            if !is_token_byte(buf[idx]) {
                idx += 1;
                continue;
            }
            let start = idx;
            while idx < buf.len() && is_token_byte(buf[idx]) {
                idx += 1;
            }
            let token = &buf[start..idx];
            if token.len() < self.min_length {
                continue;
            }
            if is_structured_identifier(token) {
                continue;
            }
            let entropy = shannon_entropy_bits_per_byte(token);
            if entropy >= self.threshold {
                findings.push(Finding {
                    entity_type: EntityType::Secret,
                    span: Span {
                        start,
                        end: idx,
                        node_kind: None,
                    },
                    confidence: confidence_for(entropy, self.threshold),
                    detector: detector_id.clone(),
                });
            }
        }
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
    fn flags_a_long_random_looking_token() {
        let buf = b"export API_KEY=zQ3v9Lm2Xp7RwT6uYbN8dKfJhC1sEoAi";
        let findings = EntropyDetector::default().detect(buf, &[]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].entity_type, EntityType::Secret);
    }

    #[test]
    fn flags_a_password_containing_common_special_characters() {
        // Codex cross-model doubt-pass finding (2026-07-15): before is_token_byte
        // included !@#$%^&*, this password was split at every special character into
        // sub-20-byte fragments, each missed individually -- a whole class of realistic
        // secrets invisible to the one detector meant to catch unshaped ones.
        let buf = b"PASSWORD=aB3!xY7@qR2#nM8$pL5%zK^wQ9&";
        let findings = EntropyDetector::default().detect(buf, &[]);
        assert_eq!(
            findings.len(),
            1,
            "expected one merged high-entropy token, got {findings:?}"
        );
        assert_eq!(findings[0].entity_type, EntityType::Secret);
    }

    #[test]
    fn ignores_structured_identifiers_even_at_a_lenient_threshold() {
        // Lenient params so this test proves the shape-exclusion fires, independent of
        // whether the default 3.5 threshold happens to catch these particular tokens.
        // These are the real dominant false-positive shapes found by the 2026-07-16
        // census against engine-gateway-lab's actual content (file paths and
        // snake_case/kebab-case identifiers), not the Hekton-run-ID shape an earlier,
        // measurement-corrected version of this exclusion assumed was dominant.
        let lenient = EntropyDetector::new(10, 0.5);
        for id in [
            "run-20260608-EG-012",
            "RISK-0017-EG-001",
            "scripts/gateway-run.sh",
            "docs/api-contract.md",
            ".hekton/risk-register.yaml",
            "requires_confirmation",
            "confirm_cloud_egress_or_abort",
            "local-coding-harness",
        ] {
            assert!(
                lenient.detect(id.as_bytes(), &[]).is_empty(),
                "expected {id} to be excluded as a structured identifier"
            );
        }
    }

    #[test]
    fn still_flags_a_real_secret_shaped_with_delimiters_at_a_lenient_threshold() {
        // Same lenient params, but each segment mixes letters and digits (a UUID's hex
        // groups), so no segment is purely alphabetic or purely numeric -- the
        // exclusion must not swallow it.
        let lenient = EntropyDetector::new(10, 0.5);
        let uuid_like = "550e8400-e29b-41d4-a716-446655440000";
        assert!(!lenient.detect(uuid_like.as_bytes(), &[]).is_empty());
    }

    #[test]
    fn ignores_a_low_entropy_repeated_token() {
        let buf = b"filler aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa end";
        assert!(EntropyDetector::default().detect(buf, &[]).is_empty());
    }

    #[test]
    fn ignores_short_high_entropy_tokens_below_min_length() {
        // High entropy but far below the default 20-byte floor.
        assert!(EntropyDetector::default()
            .detect(b"id aB3!9k", &[])
            .is_empty());
    }

    #[test]
    fn ignores_ordinary_english_prose() {
        let buf = b"the quick brown fox jumps over the lazy dog repeatedly";
        assert!(EntropyDetector::default().detect(buf, &[]).is_empty());
    }

    #[test]
    fn a_lower_threshold_flags_tokens_the_default_would_miss() {
        // 20 bytes alternating 'a'/'b': Shannon entropy is exactly 1.0 bit/byte (two
        // equally likely symbols) -- below the default 3.5 threshold, but above a
        // deliberately lenient one.
        let buf = b"token abababababababababababababababab end";
        assert!(EntropyDetector::default().detect(buf, &[]).is_empty());
        let lenient = EntropyDetector::new(10, 0.9);
        assert_eq!(lenient.detect(buf, &[]).len(), 1);
    }

    #[test]
    fn confidence_stays_within_bounds() {
        let buf = b"export API_KEY=zQ3v9Lm2Xp7RwT6uYbN8dKfJhC1sEoAi";
        for finding in EntropyDetector::default().detect(buf, &[]) {
            assert!((0.0..=1.0).contains(&finding.confidence));
        }
    }

    #[test]
    fn satisfies_the_detector_contract() {
        let buf = b"export API_KEY=zQ3v9Lm2Xp7RwT6uYbN8dKfJhC1sEoAi and plain text";
        assert_detector_contract(&EntropyDetector::default(), buf, &[]);
    }

    #[test]
    fn never_panics_on_empty_or_invalid_utf8_input() {
        assert!(EntropyDetector::default().detect(b"", &[]).is_empty());
        let _ = EntropyDetector::default().detect(&[0xFF, 0xFE, 0x00, 0x80], &[]);
    }
}
