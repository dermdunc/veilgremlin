//! Behavioural tests for `LayeredPolicyEngine`, driven off the JSON fixtures in
//! `crates/vg-policy/fixtures/`. Covers the acceptance criteria from Task T06: example
//! pack parses, 3-layer resolution with overlapping keys, and the deny rules — including
//! the security-load-bearing hard-deny checked via `vg-core`'s conformance helper.

use std::path::{Path, PathBuf};

use vg_core::{
    Actor, ActorId, ArtefactHint, Destination, DestinationId, EntityType, HandlingClass,
    PolicyEngine, PolicyError, PolicyLayers,
};
use vg_policy::LayeredPolicyEngine;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn actor(roles: &[&str]) -> Actor {
    Actor {
        id: ActorId("actor-1".to_string()),
        roles: roles.iter().map(|r| r.to_string()).collect(),
    }
}

/// The global layer alone parses and classifies — the "example policy parses" acceptance
/// criterion at its most basic.
#[test]
fn global_layer_alone_loads_and_classifies() {
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("global.policy.json"),
        repo: None,
        session: None,
    })
    .expect("global fixture must load");

    assert_eq!(engine.version(), "veilgremlin-global-v1");
    assert_eq!(
        engine.classify_entity(EntityType::Email),
        HandlingClass::Mask
    );
    assert_eq!(
        engine.classify_entity(EntityType::Password),
        HandlingClass::IrreversibleRedact
    );
    assert_eq!(
        engine.classify_entity(EntityType::Hostname),
        HandlingClass::Pass
    );
    // Not named anywhere -> the pack's `default` (mask).
    assert_eq!(
        engine.classify_entity(EntityType::Iban),
        HandlingClass::Mask
    );
}

/// All three layers present with overlapping keys: session overrides repo overrides
/// global, resolved key-by-key (not whole-layer).
#[test]
fn three_layer_resolution_overrides_key_by_key() {
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("global.policy.json"),
        repo: Some(fixture("repo.policy.json")),
        session: Some(fixture("session.policy.json")),
    })
    .expect("all three fixtures must load");

    // version: session wins outright.
    assert_eq!(engine.version(), "veilgremlin-session-v1");

    // email: global=mask, session=irreversible-redact -> session wins.
    assert_eq!(
        engine.classify_entity(EntityType::Email),
        HandlingClass::IrreversibleRedact
    );
    // person: global=mask, session=block -> session wins.
    assert_eq!(
        engine.classify_entity(EntityType::Person),
        HandlingClass::Block
    );
    // hostname: global=pass, repo=irreversible-redact, session silent -> repo wins over global.
    assert_eq!(
        engine.classify_entity(EntityType::Hostname),
        HandlingClass::IrreversibleRedact
    );
    // customer-id: only repo names it.
    assert_eq!(
        engine.classify_entity(EntityType::CustomerId),
        HandlingClass::Mask
    );
    // phone: named by no layer -> global default (mask).
    assert_eq!(
        engine.classify_entity(EntityType::Phone),
        HandlingClass::Mask
    );

    // artefact rules come only from global here and still apply after merge.
    // Extension match is case-insensitive: `.ENV` resolves via the `env` rule.
    let env_file = ArtefactHint {
        path: Some(PathBuf::from("deploy/production.ENV")),
        language_id: None,
        mime_type: None,
    };
    assert_eq!(engine.classify_artefact(&env_file), HandlingClass::Block);
    let readme = ArtefactHint {
        path: Some(PathBuf::from("README.md")),
        language_id: None,
        mime_type: None,
    };
    assert_eq!(engine.classify_artefact(&readme), HandlingClass::Pass);
}

/// `destination_allows_masked_only`: configured values, the fail-safe default for an
/// unknown destination, and the forced-true for hard-deny destinations.
#[test]
fn masked_only_send_gate() {
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("global.policy.json"),
        repo: None,
        session: None,
    })
    .expect("global fixture must load");

    assert!(engine.destination_allows_masked_only(&Destination::RemoteModelPrompt.id()));
    assert!(engine.destination_allows_masked_only(&Destination::ObservabilitySink.id()));
    assert!(!engine.destination_allows_masked_only(&Destination::LocalPatch.id()));
    // Unknown destination -> fail-safe true (require masked-only).
    assert!(engine.destination_allows_masked_only(&DestinationId("some-future-sink".to_string())));
}

/// Demask on an allowed destination is role-gated when the pack lists required roles.
#[test]
fn demask_allowed_is_role_gated_for_configured_destinations() {
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("global.policy.json"),
        repo: Some(fixture("repo.policy.json")),
        session: None,
    })
    .expect("fixtures must load");

    // local-patch: demask_allowed, no role restriction -> anyone.
    assert!(engine.demask_allowed(Destination::LocalPatch, &actor(&[])));

    // local-explanation-buffer: repo enabled it and required roles [reviewer, admin].
    assert!(engine.demask_allowed(Destination::LocalExplanationBuffer, &actor(&["reviewer"])));
    assert!(!engine.demask_allowed(Destination::LocalExplanationBuffer, &actor(&["intern"])));
}

/// THE security-load-bearing check (T06 spec item 4): `demask_allowed` denies the two
/// hard-deny destinations for any actor, via `vg-core`'s conformance helper.
#[test]
fn demask_hard_denies_remote_and_observability() {
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("global.policy.json"),
        repo: None,
        session: None,
    })
    .expect("global fixture must load");

    let admin = actor(&["admin", "reviewer", "owner"]);
    vg_core::conformance::assert_policy_engine_denies_hard_deny_destinations(&engine, &admin);
}

/// The hard-deny is enforced in code, so a hostile pack that *tries* to enable demask to
/// the hard-deny destinations still cannot: config can never override the rule.
#[test]
fn malicious_pack_cannot_unlock_hard_deny_destinations() {
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("malicious-hard-deny.policy.json"),
        repo: None,
        session: None,
    })
    .expect("malicious fixture is still valid JSON and loads");

    let admin = actor(&["admin"]);
    // demask stays denied despite `demask_allowed: true` in the pack...
    assert!(!engine.demask_allowed(Destination::RemoteModelPrompt, &admin));
    assert!(!engine.demask_allowed(Destination::ObservabilitySink, &admin));
    // ...and the masked-only send gate stays true despite `masked_only: false` in the pack.
    assert!(engine.destination_allows_masked_only(&Destination::RemoteModelPrompt.id()));
    assert!(engine.destination_allows_masked_only(&Destination::ObservabilitySink.id()));
    // The conformance helper agrees.
    vg_core::conformance::assert_policy_engine_denies_hard_deny_destinations(&engine, &admin);
}

#[test]
fn missing_global_layer_is_a_load_error() {
    let err = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("does-not-exist.policy.json"),
        repo: None,
        session: None,
    })
    .expect_err("a missing global layer must fail to load");
    assert!(
        matches!(err, PolicyError::Load(_)),
        "expected Load, got {err:?}"
    );
}

#[test]
fn invalid_handling_class_is_a_load_error() {
    let err = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("invalid-class.policy.json"),
        repo: None,
        session: None,
    })
    .expect_err("an unknown handling class must fail to load");
    assert!(
        matches!(err, PolicyError::Load(_)),
        "expected Load, got {err:?}"
    );
}

#[test]
fn malformed_json_is_a_load_error() {
    let err = LayeredPolicyEngine::load(PolicyLayers {
        global: fixture("malformed.policy.json"),
        repo: None,
        session: None,
    })
    .expect_err("malformed JSON must fail to load");
    assert!(
        matches!(err, PolicyError::Load(_)),
        "expected Load, got {err:?}"
    );
}
