//! Typed-placeholder + HMAC keying (Task T04).
//!
//! Implements the formula `VaultStore`'s trait doc (`crate::traits`) names as authoritative:
//! "stable placeholder via salted HMAC over `(canonical(value), ty, ns)`". This module owns
//! that formula plus the supporting pieces `vg-vault`'s (Task T05) `VaultStore::intern` will
//! call into: canonicalisation, the keyed hash itself, per-type/per-namespace ordinal
//! assignment for the human-readable `display` string, Luhn/mod-97 checksum validators, and a
//! session-scoped cache so repeated keying of the same value doesn't recompute the HMAC.
//!
//! Deliberately standalone: this module does not depend on `vg-vault` (which doesn't exist
//! yet) and doesn't persist anything — `Keyer`'s cache is a front-cache for one process's
//! lifetime, not a replacement for the vault's own encrypted, durable storage.
//!
//! **Judgment call, recorded per the dispatch instructions (see `docs/decisions.md`):** item 4
//! of this task's spec asks for Luhn/mod-97 validators "so a placeholder's own display value
//! can be checked (or constructed) to remain checksum-valid". This module provides both as
//! pure, tested validators, but deliberately does *not* wire them into `display` construction
//! to synthesize a fake-but-checksum-valid card number or IBAN — doing so would produce a
//! synthetic value, which ADR-005 (`docs/decisions.md`, 2026-06-30, frozen before this task)
//! explicitly rejects for Phase 1 in favour of typed placeholders (`EMAIL_001`,
//! `ACCOUNT_ID_014` — see `README.md`). `display` stays `TYPE_TAG_NNN`; the validators are
//! exposed for callers that need to check or score checksum-shaped input (e.g. a future
//! detector-confidence booster or a masking-quality check), which is still useful independent
//! of how the placeholder is displayed.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::ids::{OrgId, RepoId, SessionId};
use crate::types::{EntityType, Namespace};

type HmacSha256 = Hmac<Sha256>;

/// Normalises `value` so trivial formatting differences (leading/trailing whitespace, internal
/// whitespace runs, purely cosmetic grouping separators for structural types, and — for entity
/// types where case carries no identity information — letter case) don't produce different
/// placeholders for what is semantically the same value.
///
/// **Codex cross-model doubt-pass finding (2026-07-17):** the original version only collapsed
/// whitespace and case-folded; it never stripped internal separators, so the *same* real IBAN
/// or sort code written in its compact form (`GB29NWBK60161331926819`) vs. its conventionally
/// spaced/hyphenated form (`GB29 NWBK 6016 1331 9268 19`, `12-34-56`) canonicalised to two
/// different strings and therefore got two different placeholders — a direct violation of this
/// task's own "same value -> same placeholder within namespace" acceptance criterion, since
/// `vg-detectors`' own IBAN/sort-code/phone detectors already recognise both forms as the same
/// real value (see `iban_sortcode.rs`/`phone.rs`'s own test names). Fixed by stripping
/// formatting separators for the specific types where they carry no identity information
/// (`Iban`, `SortCode`, `Phone`) before case-folding.
///
/// Case-folding is deliberately type-specific, not blanket: `Email`/`Hostname`/`InternalIp`/
/// `Iban`/`SortCode`/`Postcode`/`TraceId` are conventionally case-insensitive (an uppercase and
/// lowercase rendering of the same IP or IBAN identify the same real-world thing). Secret-shaped
/// types (`Password`, `PrivateKey`, `Secret`, `AccessToken`, `ApiKey`) and free-text/identifier
/// types (`Person`, `Address`, `EmployeeId`, `CustomerId`, `AccountId`, `Phone`, `Custom`) keep
/// their case: lower-casing a secret would treat two genuinely different values (differing only
/// in case) as the same one, which is a correctness bug for exactly the class of value this
/// tool most needs to keep distinct.
///
/// **Accepted residual, not fixed:** `Postcode`/`InternalIp`/`Hostname` keep their internal
/// separators (spaces, dots) as-is — those separators are structurally meaningful there (an IP's
/// dots separate octets; they are not cosmetic grouping the way IBAN/sort-code/phone separators
/// are), so stripping them would be a correctness bug in the other direction.
pub fn canonicalize(value: &str, ty: &EntityType) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let separators_stripped = strip_cosmetic_separators(&collapsed, ty);
    if is_case_insensitive(ty) {
        separators_stripped.to_lowercase()
    } else {
        separators_stripped
    }
}

/// Strips separators that are pure formatting/grouping noise for `ty` (not structurally
/// meaningful), so e.g. a compact and a spaced/hyphenated IBAN canonicalise identically. See
/// `canonicalize`'s doc comment for why this is scoped to exactly these three types.
fn strip_cosmetic_separators(value: &str, ty: &EntityType) -> String {
    match ty {
        EntityType::Iban | EntityType::SortCode => value
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect(),
        EntityType::Phone => value
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '.' | '(' | ')'))
            .collect(),
        _ => value.to_string(),
    }
}

fn is_case_insensitive(ty: &EntityType) -> bool {
    matches!(
        ty,
        EntityType::Email
            | EntityType::Hostname
            | EntityType::InternalIp
            | EntityType::Iban
            | EntityType::SortCode
            | EntityType::Postcode
            | EntityType::TraceId
    )
}

/// The stable output of the keying HMAC: 32 bytes (SHA-256 width), deterministic for a given
/// `(salt, canonical(value), ty, ns)`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceholderKey(pub [u8; 32]);

impl PlaceholderKey {
    /// Lowercase hex encoding, useful as a stable string form (e.g. a vault lookup key).
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl fmt::Debug for PlaceholderKey {
    /// Deliberately redacting, matching `Secret`'s own `Debug` impl (`crate::traits`): this key
    /// is architecturally a vault lookup key (see `to_hex`'s own doc comment), and the frozen
    /// contract cares about audit/log surfaces seeing opaque references, not raw cryptographic
    /// material. **Codex cross-model doubt-pass finding (2026-07-17):** the original impl printed
    /// the full HMAC hex via `to_hex()`, which meant any incidental `{:?}` formatting (a test
    /// failure message, a log line, an error context) would leak the actual vault key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PlaceholderKey")
            .field(&"<redacted>")
            .finish()
    }
}

/// Computes the salted HMAC-SHA256 over `(canonicalize(value, ty), ty, ns)`.
///
/// Deterministic: the same `(salt, value, ty, ns)` always yields the same key. Namespace- and
/// type-sensitive by construction: the message fed to the HMAC embeds both the entity-type's
/// *keying* tag ([`type_tag_for_keying`] — see its doc comment for why this is a separate
/// function from the display tag) and the namespace tag alongside the canonical value, separated
/// by a byte (`0x1F`, the ASCII Unit Separator) that cannot appear in any of the three
/// tag/value segments as ordinary text, so concatenation can't produce the same message for two
/// different inputs (e.g. value="ab", tag="c" vs. value="a", tag="bc" would collide under naive
/// concatenation without a separator).
///
/// `salt` is caller-supplied rather than a hardcoded constant: this crate doesn't own
/// persistent secret-key storage (`vg-vault`, Task T05, wraps the real key via the OS
/// keychain per `interface-contracts.md` §5) and a compiled-in salt would make "salted" a
/// no-op — any two installs would key identically.
pub fn placeholder_key(
    salt: &[u8],
    value: &str,
    ty: &EntityType,
    ns: &Namespace,
) -> PlaceholderKey {
    let canonical = canonicalize(value, ty);
    let mut mac =
        HmacSha256::new_from_slice(salt).expect("HMAC-SHA256 accepts a key of any length");
    mac.update(canonical.as_bytes());
    mac.update(&[0x1F]);
    mac.update(type_tag_for_keying(ty).as_bytes());
    mac.update(&[0x1F]);
    mac.update(namespace_tag(ns).as_bytes());

    let digest = mac.finalize().into_bytes();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    PlaceholderKey(key)
}

/// The entity-type component of the HMAC message in [`placeholder_key`]. **Deliberately a
/// separate function from [`type_tag_for_display`]**, fixed 2026-07-17 per a Codex cross-model
/// doubt-pass finding: the original code used one lossy, cosmetically-formatted tag for both the
/// cryptographic keying input and the human-readable display prefix. Two real problems followed
/// from that conflation:
///
/// 1. **A collision bug for `Custom`.** The display formatter upper-snake-cases and collapses
///    non-alphanumeric runs, so `Custom("foo-bar")`, `Custom("foo_bar")`, and `Custom("foo bar")`
///    — three different policy-dictionary classes — all produced the identical tag
///    `CUSTOM_FOO_BAR`, meaning the same value under three different `Custom` classes silently
///    keyed identically. This function instead embeds the raw, unmodified dictionary name
///    (`format!("CUSTOM:{name}")`), so distinct names key distinctly regardless of formatting.
/// 2. **A future stability hazard for the fixed variants.** If the *display* prefix for, say,
///    `AccountId` were ever renamed for cosmetic reasons (e.g. a UX pass renaming `ACCOUNT_ID` to
///    `ACCT`), reusing that same string as the keying tag would silently change every already-
///    persisted placeholder's HMAC key. This function's fixed-variant tags are a separate,
///    stability-committed contract, even though today they happen to share text with
///    [`type_tag_for_display`]'s output for the non-`Custom` variants.
fn type_tag_for_keying(ty: &EntityType) -> String {
    match ty {
        EntityType::Custom(name) => format!("CUSTOM:{name}"),
        other => type_tag_for_display(other),
    }
}

/// The `TYPE_TAG` half of a placeholder's `display` string (`EMAIL_001`, `ACCOUNT_ID_014` —
/// `README.md`'s own example). Cosmetic only — safe to reformat freely, unlike
/// [`type_tag_for_keying`]. Written as an explicit match (not a generic
/// CamelCase-to-SCREAMING_SNAKE_CASE converter) so the exact display prefixes are visible in one
/// place rather than derived indirectly.
fn type_tag_for_display(ty: &EntityType) -> String {
    match ty {
        EntityType::Person => "PERSON".to_string(),
        EntityType::Email => "EMAIL".to_string(),
        EntityType::Phone => "PHONE".to_string(),
        EntityType::Address => "ADDRESS".to_string(),
        EntityType::Postcode => "POSTCODE".to_string(),
        EntityType::EmployeeId => "EMPLOYEE_ID".to_string(),
        EntityType::CustomerId => "CUSTOMER_ID".to_string(),
        EntityType::AccountId => "ACCOUNT_ID".to_string(),
        EntityType::Iban => "IBAN".to_string(),
        EntityType::SortCode => "SORT_CODE".to_string(),
        EntityType::InternalIp => "INTERNAL_IP".to_string(),
        EntityType::Hostname => "HOSTNAME".to_string(),
        EntityType::ApiKey => "API_KEY".to_string(),
        EntityType::TraceId => "TRACE_ID".to_string(),
        EntityType::Password => "PASSWORD".to_string(),
        EntityType::PrivateKey => "PRIVATE_KEY".to_string(),
        EntityType::Secret => "SECRET".to_string(),
        EntityType::AccessToken => "ACCESS_TOKEN".to_string(),
        EntityType::Custom(name) => format!("CUSTOM_{}", screaming_snake(name)),
    }
}

/// Upper-cases `s` and collapses any run of non-alphanumeric characters to a single `_`,
/// trimming leading/trailing underscores — turns a policy-dictionary class name like
/// `"internal-project-codename"` into `INTERNAL_PROJECT_CODENAME`.
fn screaming_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

fn namespace_tag(ns: &Namespace) -> String {
    match ns {
        Namespace::Session(SessionId(uuid)) => format!("SESSION:{uuid}"),
        Namespace::Repo(RepoId(id)) => format!("REPO:{id}"),
        Namespace::Org(OrgId(id)) => format!("ORG:{id}"),
    }
}

/// Everything `Keyer::key_for` returns: the stable HMAC key, the per-type/per-namespace
/// ordinal, and the ready-to-use `display` string (`Placeholder.display`'s eventual value once
/// `vg-vault` wraps this in a `MappingRef`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyed {
    pub key: PlaceholderKey,
    pub ordinal: u64,
    pub display: String,
}

#[derive(Default)]
struct KeyerState {
    cache: HashMap<(String, EntityType, Namespace), Keyed>,
    ordinals: HashMap<(Namespace, EntityType), u64>,
}

/// Session-scoped keyer: wraps [`placeholder_key`] with the ordinal assignment (item 3) and
/// in-memory cache (item 5) the task spec asks for.
///
/// `Send + Sync` (state behind a `Mutex`, not a `RefCell`) since `vg-vault`'s eventual
/// `VaultStore::intern(&self, ...)` — a `Send + Sync` trait per `interface-contracts.md` — will
/// hold one of these behind a shared reference, not an exclusive one.
pub struct Keyer {
    salt: Vec<u8>,
    state: Mutex<KeyerState>,
}

impl Keyer {
    pub fn new(salt: impl Into<Vec<u8>>) -> Self {
        Self {
            salt: salt.into(),
            state: Mutex::new(KeyerState::default()),
        }
    }

    /// Returns the cached `Keyed` for `(value, ty, ns)` if this `Keyer` has already seen it
    /// (after canonicalisation), or computes, assigns the next ordinal for `(ns, ty)`, caches,
    /// and returns a new one. Same key in, same `Keyed` out — including `display` — every time.
    pub fn key_for(&self, value: &str, ty: EntityType, ns: &Namespace) -> Keyed {
        let canonical = canonicalize(value, &ty);
        let cache_key = (canonical.clone(), ty.clone(), ns.clone());

        let mut state = self.state.lock().expect("Keyer mutex poisoned");
        if let Some(existing) = state.cache.get(&cache_key) {
            return existing.clone();
        }

        let key = placeholder_key(&self.salt, &canonical, &ty, ns);
        let ordinal_key = (ns.clone(), ty.clone());
        let ordinal = {
            let counter = state.ordinals.entry(ordinal_key).or_insert(0);
            *counter += 1;
            *counter
        };
        let display = format!("{}_{ordinal:03}", type_tag_for_display(&ty));

        let keyed = Keyed {
            key,
            ordinal,
            display,
        };
        state.cache.insert(cache_key, keyed.clone());
        keyed
    }
}

/// Luhn (mod-10) checksum, for card-number-shaped digit strings. Ignores interior whitespace
/// and hyphens (common human-entered grouping); any other non-digit character makes the value
/// invalid outright rather than being silently skipped, since silently dropping unexpected
/// characters could turn a malformed value into a coincidentally-valid checksum.
pub fn luhn_is_valid(value: &str) -> bool {
    let mut digits = Vec::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_whitespace() || ch == '-' {
            continue;
        }
        match ch.to_digit(10) {
            Some(d) => digits.push(d),
            None => return false,
        }
    }
    if digits.len() < 2 {
        return false;
    }

    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// ISO 7064 mod-97-10 checksum, for IBAN-shaped strings (ISO 13616): move the first four
/// characters to the end, map letters to two-digit numbers (`A`=10 .. `Z`=35), and check the
/// resulting numeric string is congruent to 1 mod 97. Computed as a running remainder (never
/// materialising the full ~30+ digit number) since IBANs comfortably overflow a `u64` once
/// expanded to decimal.
///
/// **This checks the mod-97 checksum only** — not the country-specific total length, the
/// country-code alphabetic prefix, or the BBAN structure. A string can pass this check while
/// still not being a real IBAN in any country's format. Named `_mod97_` rather than
/// `is_valid_iban` deliberately: treat this as a checksum validator, not a full IBAN format
/// validator (flagged in a Codex cross-model doubt pass, 2026-07-17 — see `docs/decisions.md`).
pub fn iban_mod97_is_valid(value: &str) -> bool {
    let cleaned: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.len() < 5 || !cleaned.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    let cleaned = cleaned.to_ascii_uppercase();
    let (head, tail) = cleaned.split_at(4);

    let mut remainder: u64 = 0;
    for ch in tail.chars().chain(head.chars()) {
        let digit_value = if ch.is_ascii_digit() {
            ch.to_digit(10).expect("ascii digit") as u64
        } else {
            (ch as u64 - 'A' as u64) + 10
        };
        let place = if digit_value >= 10 { 100 } else { 10 };
        remainder = (remainder * place + digit_value) % 97;
    }
    remainder == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn ns(id: &str) -> Namespace {
        Namespace::Repo(RepoId(id.to_string()))
    }

    #[test]
    fn canonicalize_trims_and_collapses_whitespace() {
        assert_eq!(
            canonicalize("  Jane   Doe  ", &EntityType::Person),
            "Jane Doe"
        );
    }

    #[test]
    fn canonicalize_folds_case_for_case_insensitive_types() {
        assert_eq!(
            canonicalize("Jane.Doe@Example.COM", &EntityType::Email),
            "jane.doe@example.com"
        );
    }

    #[test]
    fn canonicalize_preserves_case_for_secrets() {
        assert_eq!(
            canonicalize("Sup3rSecret!", &EntityType::Password),
            "Sup3rSecret!"
        );
    }

    #[test]
    fn placeholder_key_is_deterministic() {
        let ns = ns("acme/widgets");
        let a = placeholder_key(b"salt", "jane@example.com", &EntityType::Email, &ns);
        let b = placeholder_key(b"salt", "jane@example.com", &EntityType::Email, &ns);
        assert_eq!(a, b);
    }

    #[test]
    fn placeholder_key_is_namespace_sensitive() {
        let a = placeholder_key(
            b"salt",
            "jane@example.com",
            &EntityType::Email,
            &ns("repo-a"),
        );
        let b = placeholder_key(
            b"salt",
            "jane@example.com",
            &EntityType::Email,
            &ns("repo-b"),
        );
        assert_ne!(
            a, b,
            "the same value under different namespaces must key differently"
        );
    }

    #[test]
    fn placeholder_key_is_type_sensitive() {
        let n = ns("acme/widgets");
        let a = placeholder_key(b"salt", "12345", &EntityType::AccountId, &n);
        let b = placeholder_key(b"salt", "12345", &EntityType::CustomerId, &n);
        assert_ne!(
            a, b,
            "the same value under different entity types must key differently"
        );
    }

    #[test]
    fn placeholder_key_is_salt_sensitive() {
        let n = ns("acme/widgets");
        let a = placeholder_key(b"salt-one", "jane@example.com", &EntityType::Email, &n);
        let b = placeholder_key(b"salt-two", "jane@example.com", &EntityType::Email, &n);
        assert_ne!(a, b, "a different salt must produce a different key");
    }

    #[test]
    fn placeholder_key_has_no_naive_concatenation_collision() {
        // "ab" + "c" vs. "a" + "bc" would collide under plain string concatenation without a
        // separator between the canonical value and the type tag.
        let n = ns("repo");
        let a = placeholder_key(b"salt", "ab", &EntityType::Custom("c".to_string()), &n);
        let b = placeholder_key(b"salt", "a", &EntityType::Custom("bc".to_string()), &n);
        assert_ne!(a, b);
    }

    #[test]
    fn keyer_assigns_sequential_ordinals_per_type_within_a_namespace() {
        let keyer = Keyer::new(b"salt".to_vec());
        let n = ns("acme/widgets");
        let first = keyer.key_for("alice@example.com", EntityType::Email, &n);
        let second = keyer.key_for("bob@example.com", EntityType::Email, &n);
        assert_eq!(first.display, "EMAIL_001");
        assert_eq!(second.display, "EMAIL_002");
    }

    #[test]
    fn keyer_returns_the_same_ordinal_and_display_for_a_repeated_value() {
        let keyer = Keyer::new(b"salt".to_vec());
        let n = ns("acme/widgets");
        let first = keyer.key_for("alice@example.com", EntityType::Email, &n);
        let repeat = keyer.key_for("Alice@Example.com", EntityType::Email, &n); // different case
        assert_eq!(
            first, repeat,
            "case-insensitive canonicalisation should hit the cache"
        );
    }

    #[test]
    fn keyer_scopes_ordinals_independently_per_namespace() {
        let keyer = Keyer::new(b"salt".to_vec());
        let repo_a = ns("repo-a");
        let repo_b = ns("repo-b");
        let a = keyer.key_for("alice@example.com", EntityType::Email, &repo_a);
        let b = keyer.key_for("bob@example.com", EntityType::Email, &repo_b);
        assert_eq!(a.display, "EMAIL_001");
        assert_eq!(
            b.display, "EMAIL_001",
            "a fresh namespace starts its own ordinal sequence"
        );
    }

    #[test]
    fn keyer_scopes_ordinals_independently_per_type() {
        let keyer = Keyer::new(b"salt".to_vec());
        let n = ns("acme/widgets");
        let email = keyer.key_for("alice@example.com", EntityType::Email, &n);
        let account = keyer.key_for("acct-123", EntityType::AccountId, &n);
        assert_eq!(email.display, "EMAIL_001");
        assert_eq!(account.display, "ACCOUNT_ID_001");
    }

    #[test]
    fn keyer_display_uses_custom_dictionary_tag() {
        let keyer = Keyer::new(b"salt".to_vec());
        let n = ns("acme/widgets");
        let keyed = keyer.key_for(
            "Project Nightingale",
            EntityType::Custom("internal-project-codename".to_string()),
            &n,
        );
        assert_eq!(keyed.display, "CUSTOM_INTERNAL_PROJECT_CODENAME_001");
    }

    #[test]
    fn distinct_custom_classes_key_differently_despite_similar_display_formatting() {
        // Codex cross-model doubt-pass finding (2026-07-17): these three Custom names all
        // display-format to the same "CUSTOM_FOO_BAR" tag, but must key differently -- three
        // genuinely different policy-dictionary classes, not one.
        let n = ns("acme/widgets");
        let a = placeholder_key(
            b"salt",
            "same-value",
            &EntityType::Custom("foo-bar".to_string()),
            &n,
        );
        let b = placeholder_key(
            b"salt",
            "same-value",
            &EntityType::Custom("foo_bar".to_string()),
            &n,
        );
        let c = placeholder_key(
            b"salt",
            "same-value",
            &EntityType::Custom("foo bar".to_string()),
            &n,
        );
        assert_ne!(a, b, "foo-bar and foo_bar must key differently");
        assert_ne!(a, c, "foo-bar and \"foo bar\" must key differently");
        assert_ne!(b, c, "foo_bar and \"foo bar\" must key differently");
    }

    #[test]
    fn compact_and_spaced_iban_key_identically() {
        // Codex cross-model doubt-pass finding (2026-07-17): vg-detectors' own IBAN detector
        // recognises both forms as the same real value; keying must agree.
        let n = ns("acme/widgets");
        let compact = placeholder_key(b"salt", "GB29NWBK60161331926819", &EntityType::Iban, &n);
        let spaced = placeholder_key(
            b"salt",
            "GB29 NWBK 6016 1331 9268 19",
            &EntityType::Iban,
            &n,
        );
        assert_eq!(compact, spaced);
    }

    #[test]
    fn hyphenated_and_spaced_sort_code_key_identically() {
        let n = ns("acme/widgets");
        let hyphenated = placeholder_key(b"salt", "12-34-56", &EntityType::SortCode, &n);
        let spaced = placeholder_key(b"salt", "12 34 56", &EntityType::SortCode, &n);
        assert_eq!(hyphenated, spaced);
    }

    #[test]
    fn differently_formatted_phone_numbers_key_identically() {
        let n = ns("acme/widgets");
        let a = placeholder_key(b"salt", "+1-415-555-2671", &EntityType::Phone, &n);
        let b = placeholder_key(b"salt", "+14155552671", &EntityType::Phone, &n);
        let c = placeholder_key(b"salt", "+1 415 555 2671", &EntityType::Phone, &n);
        assert_eq!(a, b, "hyphenated and bare must key identically");
        assert_eq!(a, c, "hyphenated and spaced must key identically");
    }

    #[test]
    fn placeholder_key_debug_never_prints_the_real_hex() {
        // Codex cross-model doubt-pass finding (2026-07-17): the original Debug impl printed
        // the full HMAC hex, leaking the vault lookup key through incidental {:?} formatting.
        let n = ns("acme/widgets");
        let key = placeholder_key(b"salt", "jane@example.com", &EntityType::Email, &n);
        let debug_output = format!("{key:?}");
        assert!(!debug_output.contains(&key.to_hex()));
        assert!(debug_output.contains("redacted"));
    }

    #[test]
    fn keyer_ordinal_display_does_not_truncate_past_three_digits() {
        let keyer = Keyer::new(b"salt".to_vec());
        let n = ns("acme/widgets");
        for i in 0..1000 {
            keyer.key_for(&format!("user{i}@example.com"), EntityType::Email, &n);
        }
        let last = keyer.key_for("user999@example.com", EntityType::Email, &n);
        assert_eq!(last.ordinal, 1000);
        assert_eq!(last.display, "EMAIL_1000");
    }

    #[test]
    fn placeholder_key_to_hex_round_trips_length() {
        let n = ns("acme/widgets");
        let key = placeholder_key(b"salt", "jane@example.com", &EntityType::Email, &n);
        assert_eq!(key.to_hex().len(), 64); // 32 bytes -> 64 hex chars
    }

    #[test]
    fn session_namespace_is_uuid_sensitive() {
        let a = Namespace::Session(SessionId(Uuid::nil()));
        let b = Namespace::Session(SessionId(Uuid::new_v4()));
        let key_a = placeholder_key(b"salt", "value", &EntityType::TraceId, &a);
        let key_b = placeholder_key(b"salt", "value", &EntityType::TraceId, &b);
        assert_ne!(key_a, key_b);
    }

    // -- Luhn --

    #[test]
    fn luhn_accepts_a_known_valid_number() {
        assert!(luhn_is_valid("79927398713"));
    }

    #[test]
    fn luhn_rejects_a_single_flipped_digit() {
        assert!(!luhn_is_valid("79927398714"));
    }

    #[test]
    fn luhn_accepts_a_spaced_test_card_number() {
        assert!(luhn_is_valid("4111 1111 1111 1111"));
    }

    #[test]
    fn luhn_rejects_non_digit_junk() {
        assert!(!luhn_is_valid("4111-1111-ABCD-1111"));
    }

    #[test]
    fn luhn_rejects_too_short_input() {
        assert!(!luhn_is_valid("5"));
    }

    // -- mod-97 --

    #[test]
    fn iban_mod97_accepts_the_canonical_example_iban() {
        assert!(iban_mod97_is_valid("GB29NWBK60161331926819"));
    }

    #[test]
    fn iban_mod97_accepts_a_spaced_iban() {
        assert!(iban_mod97_is_valid("GB29 NWBK 6016 1331 9268 19"));
    }

    #[test]
    fn iban_mod97_rejects_a_single_flipped_digit() {
        assert!(!iban_mod97_is_valid("GB29NWBK60161331926818"));
    }

    #[test]
    fn iban_mod97_rejects_too_short_input() {
        assert!(!iban_mod97_is_valid("GB29"));
    }

    #[test]
    fn iban_mod97_rejects_non_alphanumeric_input() {
        assert!(!iban_mod97_is_valid("GB29-NWBK-60161331926819"));
    }
}
