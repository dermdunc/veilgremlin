//! Cross-crate integration: real parser `Span` output → the T03 detectors.
//!
//! Required by `docs/architecture/interface-contracts.md` (cross-crate integration note,
//! 2026-07-16): at least one integration test must feed this crate's real `Span` output
//! into `vg_detectors::all_detectors()` on a realistic fixture. This file does that for
//! several formats, and — critically — records the observed behaviour of the detectors'
//! `spans` parameter: every detector's `detect(&self, buf, _spans)` currently ignores its
//! `spans` argument (the `_spans` no-op confirmed 2026-07-16). We assert that no-op here
//! so it cannot silently change or be silently relied upon before Task T07 wires spans
//! into the pipeline. See `docs/decisions.md` (2026-07-17, T08) for the classification of
//! whether this is a real gap or a stage-appropriate one.

use vg_core::{EntityType, Finding, Parser, Span};
use vg_detectors::all_detectors;
use vg_parsers::{csv::CsvParser, env::EnvParser, json::JsonParser, yaml::YamlParser};

/// Runs every detector over `buf` with the given `spans`, flattening all findings.
fn detect_all(buf: &[u8], spans: &[Span]) -> Vec<Finding> {
    all_detectors()
        .iter()
        .flat_map(|d| d.detect(buf, spans))
        .collect()
}

fn has_entity(findings: &[Finding], ty: &EntityType) -> bool {
    findings.iter().any(|f| f.entity_type == *ty)
}

#[test]
fn json_parser_spans_feed_detectors_and_find_email_and_ip() {
    // A realistic service-config fixture: an email and an internal IP live in string
    // values the JSON parser tags, embedded in structure the detectors would otherwise
    // have to wade through.
    let buf = br#"{
        "service": "billing",
        "owner": "jane.doe@example.com",
        "upstream": "10.1.2.3",
        "retries": 5
    }"#;

    let parsed = JsonParser.parse(buf);
    assert!(!parsed.spans.is_empty(), "parser produced no spans");

    // Feed the REAL parser spans into the detectors (the integration requirement).
    let findings = detect_all(buf, &parsed.spans);
    assert!(
        has_entity(&findings, &EntityType::Email),
        "expected an Email finding, got {findings:?}"
    );
    assert!(
        has_entity(&findings, &EntityType::InternalIp),
        "expected an InternalIp finding, got {findings:?}"
    );

    // Every finding's span must be a valid byte range into buf (the Detector contract),
    // regardless of what parser spans we supplied.
    for f in &findings {
        assert!(f.span.start <= f.span.end && f.span.end <= buf.len());
    }
}

#[test]
fn detectors_currently_ignore_the_spans_parameter() {
    // The heart of the recorded assertion: feeding real parser spans vs. an empty slice
    // vs. deliberately WRONG spans must all yield identical findings, because every
    // detector's signature is `detect(&self, buf, _spans)` — `spans` is a no-op today.
    //
    // If a future change (T07) makes detectors span-aware, THIS test breaks first and
    // loudly, forcing the decision to be revisited rather than the no-op being assumed
    // still true. That is the point of pinning it.
    let buf = br#"{"owner": "jane.doe@example.com", "host": "10.1.2.3"}"#;

    let parsed = JsonParser.parse(buf);
    let with_real_spans = detect_all(buf, &parsed.spans);
    let with_no_spans = detect_all(buf, &[]);
    let with_bogus_spans = detect_all(
        buf,
        &[Span {
            start: 0,
            end: 1,
            node_kind: None,
        }],
    );

    assert_eq!(
        with_real_spans, with_no_spans,
        "detectors' output changed with parser spans — the `_spans` no-op no longer holds"
    );
    assert_eq!(
        with_real_spans, with_bogus_spans,
        "detectors' output changed with bogus spans — the `_spans` no-op no longer holds"
    );
}

#[test]
fn env_and_yaml_and_csv_fixtures_also_feed_detectors() {
    // Breadth: three more formats, each a realistic fixture, all feeding real spans into
    // the detectors — so the integration surface isn't a single-format happy path.
    let env =
        b"# service creds\nDATABASE_URL=postgres://u@db.internal\nADMIN_EMAIL=ops@example.com\n";
    let env_spans = EnvParser.parse(env).spans;
    let env_findings = detect_all(env, &env_spans);
    assert!(
        has_entity(&env_findings, &EntityType::Email),
        "env: {env_findings:?}"
    );

    let yaml = b"owner:\n  email: jane@example.com\n  ip: 192.168.0.1\n";
    let yaml_spans = YamlParser.parse(yaml).spans;
    let yaml_findings = detect_all(yaml, &yaml_spans);
    assert!(
        has_entity(&yaml_findings, &EntityType::Email),
        "yaml: {yaml_findings:?}"
    );
    assert!(
        has_entity(&yaml_findings, &EntityType::InternalIp),
        "yaml: {yaml_findings:?}"
    );

    let csv = b"name,email,ip\njane,jane@example.com,10.0.0.9\n";
    let csv_spans = CsvParser.parse(csv).spans;
    let csv_findings = detect_all(csv, &csv_spans);
    assert!(
        has_entity(&csv_findings, &EntityType::Email),
        "csv: {csv_findings:?}"
    );
}
