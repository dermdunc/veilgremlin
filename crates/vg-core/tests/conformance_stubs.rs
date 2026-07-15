//! Contract-conformance test scaffold (Task T02 acceptance: "conformance test scaffold
//! exists"). Exercises `vg_core::conformance`'s helpers against minimal mock
//! implementations of every trait seam, in the same shape Wave B squads
//! (`vg-detectors`, `vg-parsers`, `vg-vault`, `vg-policy`, `vg-audit`) will use in their
//! own crates once they implement these traits for real.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use uuid::Uuid;

use vg_core::{
    Actor, ActorId, ArtefactHint, ArtefactKind, AuditEvent, AuditId, AuditSink, Destination,
    DestinationId, Detector, DetectorId, EntityCounts, EntityType, Finding, HandlingClass,
    MappingRef, Namespace, ParseResult, Parser, Placeholder, PolicyEngine, PolicyError,
    PolicyLayers, Secret, SessionId, Span, VaultError, VaultStore,
};

struct MockDetector {
    entity_types: Vec<EntityType>,
}

impl Default for MockDetector {
    fn default() -> Self {
        Self {
            entity_types: vec![EntityType::Secret],
        }
    }
}

impl Detector for MockDetector {
    fn id(&self) -> DetectorId {
        DetectorId("mock-secret-detector".to_string())
    }

    fn detect(&self, buf: &[u8], _spans: &[Span]) -> Vec<Finding> {
        let text = String::from_utf8_lossy(buf);
        text.find("SECRET")
            .map(|pos| Finding {
                entity_type: EntityType::Secret,
                span: Span {
                    start: pos,
                    end: pos + "SECRET".len(),
                    node_kind: None,
                },
                confidence: 0.99,
                detector: self.id(),
            })
            .into_iter()
            .collect()
    }

    fn entity_types(&self) -> &[EntityType] {
        &self.entity_types
    }
}

struct MockParser;

impl Parser for MockParser {
    fn can_parse(&self, _artefact: &ArtefactHint) -> bool {
        true
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        ParseResult {
            spans: vec![Span {
                start: 0,
                end: buf.len(),
                node_kind: None,
            }],
            artefact_kind: ArtefactKind::PlainText,
        }
    }
}

/// TEST-ONLY cache key: a delimiter-joined string of the raw value, type, and
/// namespace. This is a shortcut for the mock, NOT the contract's specified scheme —
/// the real `vg-vault` (Task T05) must implement a salted HMAC over
/// `(canonical(value), ty, ns)` (interface-contracts.md §5), not string concatenation.
fn vault_key(value: &str, ty: &EntityType, ns: &Namespace) -> String {
    format!("{value}|{ty:?}|{ns:?}")
}

#[derive(Default)]
struct MockVault {
    forward: Mutex<HashMap<String, Placeholder>>,
    // Keyed by mapping_ref id; value is (raw value, namespace it was interned under).
    // Storing `ns` here (not just the raw value) is what makes namespace isolation on
    // resolve enforceable at all — see VaultStore::resolve's trait doc.
    reverse: Mutex<HashMap<Uuid, (String, Namespace)>>,
}

impl VaultStore for MockVault {
    fn intern(
        &self,
        value: &Secret,
        ty: EntityType,
        ns: &Namespace,
    ) -> Result<Placeholder, VaultError> {
        let key = vault_key(value.expose_secret(), &ty, ns);
        let mut forward = self.forward.lock().unwrap();
        if let Some(existing) = forward.get(&key) {
            return Ok(existing.clone());
        }
        let id = Uuid::new_v4();
        let placeholder = Placeholder {
            display: format!("{{{{{ty:?}_1}}}}"),
            mapping_ref: MappingRef(id),
        };
        forward.insert(key, placeholder.clone());
        self.reverse
            .lock()
            .unwrap()
            .insert(id, (value.expose_secret().to_string(), ns.clone()));
        Ok(placeholder)
    }

    fn resolve(&self, p: &Placeholder, ns: &Namespace) -> Result<Secret, VaultError> {
        let reverse = self.reverse.lock().unwrap();
        let (value, interned_ns) = reverse.get(&p.mapping_ref.0).ok_or(VaultError::NotFound)?;
        if interned_ns != ns {
            // Namespace mismatch: this is the isolation invariant, not a missing entry —
            // still NotFound from the caller's perspective (never leak that a mapping
            // exists in a *different* namespace).
            return Err(VaultError::NotFound);
        }
        Ok(Secret::new(value.clone()))
    }

    fn purge_expired(&self) -> Result<usize, VaultError> {
        Ok(0)
    }
}

struct MockPolicyEngine {
    version: String,
}

impl PolicyEngine for MockPolicyEngine {
    fn load(_layers: PolicyLayers) -> Result<Self, PolicyError> {
        Ok(Self {
            version: "test-v0".to_string(),
        })
    }

    fn classify_artefact(&self, _hint: &ArtefactHint) -> HandlingClass {
        HandlingClass::Pass
    }

    fn classify_entity(&self, ty: EntityType) -> HandlingClass {
        match ty {
            EntityType::Secret
            | EntityType::Password
            | EntityType::PrivateKey
            | EntityType::ApiKey
            | EntityType::AccessToken => HandlingClass::IrreversibleRedact,
            _ => HandlingClass::Mask,
        }
    }

    fn destination_allows_masked_only(&self, _dest: &DestinationId) -> bool {
        true
    }

    fn demask_allowed(&self, dest: Destination, _actor: &Actor) -> bool {
        !matches!(
            dest,
            Destination::RemoteModelPrompt | Destination::ObservabilitySink
        )
    }

    fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Default)]
struct MockAuditSink {
    events: Mutex<HashMap<Uuid, AuditEvent>>,
}

impl AuditSink for MockAuditSink {
    fn write(&self, event: AuditEvent) -> Result<AuditId, vg_core::AuditError> {
        let id = Uuid::new_v4();
        self.events.lock().unwrap().insert(id, event);
        Ok(AuditId(id))
    }

    fn get(&self, id: AuditId) -> Option<AuditEvent> {
        self.events.lock().unwrap().get(&id.0).cloned()
    }
}

#[test]
fn detector_satisfies_the_contract() {
    let detector = MockDetector::default();
    vg_core::conformance::assert_detector_contract(&detector, b"token=SECRETvalue", &[]);
}

#[test]
fn parser_never_panics_on_malformed_input() {
    let parser = MockParser;
    vg_core::conformance::assert_parser_never_panics(&parser, b"{ this is not valid json");
}

#[test]
fn vault_roundtrips_and_is_stable_within_a_namespace() {
    let vault = MockVault::default();
    let ns = Namespace::Session(SessionId(Uuid::nil()));
    let other_ns = Namespace::Session(SessionId(Uuid::from_u128(1)));
    vg_core::conformance::assert_vault_roundtrip(
        &vault,
        "s3cr3t-value",
        EntityType::Secret,
        &ns,
        &other_ns,
    );
}

#[test]
fn audit_sink_roundtrips_a_written_event() {
    let sink = MockAuditSink::default();
    let event = AuditEvent::Scan {
        counts: EntityCounts::default(),
        detector_version: "mock-1".to_string(),
        latency_us: 10,
    };
    vg_core::conformance::assert_audit_sink_roundtrip(&sink, event);
}

#[test]
fn audit_event_never_embeds_a_raw_value() {
    let event = AuditEvent::MappingCreated {
        mapping_ref: MappingRef(Uuid::nil()),
        entity_type: EntityType::Email,
    };
    vg_core::conformance::assert_audit_event_excludes_raw_values(&event, &["jane.doe@example.com"]);
}

#[test]
fn policy_engine_hard_denies_remote_and_observability_destinations() {
    let engine = MockPolicyEngine::load(PolicyLayers {
        global: PathBuf::from("policy.yaml"),
        repo: None,
        session: None,
    })
    .expect("mock policy load always succeeds");
    let actor = Actor {
        id: ActorId("actor-1".to_string()),
        roles: vec!["admin".to_string()],
    };

    vg_core::conformance::assert_policy_engine_denies_hard_deny_destinations(&engine, &actor);
    assert!(engine.demask_allowed(Destination::LocalPatch, &actor));
}

#[test]
fn parser_never_panics_on_a_battery_of_adversarial_buffers() {
    // MockParser is trivially safe by construction (see its impl above), so this can
    // only prove the harness call itself doesn't crash for THIS parser — it cannot
    // verify panic-safety in general. A real parser (tree-sitter, JSON/YAML) copying
    // this template must run its own equivalent battery against its actual logic.
    let parser = MockParser;
    for buf in [
        &b""[..],
        b"{ this is not valid json",
        &[0xFF, 0xFE, 0x00, 0x80][..], // invalid UTF-8
        &vec![b'{'; 10_000][..],       // deeply unbalanced/large input
    ] {
        vg_core::conformance::assert_parser_never_panics(&parser, buf);
    }
}
