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
/// this exclusion assumed structured internal run IDs were the dominant
/// false-positive shape. Measuring the *actual* matched tokens (via a temporary local
/// debug print against real internal-tooling content, never committed) showed that
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
/// **`=` added to the split set (2026-07-21, closing a T10 residual surfaced by re-running
/// `vg bench` after the commit-SHA/ISBN fixes below: `docs/decisions.md`).** `is_token_byte`
/// treats `=` as part of a token (needed for real `KEY=<secret>` shapes where the value
/// itself is high-entropy), so a benign env-style assignment whose value is ALSO
/// word/numeric-shaped -- `LICENSE_KEY=ACME-2026-DEMO-KEY` -- was being scored as one
/// entropy blob across the `=` boundary instead of recognized as structured. Splitting on
/// `=` here (a decomposition-only concern, separate from `is_token_byte`'s tokenization)
/// lets each side of an assignment get evaluated on its own segments, without changing what
/// counts as "one token" for entropy scoring or matching. **Does not create a new residual
/// class:** a real secret whose *value* segments are only alphabetic after the `=` split
/// (e.g. a dictionary-word passphrase) was already excluded by this function before `=` was
/// added -- the same accepted trade-off below, just reachable across one more delimiter, not
/// a new one. A value segment that mixes letters and digits (the vast majority of real
/// secret shapes) still fails `all_alpha`/`all_digit` and is unaffected.
fn is_structured_identifier(token: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(token) else {
        return false;
    };
    // Empty segments (a leading/trailing delimiter, e.g. the dotfile in
    // `.hekton/risk-register.yaml`, or a doubled delimiter) are common and not
    // themselves suspicious -- filtered out rather than treated as disqualifying.
    let segments: Vec<&str> = s
        .split(['/', '.', '_', '-', '='])
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

/// Case-insensitive markers that, immediately before a hex-shaped token (skipping a
/// single separator like `:`/`=`/whitespace), corroborate "this is a git object hash",
/// not a secret.
const GIT_SHA_CONTEXT_WORDS: [&[u8]; 6] = [b"commit", b"sha1", b"sha256", b"sha", b"rev", b"hash"];

/// Whether the word immediately preceding `start` in `buf` (after skipping ordinary
/// separator bytes: space, tab, `:`, `=`) case-insensitively matches a git-SHA context
/// marker.
fn preceded_by_git_sha_context(buf: &[u8], start: usize) -> bool {
    let mut i = start;
    while i > 0 && matches!(buf[i - 1], b' ' | b'\t' | b':' | b'=') {
        i -= 1;
    }
    let word_end = i;
    while i > 0 && buf[i - 1].is_ascii_alphabetic() {
        i -= 1;
    }
    let word = &buf[i..word_end];
    GIT_SHA_CONTEXT_WORDS
        .iter()
        .any(|w| word.eq_ignore_ascii_case(w))
}

/// Excludes a token from `Secret` classification when it is shaped like a git object
/// hash (7-64 lowercase hex characters -- covers abbreviated and full SHA-1/SHA-256
/// object IDs) **and** is immediately preceded by a git-hash context word (`commit`,
/// `sha`, `rev`, `hash`, ...). Deliberately narrow on both axes, not a blanket hex-shape
/// carve-out (2026-07-21, closing the T10 residual FP: `docs/decisions.md`,
/// benign-slice entropy finding = 1, a commit SHA):
///
/// - **Length + charset alone is not enough.** Many real secrets (session tokens, some
///   API keys) are themselves lowercase hex of a length that would collide with a SHA;
///   excluding every such token regardless of context would reopen a false-negative hole
///   this detector exists to avoid -- worse than the false positive being fixed.
/// - **Context alone is not enough either.** A token that merely follows the word
///   "commit" but isn't hex-shaped (e.g. a non-hex build identifier) is left to the
///   normal entropy/structured-identifier logic, unaffected by this exclusion.
/// - **Accepted residual:** a real secret that is coincidentally pure lowercase hex AND
///   immediately preceded by one of these context words (e.g. `commit token: <hex
///   secret>`) would be missed. Narrow and named, not fixed here -- the same posture
///   this file already takes with its other documented residuals.
fn looks_like_git_sha(buf: &[u8], start: usize, token: &[u8]) -> bool {
    let len = token.len();
    if !(7..=64).contains(&len) {
        return false;
    }
    if !token
        .iter()
        .all(|&b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return false;
    }
    preceded_by_git_sha_context(buf, start)
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
            if looks_like_git_sha(buf, start, token) {
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
        // census against real internal-tooling content (file paths and
        // snake_case/kebab-case identifiers), not the structured-run-ID shape an earlier,
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
    fn ignores_a_license_key_shaped_env_assignment() {
        // A second T10 residual (docs/decisions.md, 2026-07-21), surfaced only after the
        // commit-SHA fix: `=` wasn't a split delimiter, so this whole assignment scored as
        // one entropy blob even though both sides are word/numeric-shaped.
        let buf = b"LICENSE_KEY=ACME-2026-DEMO-KEY";
        assert!(EntropyDetector::default().detect(buf, &[]).is_empty());
    }

    #[test]
    fn still_flags_a_real_secret_after_an_equals_sign() {
        // Same `KEY=value` shape, but the value mixes letters and digits -- must not be
        // swallowed by the `=` split added for the LICENSE_KEY case above.
        let buf = b"API_KEY=zQ3v9Lm2Xp7RwT6uYbN8dKfJhC1sEoAi";
        assert_eq!(EntropyDetector::default().detect(buf, &[]).len(), 1);
    }

    #[test]
    fn ignores_a_commit_sha_with_git_context() {
        // The exact T10 benign-lookalike residual (docs/decisions.md, 2026-07-21):
        // a full 40-char lowercase-hex SHA-1 immediately preceded by "commit".
        let buf = b"commit 17b27d5c8f9e2a1b3d4c5e6f7a8b9c0d1e2f3a4b merged on 2026-07-18";
        assert!(EntropyDetector::default().detect(buf, &[]).is_empty());
    }

    #[test]
    fn still_flags_a_hex_looking_secret_with_no_git_context() {
        // Same shape (lowercase hex, SHA-length), no context word -- must NOT be
        // excluded by a blanket hex-shape carve-out.
        let buf = b"token=17b27d5c8f9e2a1b3d4c5e6f7a8b9c0d1e2f3a4b";
        assert_eq!(EntropyDetector::default().detect(buf, &[]).len(), 1);
    }

    #[test]
    fn still_flags_a_non_hex_token_after_the_word_commit() {
        // Context word present, but the token itself isn't hex-shaped -- the context
        // exclusion must not swallow ordinary high-entropy secrets that merely follow
        // the word "commit" in prose.
        let buf = b"commit token zQ3v9Lm2Xp7RwT6uYbN8dKfJhC1sEoAi applied";
        assert_eq!(EntropyDetector::default().detect(buf, &[]).len(), 1);
    }

    #[test]
    fn git_sha_context_words_are_case_insensitive_and_tolerate_a_colon() {
        let buf = b"Commit: 17b27d5c8f9e2a1b3d4c5e6f7a8b9c0d1e2f3a4b";
        assert!(EntropyDetector::default().detect(buf, &[]).is_empty());
    }

    #[test]
    fn never_panics_on_empty_or_invalid_utf8_input() {
        assert!(EntropyDetector::default().detect(b"", &[]).is_empty());
        let _ = EntropyDetector::default().detect(&[0xFF, 0xFE, 0x00, 0x80], &[]);
    }
}
