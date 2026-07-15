//! Contract-conformance helpers.
//!
//! Wave B squads (`vg-detectors`, `vg-parsers`, `vg-vault`, `vg-policy`, `vg-audit`) call
//! these from their own crate's test suite to check a trait impl against the invariants
//! named in `docs/architecture/interface-contracts.md`, without `vg-core` needing to
//! know about any concrete impl. Each function asserts internally, so it reads naturally
//! inside a `#[test]`:
//!
//! ```ignore
//! #[test]
//! fn my_detector_is_conformant() {
//!     let d = MyDetector::default();
//!     vg_core::conformance::assert_detector_contract(&d, b"...", &[]);
//! }
//! ```
//!
//! See `crates/vg-core/tests/conformance_stubs.rs` for a full worked example against
//! mock implementations of every trait.

use crate::audit::AuditEvent;
use crate::traits::{AuditSink, Detector, Parser, Secret, VaultStore};
use crate::types::{EntityType, MaskedPack, Namespace, Span};

/// `Detector::detect` must be deterministic and only return entity types it declares,
/// with confidence in `0.0..=1.0`.
pub fn assert_detector_contract<D: Detector>(detector: &D, buf: &[u8], spans: &[Span]) {
    let first = detector.detect(buf, spans);
    let second = detector.detect(buf, spans);
    assert_eq!(
        first, second,
        "Detector::detect must be deterministic for the same input"
    );

    let declared = detector.entity_types();
    for finding in &first {
        assert!(
            declared.contains(&finding.entity_type),
            "Finding {finding:?} has an entity_type not declared by Detector::entity_types()"
        );
        assert!(
            (0.0..=1.0).contains(&finding.confidence),
            "Finding confidence {} out of 0.0..=1.0",
            finding.confidence
        );
    }
}

/// `Parser::parse` must never panic, even on malformed input.
pub fn assert_parser_never_panics<P: Parser>(parser: &P, buf: &[u8]) {
    let _ = parser.parse(buf);
}

/// A value interned into a `VaultStore` must resolve back to the same raw value, and
/// interning the same `(value, ty, ns)` twice must yield the same placeholder (the
/// stable-placeholder invariant).
pub fn assert_vault_roundtrip<V: VaultStore>(
    vault: &V,
    value: &str,
    ty: EntityType,
    ns: &Namespace,
) {
    let first = vault
        .intern(&Secret::new(value.to_string()), ty.clone(), ns)
        .expect("intern should succeed");
    let second = vault
        .intern(&Secret::new(value.to_string()), ty, ns)
        .expect("intern should succeed");
    assert_eq!(
        first.mapping_ref, second.mapping_ref,
        "interning the same (value, type, namespace) twice must yield the same placeholder"
    );

    let resolved = vault.resolve(&first, ns).expect("resolve should succeed");
    assert_eq!(
        resolved.expose_secret(),
        value,
        "resolve must return the original raw value"
    );
}

/// A written `AuditEvent` must be returned unchanged by `get`.
pub fn assert_audit_sink_roundtrip<A: AuditSink>(sink: &A, event: AuditEvent) {
    let id = sink.write(event.clone()).expect("write should succeed");
    let fetched = sink
        .get(id)
        .expect("get must return the event just written");
    assert_eq!(
        fetched, event,
        "AuditSink::get must return exactly what was written"
    );
}

/// No `AuditEvent` variant may embed a known raw value — audit events carry only refs,
/// counts, and versions.
pub fn assert_audit_event_excludes_raw_values(event: &AuditEvent, raw_values: &[&str]) {
    let rendered = format!("{event:?}");
    for raw in raw_values {
        assert!(
            !rendered.contains(raw),
            "AuditEvent {event:?} appears to embed a raw value ({raw:?})"
        );
    }
}

/// `MaskedPack.text` must never contain a raw detected value.
pub fn assert_masked_pack_excludes_raw_values(pack: &MaskedPack, raw_values: &[&str]) {
    for raw in raw_values {
        assert!(
            !pack.text.contains(raw),
            "MaskedPack.text appears to embed a raw value ({raw:?}) — must be placeholders only"
        );
    }
}
