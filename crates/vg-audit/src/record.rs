//! Versioned on-disk record schema for the JSONL audit log.
//!
//! `vg_core::AuditEvent` is the frozen in-memory contract type and deliberately does not
//! derive serde — how events are persisted is `vg-audit`'s concern, not the contract's.
//! The types here are the *storage* schema: an explicit mirror of the v1 contract shapes,
//! with every record carrying `schema_version` so the format can evolve (a later build
//! adds `RecordV2` and keeps parsing v1 lines) without breaking old logs.
//!
//! Mirroring instead of deriving on the contract types is a feature, not duplication:
//! a change to a `vg-core` type cannot silently change what old audit logs mean — it
//! surfaces here as a compile error in the conversions below, forcing a deliberate
//! schema-version decision.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vg_core::{
    ActorId, ArtefactKind, AuditError, AuditEvent, Destination, EntityCounts, EntityType,
    HandlingClass, MappingRef,
};

/// The newest schema version this build writes (and the only one it currently reads).
pub(crate) const SCHEMA_VERSION: u32 = 1;

/// UUIDs as canonical hyphenated strings on the wire, via `Display`/`parse_str` rather
/// than the `uuid/serde` feature — see the dependency note in this crate's `Cargo.toml`
/// for why that feature is deliberately not enabled.
mod uuid_string {
    use serde::{de::Error as _, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S: Serializer>(u: &Uuid, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(u)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Uuid, D::Error> {
        let s = String::deserialize(d)?;
        Uuid::parse_str(&s).map_err(D::Error::custom)
    }
}

/// Minimal first pass over a line, to dispatch on `schema_version` before committing to
/// a concrete record shape.
#[derive(Debug, Deserialize)]
pub(crate) struct VersionProbe {
    pub schema_version: u32,
}

/// One persisted audit record: one line of the JSONL file.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RecordV1 {
    pub schema_version: u32,
    #[serde(with = "uuid_string")]
    pub id: Uuid,
    pub event: EventV1,
}

/// Storage mirror of [`vg_core::AuditEvent`] (contract v1, all six variants).
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum EventV1 {
    Scan {
        counts: Vec<CountV1>,
        detector_version: String,
        latency_us: u64,
    },
    PolicyDecision {
        artefact: ArtefactKindV1,
        class: HandlingClassV1,
        policy_version: String,
    },
    MappingCreated {
        #[serde(with = "uuid_string")]
        mapping_ref: Uuid,
        entity_type: EntityTypeV1,
    },
    Block {
        artefact: ArtefactKindV1,
        reason: String,
    },
    DemaskRequest {
        dest: DestinationV1,
        actor: String,
    },
    DemaskDecision {
        dest: DestinationV1,
        actor: String,
        allowed: bool,
        policy_version: String,
    },
}

/// One `(entity type, count)` pair from [`vg_core::EntityCounts`]. A struct rather than
/// a JSON map because `EntityType` keys (e.g. `Custom("...")`) are not plain strings.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CountV1 {
    pub entity_type: EntityTypeV1,
    pub count: u64,
}

/// Storage mirror of [`vg_core::EntityType`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EntityTypeV1 {
    Person,
    Email,
    Phone,
    Address,
    Postcode,
    EmployeeId,
    CustomerId,
    AccountId,
    Iban,
    SortCode,
    InternalIp,
    Hostname,
    ApiKey,
    TraceId,
    Password,
    PrivateKey,
    Secret,
    AccessToken,
    Custom(String),
}

/// Storage mirror of [`vg_core::ArtefactKind`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArtefactKindV1 {
    Json,
    Yaml,
    Toml,
    Sql,
    Csv,
    LogLine,
    Diff,
    EnvFile,
    SourceCode(String),
    PlainText,
    Unknown,
}

/// Storage mirror of [`vg_core::HandlingClass`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HandlingClassV1 {
    Mask,
    IrreversibleRedact,
    Block,
    Pass,
}

/// Storage mirror of [`vg_core::Destination`]. kebab-case so the serialized strings are
/// exactly the stable `DestinationId` keys from `Destination::id()` (`"local-patch"`,
/// `"remote-model-prompt"`, ...), tested below — the audit log and policy dictionaries
/// then speak the same destination vocabulary.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DestinationV1 {
    LocalPatch,
    LocalTestFixture,
    LocalExplanationBuffer,
    RemoteModelPrompt,
    ObservabilitySink,
}

/// Builds the error for a contract variant this storage schema doesn't cover yet.
///
/// **Codex cross-model doubt-pass finding (2026-07-17):** the original passed
/// `format!("EntityType::{other:?}")` etc. — Debug-formatting the *value* of the unknown
/// variant into the error string. Since a future variant could carry a raw sensitive value,
/// and `AuditError::Write` is a string a caller may well log, that turned the one tool whose
/// entire purpose is keeping raw values out of side channels into a raw-value side channel.
/// This now names only the *type*, never the value. The variant is unambiguous from the
/// calling code (a future contract variant that reaches this arm is one someone just added),
/// so no diagnostic power is lost.
fn unsupported(type_name: &str) -> AuditError {
    AuditError::Write(format!(
        "cannot persist this {type_name}: a variant not covered by audit storage schema \
         v{SCHEMA_VERSION} (a new contract variant needs a schema addition here first). \
         The variant's payload is deliberately omitted — a redaction tool must not echo \
         possibly-sensitive event data into error text."
    ))
}

// `EntityType`, `ArtefactKind`, and `AuditEvent` are `#[non_exhaustive]`, so every
// conversion toward storage is fallible: a future contract variant must fail loudly at
// write time rather than be silently dropped or mis-filed.

impl TryFrom<&EntityType> for EntityTypeV1 {
    type Error = AuditError;

    fn try_from(ty: &EntityType) -> Result<Self, AuditError> {
        Ok(match ty {
            EntityType::Person => Self::Person,
            EntityType::Email => Self::Email,
            EntityType::Phone => Self::Phone,
            EntityType::Address => Self::Address,
            EntityType::Postcode => Self::Postcode,
            EntityType::EmployeeId => Self::EmployeeId,
            EntityType::CustomerId => Self::CustomerId,
            EntityType::AccountId => Self::AccountId,
            EntityType::Iban => Self::Iban,
            EntityType::SortCode => Self::SortCode,
            EntityType::InternalIp => Self::InternalIp,
            EntityType::Hostname => Self::Hostname,
            EntityType::ApiKey => Self::ApiKey,
            EntityType::TraceId => Self::TraceId,
            EntityType::Password => Self::Password,
            EntityType::PrivateKey => Self::PrivateKey,
            EntityType::Secret => Self::Secret,
            EntityType::AccessToken => Self::AccessToken,
            EntityType::Custom(name) => Self::Custom(name.clone()),
            _ => return Err(unsupported("EntityType")),
        })
    }
}

impl From<EntityTypeV1> for EntityType {
    fn from(ty: EntityTypeV1) -> Self {
        match ty {
            EntityTypeV1::Person => Self::Person,
            EntityTypeV1::Email => Self::Email,
            EntityTypeV1::Phone => Self::Phone,
            EntityTypeV1::Address => Self::Address,
            EntityTypeV1::Postcode => Self::Postcode,
            EntityTypeV1::EmployeeId => Self::EmployeeId,
            EntityTypeV1::CustomerId => Self::CustomerId,
            EntityTypeV1::AccountId => Self::AccountId,
            EntityTypeV1::Iban => Self::Iban,
            EntityTypeV1::SortCode => Self::SortCode,
            EntityTypeV1::InternalIp => Self::InternalIp,
            EntityTypeV1::Hostname => Self::Hostname,
            EntityTypeV1::ApiKey => Self::ApiKey,
            EntityTypeV1::TraceId => Self::TraceId,
            EntityTypeV1::Password => Self::Password,
            EntityTypeV1::PrivateKey => Self::PrivateKey,
            EntityTypeV1::Secret => Self::Secret,
            EntityTypeV1::AccessToken => Self::AccessToken,
            EntityTypeV1::Custom(name) => Self::Custom(name),
        }
    }
}

impl TryFrom<&ArtefactKind> for ArtefactKindV1 {
    type Error = AuditError;

    fn try_from(kind: &ArtefactKind) -> Result<Self, AuditError> {
        Ok(match kind {
            ArtefactKind::Json => Self::Json,
            ArtefactKind::Yaml => Self::Yaml,
            ArtefactKind::Toml => Self::Toml,
            ArtefactKind::Sql => Self::Sql,
            ArtefactKind::Csv => Self::Csv,
            ArtefactKind::LogLine => Self::LogLine,
            ArtefactKind::Diff => Self::Diff,
            ArtefactKind::EnvFile => Self::EnvFile,
            ArtefactKind::SourceCode(lang) => Self::SourceCode(lang.clone()),
            ArtefactKind::PlainText => Self::PlainText,
            ArtefactKind::Unknown => Self::Unknown,
            _ => return Err(unsupported("ArtefactKind")),
        })
    }
}

impl From<ArtefactKindV1> for ArtefactKind {
    fn from(kind: ArtefactKindV1) -> Self {
        match kind {
            ArtefactKindV1::Json => Self::Json,
            ArtefactKindV1::Yaml => Self::Yaml,
            ArtefactKindV1::Toml => Self::Toml,
            ArtefactKindV1::Sql => Self::Sql,
            ArtefactKindV1::Csv => Self::Csv,
            ArtefactKindV1::LogLine => Self::LogLine,
            ArtefactKindV1::Diff => Self::Diff,
            ArtefactKindV1::EnvFile => Self::EnvFile,
            ArtefactKindV1::SourceCode(lang) => Self::SourceCode(lang),
            ArtefactKindV1::PlainText => Self::PlainText,
            ArtefactKindV1::Unknown => Self::Unknown,
        }
    }
}

// `HandlingClass` is exhaustive in contract v1, so its conversions are infallible — if
// a variant is ever added, that is a contract change and the compiler flags the match
// here. `Destination` is `#[non_exhaustive]`, so it takes the fallible path like
// `EntityType`/`ArtefactKind`.

impl From<HandlingClass> for HandlingClassV1 {
    fn from(class: HandlingClass) -> Self {
        match class {
            HandlingClass::Mask => Self::Mask,
            HandlingClass::IrreversibleRedact => Self::IrreversibleRedact,
            HandlingClass::Block => Self::Block,
            HandlingClass::Pass => Self::Pass,
        }
    }
}

impl From<HandlingClassV1> for HandlingClass {
    fn from(class: HandlingClassV1) -> Self {
        match class {
            HandlingClassV1::Mask => Self::Mask,
            HandlingClassV1::IrreversibleRedact => Self::IrreversibleRedact,
            HandlingClassV1::Block => Self::Block,
            HandlingClassV1::Pass => Self::Pass,
        }
    }
}

impl TryFrom<&Destination> for DestinationV1 {
    type Error = AuditError;

    fn try_from(dest: &Destination) -> Result<Self, AuditError> {
        Ok(match dest {
            Destination::LocalPatch => Self::LocalPatch,
            Destination::LocalTestFixture => Self::LocalTestFixture,
            Destination::LocalExplanationBuffer => Self::LocalExplanationBuffer,
            Destination::RemoteModelPrompt => Self::RemoteModelPrompt,
            Destination::ObservabilitySink => Self::ObservabilitySink,
            _ => return Err(unsupported("Destination")),
        })
    }
}

impl From<DestinationV1> for Destination {
    fn from(dest: DestinationV1) -> Self {
        match dest {
            DestinationV1::LocalPatch => Self::LocalPatch,
            DestinationV1::LocalTestFixture => Self::LocalTestFixture,
            DestinationV1::LocalExplanationBuffer => Self::LocalExplanationBuffer,
            DestinationV1::RemoteModelPrompt => Self::RemoteModelPrompt,
            DestinationV1::ObservabilitySink => Self::ObservabilitySink,
        }
    }
}

fn counts_to_v1(counts: &EntityCounts) -> Result<Vec<CountV1>, AuditError> {
    counts
        .0
        .iter()
        .map(|(ty, n)| {
            Ok(CountV1 {
                entity_type: EntityTypeV1::try_from(ty)?,
                count: *n as u64,
            })
        })
        .collect()
}

fn counts_from_v1(counts: Vec<CountV1>) -> EntityCounts {
    EntityCounts(
        counts
            .into_iter()
            // Saturating u64→usize is only lossy on a <64-bit target reading a log
            // written on a 64-bit one, and a saturated count is still "a lot".
            .map(|c| {
                (
                    c.entity_type.into(),
                    usize::try_from(c.count).unwrap_or(usize::MAX),
                )
            })
            .collect(),
    )
}

impl TryFrom<&AuditEvent> for EventV1 {
    type Error = AuditError;

    fn try_from(event: &AuditEvent) -> Result<Self, AuditError> {
        Ok(match event {
            AuditEvent::Scan {
                counts,
                detector_version,
                latency_us,
            } => Self::Scan {
                counts: counts_to_v1(counts)?,
                detector_version: detector_version.clone(),
                latency_us: *latency_us,
            },
            AuditEvent::PolicyDecision {
                artefact,
                class,
                policy_version,
            } => Self::PolicyDecision {
                artefact: artefact.try_into()?,
                class: (*class).into(),
                policy_version: policy_version.clone(),
            },
            AuditEvent::MappingCreated {
                mapping_ref,
                entity_type,
            } => Self::MappingCreated {
                mapping_ref: mapping_ref.0,
                entity_type: entity_type.try_into()?,
            },
            AuditEvent::Block { artefact, reason } => Self::Block {
                artefact: artefact.try_into()?,
                reason: reason.clone(),
            },
            AuditEvent::DemaskRequest { dest, actor } => Self::DemaskRequest {
                dest: dest.try_into()?,
                actor: actor.0.clone(),
            },
            AuditEvent::DemaskDecision {
                dest,
                actor,
                allowed,
                policy_version,
            } => Self::DemaskDecision {
                dest: dest.try_into()?,
                actor: actor.0.clone(),
                allowed: *allowed,
                policy_version: policy_version.clone(),
            },
            _ => return Err(unsupported("AuditEvent")),
        })
    }
}

impl From<EventV1> for AuditEvent {
    fn from(event: EventV1) -> Self {
        match event {
            EventV1::Scan {
                counts,
                detector_version,
                latency_us,
            } => Self::Scan {
                counts: counts_from_v1(counts),
                detector_version,
                latency_us,
            },
            EventV1::PolicyDecision {
                artefact,
                class,
                policy_version,
            } => Self::PolicyDecision {
                artefact: artefact.into(),
                class: class.into(),
                policy_version,
            },
            EventV1::MappingCreated {
                mapping_ref,
                entity_type,
            } => Self::MappingCreated {
                mapping_ref: MappingRef(mapping_ref),
                entity_type: entity_type.into(),
            },
            EventV1::Block { artefact, reason } => Self::Block {
                artefact: artefact.into(),
                reason,
            },
            EventV1::DemaskRequest { dest, actor } => Self::DemaskRequest {
                dest: dest.into(),
                actor: ActorId(actor),
            },
            EventV1::DemaskDecision {
                dest,
                actor,
                allowed,
                policy_version,
            } => Self::DemaskDecision {
                dest: dest.into(),
                actor: ActorId(actor),
                allowed,
                policy_version,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialized destination strings must match the stable `DestinationId` keys —
    /// see `DestinationV1`'s doc for why this alignment is deliberate.
    #[test]
    fn destination_serializes_to_the_stable_destination_id_string() {
        for dest in [
            Destination::LocalPatch,
            Destination::LocalTestFixture,
            Destination::LocalExplanationBuffer,
            Destination::RemoteModelPrompt,
            Destination::ObservabilitySink,
        ] {
            let expected = format!("\"{}\"", dest.id().0);
            let v1 = DestinationV1::try_from(&dest).unwrap();
            let serialized = serde_json::to_string(&v1).unwrap();
            assert_eq!(serialized, expected);
        }
    }

    /// The exact v1 wire shape, pinned. If this test breaks, the storage schema changed:
    /// that requires a `SCHEMA_VERSION` bump and a new record type, not an edit here.
    #[test]
    fn v1_wire_format_is_pinned() {
        let record = RecordV1 {
            schema_version: SCHEMA_VERSION,
            id: Uuid::nil(),
            event: EventV1::Scan {
                counts: vec![CountV1 {
                    entity_type: EntityTypeV1::Email,
                    count: 2,
                }],
                detector_version: "detectors-v1".to_string(),
                latency_us: 1500,
            },
        };
        assert_eq!(
            serde_json::to_string(&record).unwrap(),
            r#"{"schema_version":1,"id":"00000000-0000-0000-0000-000000000000","event":{"kind":"scan","counts":[{"entity_type":"email","count":2}],"detector_version":"detectors-v1","latency_us":1500}}"#
        );
    }

    #[test]
    fn custom_entity_type_roundtrips_with_its_dictionary_name() {
        let ty = EntityType::Custom("internal-project-codename".to_string());
        let v1 = EntityTypeV1::try_from(&ty).unwrap();
        let json = serde_json::to_string(&v1).unwrap();
        assert_eq!(json, r#"{"custom":"internal-project-codename"}"#);
        let back: EntityTypeV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(EntityType::from(back), ty);
    }
}
