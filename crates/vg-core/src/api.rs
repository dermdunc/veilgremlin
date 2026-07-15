//! Library API owned by `vg-core`: `scan`/`mask`/`rehydrate`/`benchmark`, plus the
//! supporting types those signatures need (`Input`, `Context`, `Policy`, `Actor`,
//! `Destination`, `Corpus`, `Metrics`).
//!
//! T02 (this task) freezes the signatures and implements the one piece of real logic
//! that does not depend on any Wave B crate: `rehydrate`'s destination hard-deny gate.
//! The rest of each body is pipeline assembly (detectors → policy → vault → masked pack)
//! and is wired in Task T07 once Wave B's trait implementations exist — see
//! `docs/architecture/agent-factory-plan.md` §3.

use crate::audit::AuditEvent;
use crate::error::{MaskError, RehydrateDenied};
use crate::ids::ActorId;
use crate::traits::{ArtefactHint, AuditSink, Detector, Parser, PolicyEngine, VaultStore};
use crate::types::{Finding, MappingRef, MaskedPack, Namespace};

/// Raw bytes plus whatever hint the caller already has about their shape.
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub buf: Vec<u8>,
    pub hint: ArtefactHint,
}

/// The detectors and parsers `scan` runs, borrowed as trait objects so `vg-core` never
/// depends on the Wave B crates that implement them.
pub struct Context<'a> {
    pub parsers: &'a [&'a dyn Parser],
    pub detectors: &'a [&'a dyn Detector],
}

/// A resolved policy plus the vault/audit handles `mask` needs to classify, intern, and
/// record its decision. Bundled into one struct (rather than three separate `mask`
/// parameters) because `mask` always needs all three together.
pub struct Policy {
    pub engine: Box<dyn PolicyEngine>,
    pub vault: Box<dyn VaultStore>,
    pub audit: Box<dyn AuditSink>,
}

/// Where a (un)masked value is headed. `RemoteModelPrompt` and `ObservabilitySink` are
/// **hard-deny**: see [`rehydrate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Destination {
    LocalPatch,
    LocalTestFixture,
    LocalExplanationBuffer,
    RemoteModelPrompt,
    ObservabilitySink,
}

impl Destination {
    /// Stable key for `PolicyEngine::destination_allows_masked_only` lookups —
    /// deliberately a separate type ([`DestinationId`]) from `Destination` itself, since
    /// policy dictionaries key on a stable string, not the richer runtime enum.
    pub fn id(&self) -> DestinationId {
        let s = match self {
            Destination::LocalPatch => "local-patch",
            Destination::LocalTestFixture => "local-test-fixture",
            Destination::LocalExplanationBuffer => "local-explanation-buffer",
            Destination::RemoteModelPrompt => "remote-model-prompt",
            Destination::ObservabilitySink => "observability-sink",
        };
        DestinationId(s.to_string())
    }

    /// True for destinations `rehydrate` denies regardless of actor or `PolicyEngine`
    /// impl — the "hard-deny... regardless of actor" invariant from
    /// `interface-contracts.md` §2.
    fn is_hard_deny(&self) -> bool {
        matches!(
            self,
            Destination::RemoteModelPrompt | Destination::ObservabilitySink
        )
    }
}

/// Lightweight key `PolicyEngine` impls use to look up destination policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DestinationId(pub String);

/// Who is requesting a demask, checked by `PolicyEngine::demask_allowed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub id: ActorId,
    pub roles: Vec<String>,
}

/// A seeded evaluation corpus for `benchmark` (Task T10 populates real corpora).
#[derive(Debug, Clone, Default)]
pub struct Corpus {
    pub samples: Vec<CorpusSample>,
}

#[derive(Debug, Clone)]
pub struct CorpusSample {
    pub input: Input,
    pub expected_findings: Vec<Finding>,
}

/// Go/No-Go metrics, per `agent-factory-plan.md` §8.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Metrics {
    pub recall: f64,
    pub precision: f64,
    pub false_positive_rate: f64,
    pub p95_latency_us: u64,
}

/// Runs parsers then detectors over `input`. Pipeline assembly wired in Task T07.
pub fn scan(input: &Input, ctx: &Context) -> Vec<Finding> {
    let _ = (input, ctx);
    todo!("pipeline assembly wired in Task T07 (vg-core pipeline wiring)")
}

/// Detect, classify, and mask `input` per `policy`, interning reversible values into the
/// vault and recording an audit event. Pipeline assembly wired in Task T07.
pub fn mask(
    input: &Input,
    policy: &Policy,
    ns: &Namespace,
) -> Result<(MaskedPack, Vec<MappingRef>, AuditEvent), MaskError> {
    let _ = (input, policy, ns);
    todo!("pipeline assembly wired in Task T07 (vg-core pipeline wiring)")
}

/// Reverses a masked placeholder back to its raw value for an authorised, local
/// destination.
///
/// The destination hard-deny gate below does not depend on any Wave B crate, so it is
/// implemented for real here rather than deferred: `RemoteModelPrompt` and
/// `ObservabilitySink` are denied unconditionally, before any `PolicyEngine` or
/// `VaultStore` is even consulted. Resolving an *allowed* destination still needs a
/// wired `VaultStore` + `PolicyEngine`, so that half is deferred to Task T07/T09.
pub fn rehydrate(
    masked: &str,
    dest: Destination,
    actor: &Actor,
) -> Result<String, RehydrateDenied> {
    if dest.is_hard_deny() {
        return Err(RehydrateDenied {
            destination: dest,
            actor: actor.id.clone(),
            reason: "destination is hard-deny in default policy".to_string(),
        });
    }
    let _ = masked;
    todo!("vault/policy-authorised resolution wired in Task T07/T09 (vg-cli demask gate)")
}

/// Runs `corpus` through the pipeline and reports Go/No-Go metrics. Wired in Task T10.
pub fn benchmark(corpus: &Corpus, policy: &Policy) -> Metrics {
    let _ = (corpus, policy);
    todo!("eval harness wired in Task T10 (vg-bench)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(id: &str) -> Actor {
        Actor {
            id: ActorId(id.to_string()),
            roles: vec!["admin".to_string()],
        }
    }

    #[test]
    fn rehydrate_denies_remote_model_prompt_regardless_of_actor() {
        let err = rehydrate(
            "{{EMAIL_1}}",
            Destination::RemoteModelPrompt,
            &actor("admin-1"),
        )
        .expect_err("RemoteModelPrompt must be hard-denied even for an admin actor");
        assert_eq!(err.destination, Destination::RemoteModelPrompt);
    }

    #[test]
    fn rehydrate_denies_observability_sink_regardless_of_actor() {
        let err = rehydrate(
            "{{EMAIL_1}}",
            Destination::ObservabilitySink,
            &actor("admin-1"),
        )
        .expect_err("ObservabilitySink must be hard-denied even for an admin actor");
        assert_eq!(err.destination, Destination::ObservabilitySink);
    }

    #[test]
    #[should_panic(expected = "wired in Task T07/T09")]
    fn rehydrate_allowed_destination_is_not_yet_wired() {
        // Documents current state rather than asserting a real resolution: the
        // vault/policy-authorised path is out of scope for T02 and lands in T07/T09.
        let _ = rehydrate("{{EMAIL_1}}", Destination::LocalPatch, &actor("admin-1"));
    }

    #[test]
    fn destination_id_is_stable_for_policy_lookups() {
        assert_eq!(
            Destination::RemoteModelPrompt.id(),
            DestinationId("remote-model-prompt".to_string())
        );
    }
}
