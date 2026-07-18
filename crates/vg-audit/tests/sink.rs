//! `JsonlAuditSink` against the frozen `AuditSink` contract: conformance roundtrips for
//! every `AuditEvent` variant, durability across reopen, append-only recovery behaviour,
//! schema versioning, and the redaction-safety property test the T05b acceptance
//! criteria name ("no raw value ever serialised").

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Arc;

use uuid::Uuid;
use vg_audit::{JsonlAuditSink, OpenError};
use vg_core::{
    ActorId, ArtefactKind, AuditEvent, AuditSink, Destination, EntityCounts, EntityType,
    HandlingClass, MappingRef,
};

fn sample_counts() -> EntityCounts {
    let mut m = BTreeMap::new();
    m.insert(EntityType::Email, 2);
    m.insert(EntityType::Iban, 1);
    m.insert(EntityType::Custom("project-codename".to_string()), 3);
    EntityCounts(m)
}

/// One event per contract-v1 variant, exercising the data-bearing corners (custom entity
/// types, source-code artefacts, hard-deny destinations).
fn one_of_each_variant() -> Vec<AuditEvent> {
    vec![
        AuditEvent::Scan {
            counts: sample_counts(),
            detector_version: "detectors-v1".to_string(),
            latency_us: 1832,
        },
        AuditEvent::PolicyDecision {
            artefact: ArtefactKind::SourceCode("rust".to_string()),
            class: HandlingClass::Mask,
            policy_version: "policy-v1".to_string(),
        },
        AuditEvent::MappingCreated {
            mapping_ref: MappingRef(Uuid::new_v4()),
            entity_type: EntityType::Custom("internal-hostname".to_string()),
        },
        AuditEvent::Block {
            artefact: ArtefactKind::EnvFile,
            reason: "env files are block-class in default policy".to_string(),
        },
        AuditEvent::DemaskRequest {
            dest: Destination::LocalPatch,
            actor: ActorId("derm".to_string()),
        },
        AuditEvent::DemaskDecision {
            dest: Destination::RemoteModelPrompt,
            actor: ActorId("derm".to_string()),
            allowed: false,
            policy_version: "policy-v1".to_string(),
        },
    ]
}

/// Std-only tempdir (removed on drop) — this crate deliberately takes no `tempfile`
/// dev-dependency; see the dependency note in its `Cargo.toml`.
struct TempDir(std::path::PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_log() -> (TempDir, std::path::PathBuf) {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "vg-audit-test-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    let path = dir.join("audit.jsonl");
    (TempDir(dir), path)
}

#[test]
fn every_variant_satisfies_the_sink_roundtrip_conformance() {
    let (_dir, path) = temp_log();
    let sink = JsonlAuditSink::open(&path).unwrap();
    for event in one_of_each_variant() {
        vg_core::conformance::assert_audit_sink_roundtrip(&sink, event);
    }
}

#[test]
fn events_survive_a_reopen_byte_for_byte() {
    let (_dir, path) = temp_log();
    let events = one_of_each_variant();

    let ids: Vec<_> = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        events
            .iter()
            .map(|e| sink.write(e.clone()).unwrap())
            .collect()
    };

    let reopened = JsonlAuditSink::open(&path).unwrap();
    assert_eq!(reopened.len(), events.len());
    assert_eq!(reopened.skipped_lines(), 0);
    for (id, event) in ids.iter().zip(&events) {
        assert_eq!(
            reopened.get(*id).as_ref(),
            Some(event),
            "event must be identical after reopen"
        );
    }
}

#[test]
fn reopen_appends_rather_than_rewrites() {
    let (_dir, path) = temp_log();
    let first_id = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        sink.write(one_of_each_variant().remove(0)).unwrap()
    };
    let second_id = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        sink.write(one_of_each_variant().remove(3)).unwrap()
    };

    let contents = std::fs::read_to_string(&path).unwrap();
    assert_eq!(contents.lines().count(), 2, "one JSONL line per event");

    let sink = JsonlAuditSink::open(&path).unwrap();
    assert!(sink.get(first_id).is_some());
    assert!(sink.get(second_id).is_some());
}

#[test]
fn a_torn_final_line_is_skipped_healed_and_does_not_lose_earlier_events() {
    let (_dir, path) = temp_log();
    let id = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        sink.write(one_of_each_variant().remove(0)).unwrap()
    };

    // Simulate a crash mid-append: a partial record with no trailing newline.
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(br#"{"schema_version":1,"id":"trunc"#)
        .unwrap();
    drop(file);

    let sink = JsonlAuditSink::open(&path).unwrap();
    assert_eq!(
        sink.skipped_lines(),
        1,
        "the torn line is counted, not fatal"
    );
    assert!(sink.get(id).is_some(), "intact earlier events survive");

    // The heal-newline means a fresh write starts on its own line and everything
    // reopens cleanly again.
    let new_id = sink.write(one_of_each_variant().remove(5)).unwrap();
    let reopened = JsonlAuditSink::open(&path).unwrap();
    assert!(reopened.get(id).is_some());
    assert!(reopened.get(new_id).is_some());
    assert_eq!(reopened.len(), 2);
}

#[test]
fn a_valid_record_without_a_trailing_newline_is_not_indexed_then_lost() {
    // Codex critique (2026-07-17): a final line that is a *complete, parseable* record but
    // lacks a trailing newline is a torn write (the '\n'+fsync never landed). It must be
    // discarded consistently — not indexed (so `get()`/`len()` see it) and then truncated
    // off disk (so a reopen loses it). This guards that index and disk agree.
    let (_dir, path) = temp_log();
    // One good, newline-terminated record, then a second valid record with NO trailing \n.
    let good_line = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        let _ = sink.write(one_of_each_variant().remove(0)).unwrap();
        // Build a second valid record line by writing+reading it, then re-append without \n.
        let second = JsonlAuditSink::open(&path).unwrap();
        let _ = second.write(one_of_each_variant().remove(3)).unwrap();
        std::fs::read_to_string(&path).unwrap()
    };
    // Rewrite the file: first line kept with its newline, second line stripped of its newline.
    let mut lines: Vec<&str> = good_line.lines().collect();
    let torn = lines.pop().unwrap().to_string();
    let rebuilt = format!("{}\n{}", lines.join("\n"), torn); // torn tail has no trailing \n
    std::fs::write(&path, rebuilt).unwrap();

    let sink = JsonlAuditSink::open(&path).unwrap();
    assert_eq!(
        sink.len(),
        1,
        "the torn (newline-less) record must not be indexed"
    );
    assert_eq!(sink.skipped_lines(), 1);

    // It must also be gone from disk (truncated), so a reopen agrees.
    let reopened = JsonlAuditSink::open(&path).unwrap();
    assert_eq!(reopened.len(), 1, "index and disk must agree after reopen");
    assert_eq!(
        reopened.skipped_lines(),
        0,
        "the torn tail was truncated, nothing to skip now"
    );
}

#[test]
fn a_complete_but_corrupt_interior_line_refuses_to_open() {
    // Codex doubt-pass (2026-07-17): a complete (newline-terminated) line that fails to
    // parse is corruption/tampering, not a torn write — it must be fatal, not silently
    // skipped, or a damaged interior record would vanish without a trace.
    let (_dir, path) = temp_log();
    let id = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        sink.write(one_of_each_variant().remove(0)).unwrap()
    };
    // Append a complete garbage line AFTER a good one, then another good line.
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(b"{not valid json at all}\n").unwrap();
    drop(file);
    let _ = id;

    match JsonlAuditSink::open(&path) {
        Err(OpenError::CorruptLine { line, .. }) => assert_eq!(line, 2),
        other => panic!("expected CorruptLine on a complete garbage interior line, got {other:?}"),
    }
}

#[test]
fn a_malformed_schema_version_on_a_complete_line_refuses_to_open() {
    // Codex doubt-pass (2026-07-17): a complete line whose schema_version isn't a plain
    // number (here a string) used to fail VersionProbe and be silently skipped as if torn,
    // bypassing the strict unknown-version path. It must be fatal.
    let (_dir, path) = temp_log();
    std::fs::write(
        &path,
        "{\"schema_version\":\"2\",\"id\":\"00000000-0000-0000-0000-000000000000\",\"event\":{}}\n",
    )
    .unwrap();
    match JsonlAuditSink::open(&path) {
        Err(OpenError::CorruptLine { line, .. }) => assert_eq!(line, 1),
        other => panic!("expected CorruptLine on a malformed schema_version, got {other:?}"),
    }
}

#[test]
fn a_duplicate_audit_id_refuses_to_open() {
    // Codex doubt-pass (2026-07-17): IDs are internal UUIDv4s, so a duplicate on replay
    // means the append-only log was spliced/tampered — fatal, not silently shadowed.
    let (_dir, path) = temp_log();
    let good_line = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        sink.write(one_of_each_variant().remove(0)).unwrap();
        std::fs::read_to_string(&path).unwrap()
    };
    // Append the exact same record line again (same id) — a spliced duplicate.
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(good_line.trim_end().as_bytes()).unwrap();
    file.write_all(b"\n").unwrap();
    drop(file);

    match JsonlAuditSink::open(&path) {
        Err(OpenError::DuplicateId { line, .. }) => assert_eq!(line, 2),
        other => panic!("expected DuplicateId on a spliced duplicate record, got {other:?}"),
    }
}

#[test]
fn invalid_utf8_in_the_torn_tail_is_tolerated_not_fatal() {
    // Codex doubt-pass (2026-07-17): a crash mid-multibyte-UTF-8 in the final line must not
    // brick the whole log — the earlier intact events must still open.
    let (_dir, path) = temp_log();
    let id = {
        let sink = JsonlAuditSink::open(&path).unwrap();
        sink.write(one_of_each_variant().remove(0)).unwrap()
    };
    // Append a torn final line containing a lone continuation byte (invalid UTF-8), no
    // trailing newline — a crash mid-multibyte sequence.
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(&[b'{', 0x80, 0xFF]).unwrap();
    drop(file);

    let sink =
        JsonlAuditSink::open(&path).expect("invalid UTF-8 in the torn tail must not brick the log");
    assert_eq!(sink.skipped_lines(), 1);
    assert!(sink.get(id).is_some(), "the intact earlier event survives");
}

#[test]
fn an_unknown_schema_version_refuses_to_open() {
    let (_dir, path) = temp_log();
    std::fs::write(
        &path,
        "{\"schema_version\":2,\"id\":\"00000000-0000-0000-0000-000000000000\",\"event\":{}}\n",
    )
    .unwrap();

    match JsonlAuditSink::open(&path) {
        Err(OpenError::UnknownSchemaVersion { line, version, .. }) => {
            assert_eq!(line, 1);
            assert_eq!(version, 2);
        }
        other => panic!("expected UnknownSchemaVersion, got {other:?}"),
    }
}

#[test]
fn concurrent_writers_all_land_durably() {
    let (_dir, path) = temp_log();
    let sink = Arc::new(JsonlAuditSink::open(&path).unwrap());

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let sink = Arc::clone(&sink);
            std::thread::spawn(move || {
                (0..10)
                    .map(|_| sink.write(one_of_each_variant().remove(4)).unwrap())
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let ids: Vec<_> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();

    assert_eq!(sink.len(), 80);
    for id in &ids {
        assert!(sink.get(*id).is_some());
    }
    let reopened = JsonlAuditSink::open(&path).unwrap();
    assert_eq!(reopened.len(), 80);
}

/// Raw values that exercise the exact leak class the conformance helper's doc warns
/// about: control characters and quotes render *escaped* in both `{event:?}` and JSON,
/// so a literal-substring check alone would false-negative on them.
const ADVERSARIAL_RAW_VALUES: &[&str] = &[
    "hunter2",
    "p@ss\nword",
    "tab\tseparated\tsecret",
    "quote\"inside\"quote",
    "back\\slash",
    "carriage\rreturn",
    "DE89370400440532013000",
    "sk-ant-api03-realish-key-material",
    "🔑-unicode-secret",
];

/// The T05b acceptance property test: events built per contract (refs/counts/versions
/// only) never serialise any raw detected value — checked against the *persisted bytes*
/// (literal and JSON-escaped forms), and against the Debug form via the conformance
/// helper.
#[test]
fn no_raw_value_is_ever_serialised() {
    let (_dir, path) = temp_log();
    let sink = JsonlAuditSink::open(&path).unwrap();

    // Write a full complement of events "about" sensitive inputs. Per the contract, the
    // events reference them only by count/ref/type — the raw values appear nowhere.
    for _ in ADVERSARIAL_RAW_VALUES {
        for event in one_of_each_variant() {
            vg_core::conformance::assert_audit_event_excludes_raw_values(
                &event,
                ADVERSARIAL_RAW_VALUES,
            );
            sink.write(event).unwrap();
        }
    }

    let persisted = std::fs::read_to_string(&path).unwrap();
    for raw in ADVERSARIAL_RAW_VALUES {
        assert!(
            !persisted.contains(raw),
            "raw value {raw:?} leaked into the audit log verbatim"
        );
        // JSON string escaping (serde_json) is what the persisted bytes would actually
        // contain if a raw value with control characters slipped into a String field.
        let json_escaped = serde_json::to_string(raw).unwrap();
        let json_escaped = json_escaped.trim_matches('"');
        assert!(
            !persisted.contains(json_escaped),
            "raw value {raw:?} leaked into the audit log JSON-escaped"
        );
    }
}

/// Negative control for the property test above: a writer that *does* embed a raw value
/// (here, in `Block.reason` — the one free-text field in the contract) must be caught by
/// the conformance helper, proving the test isn't passing vacuously.
#[test]
fn the_conformance_helper_catches_a_leaky_event() {
    for raw in ADVERSARIAL_RAW_VALUES {
        let leaky = AuditEvent::Block {
            artefact: ArtefactKind::LogLine,
            reason: format!("blocked because it contained {raw}"),
        };
        let caught = std::panic::catch_unwind(|| {
            vg_core::conformance::assert_audit_event_excludes_raw_values(
                &leaky,
                ADVERSARIAL_RAW_VALUES,
            );
        })
        .is_err();
        assert!(
            caught,
            "helper must flag a raw value ({raw:?}) embedded in an event"
        );
    }
}
