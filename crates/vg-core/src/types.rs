//! Shared types owned by `vg-core`, frozen per
//! `docs/architecture/interface-contracts.md` v1.

use std::collections::BTreeMap;

use uuid::Uuid;

use crate::ids::{DetectorId, OrgId, RepoId, SessionId};

/// Classification of a detected entity.
///
/// `Custom` supports policy-dictionary-defined classes (the contract's "extensible via
/// policy dictionaries" note) without requiring a contract-change PR for every new
/// dictionary entry — only genuinely new *fixed* classes need one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityType {
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
    /// A policy-dictionary-defined class, named by the dictionary (e.g.
    /// `"internal-project-codename"`).
    Custom(String),
}

/// What the policy says to do with a class of entity/artefact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlingClass {
    /// Reversible typed placeholder via vault.
    Mask,
    /// One-way; never vault-stored.
    IrreversibleRedact,
    /// Do not send (artefact-level).
    Block,
    /// Non-sensitive.
    Pass,
}

/// Namespace for placeholder stability: the same raw value maps to the same placeholder
/// only within the same namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Namespace {
    Session(SessionId),
    Repo(RepoId),
    Org(OrgId),
}

/// Structural context a `Parser` attaches to a `Span`, letting detectors be
/// field/structure-aware.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NodeKind {
    Key,
    Value,
    Field(String),
    StringLiteral,
    Comment,
    Identifier,
    Other(String),
}

/// Byte span + optional structural context from a parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub node_kind: Option<NodeKind>,
}

/// A single detection over an input buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub entity_type: EntityType,
    pub span: Span,
    pub confidence: f32,
    pub detector: DetectorId,
}

/// Opaque handle into the vault; never the raw value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MappingRef(pub Uuid);

/// Counts of findings by `EntityType`, used for audit/mask stats without exposing
/// values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityCounts(pub BTreeMap<EntityType, usize>);

/// Summary stats attached to a `MaskedPack`: counts by `EntityType`, blocked artefacts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaskStats {
    pub counts: EntityCounts,
    pub blocked_artefacts: usize,
}

/// The only thing serialized toward a model. **Contains no raw detected value or vault
/// key** — enforced by `mask()`'s implementation (Task T07) and checked by
/// [`crate::conformance::assert_masked_pack_excludes_raw_values`] in downstream
/// integration tests.
#[derive(Debug, Clone, PartialEq)]
pub struct MaskedPack {
    pub text: String,
    pub mapping_refs: Vec<MappingRef>,
    pub stats: MaskStats,
    pub policy_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::assert_masked_pack_excludes_raw_values;

    #[test]
    fn masked_pack_invariant_helper_flags_a_leaked_raw_value() {
        let leaking = MaskedPack {
            text: "contact jane.doe@example.com for details".to_string(),
            mapping_refs: vec![],
            stats: MaskStats::default(),
            policy_version: "v1".to_string(),
        };
        let result = std::panic::catch_unwind(|| {
            assert_masked_pack_excludes_raw_values(&leaking, &["jane.doe@example.com"]);
        });
        assert!(
            result.is_err(),
            "helper must panic when a raw value is present in MaskedPack.text"
        );
    }

    #[test]
    fn masked_pack_invariant_helper_passes_on_placeholder_only_text() {
        let masked = MaskedPack {
            text: "contact {{EMAIL_1}} for details".to_string(),
            mapping_refs: vec![MappingRef(Uuid::nil())],
            stats: MaskStats::default(),
            policy_version: "v1".to_string(),
        };
        assert_masked_pack_excludes_raw_values(&masked, &["jane.doe@example.com"]);
    }
}
