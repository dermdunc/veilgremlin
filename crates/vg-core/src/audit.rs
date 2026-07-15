//! `AuditEvent` — the append-only audit record type. Owned by `vg-core` (not
//! `vg-audit`) because the `AuditSink` trait, also frozen in `vg-core`, is defined over
//! it; `vg-audit` (Squad 5) implements `AuditSink`, it does not own this type.

use crate::api::Destination;
use crate::ids::ActorId;
use crate::traits::ArtefactKind;
use crate::types::{EntityCounts, EntityType, HandlingClass, MappingRef};

/// An append-only audit record. **Contract: no raw values in any variant** — refs,
/// counts, and versions only. Checked by
/// [`crate::conformance::assert_audit_event_excludes_raw_values`].
///
/// `#[non_exhaustive]`: the draft contract's own ellipsis ("... provider destination,
/// build_provenance_version") signals more variants land later (Task T09/T10) — adding
/// them is additive, not a breaking change, and still goes through the contract-change
/// protocol.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AuditEvent {
    Scan {
        counts: EntityCounts,
        detector_version: String,
        latency_us: u64,
    },
    PolicyDecision {
        artefact: ArtefactKind,
        class: HandlingClass,
        policy_version: String,
    },
    MappingCreated {
        mapping_ref: MappingRef,
        entity_type: EntityType,
    },
    Block {
        artefact: ArtefactKind,
        reason: String,
    },
    DemaskRequest {
        dest: Destination,
        actor: ActorId,
    },
    DemaskDecision {
        dest: Destination,
        actor: ActorId,
        allowed: bool,
        policy_version: String,
    },
}
