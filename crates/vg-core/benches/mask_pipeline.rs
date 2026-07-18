//! Criterion bench for the full `vg_core::mask` pipeline against a realistic reference
//! artefact, tracking the interface contract's p95 < 25ms budget end-to-end
//! (`docs/architecture/interface-contracts.md` §3) — the pipeline analogue of
//! `vg-detectors/benches/detectors.rs`, which benches the detectors alone.
//!
//! Like that bench, CI only compile-checks this (`cargo bench --workspace --locked
//! --no-run`); running it for real (precise, statistically-sound measurement) is a
//! manual/local step until Task T10 wires up real baseline management. The 25ms figure is
//! the *hot-path detection* target; a full `mask` additionally does vault SQLCipher writes
//! and a per-call fsync'd audit append, so the steady-state number here is expected to sit
//! above pure detection — this bench exists to make that end-to-end number visible and
//! track its regressions, with `tests/pipeline_latency_gate.rs` as the coarse CI-enforced
//! companion.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::TempDir;

use vg_audit::JsonlAuditSink;
use vg_core::{
    mask, ArtefactHint, Context, Detector, Input, Namespace, Parser, Policy, PolicyEngine,
    PolicyLayers, RepoId,
};
use vg_detectors::all_detectors;
use vg_parsers::all_parsers;
use vg_policy::LayeredPolicyEngine;
use vg_vault::{Vault, VaultConfig};

const TEST_KEY: [u8; 32] = [7u8; 32];

const REFERENCE_ARTEFACT: &str = r#"
2026-07-15T10:22:31Z INFO  request from 10.0.0.42 user=jane.doe@example.com
  Authorization Bearer FAKEBENCHSECRET9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a
  customer phone +1-415-555-2671, iban GB29 NWBK 6016 1331 9268 19, sort code 12-34-56
  ipv6 peer 2001:0db8:85a3:0000:0000:8a2e:0370:7334
  the quick brown fox jumps over the lazy dog while nothing sensitive happens here
"#;

fn bench_mask_pipeline(c: &mut Criterion) {
    let dir = TempDir::new().expect("temp dir");
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../vg-policy/fixtures/global.policy.json"),
        repo: None,
        session: None,
    })
    .expect("load policy");
    let vault = Vault::open_with_key(VaultConfig::new(dir.path().join("vault.db")), TEST_KEY)
        .expect("open vault");
    let audit = JsonlAuditSink::open(dir.path().join("audit.jsonl")).expect("open audit");
    let policy = Policy {
        engine: Box::new(engine),
        vault: Box::new(vault),
        audit: Box::new(audit),
    };
    let ns = Namespace::Repo(RepoId("bench".to_string()));

    let detectors = all_detectors();
    let detector_refs: Vec<&dyn Detector> = detectors.iter().map(|d| d.as_ref()).collect();
    let parsers = all_parsers();
    let parser_refs: Vec<&dyn Parser> = parsers.iter().map(|p| p.as_ref()).collect();
    let ctx = Context {
        parsers: &parser_refs,
        detectors: &detector_refs,
    };

    let input = Input {
        buf: REFERENCE_ARTEFACT.as_bytes().to_vec(),
        hint: ArtefactHint::default(),
    };

    c.bench_function("mask/reference_artefact", |b| {
        b.iter(|| black_box(mask(black_box(&input), &ctx, &policy, &ns).expect("mask succeeds")))
    });
}

criterion_group!(benches, bench_mask_pipeline);
criterion_main!(benches);
