//! CI-enforced latency regression gate for the **full `mask` pipeline** — the pipeline
//! analogue of `vg-detectors`'s `tests/latency_gate.rs` (its documented precedent). A
//! plain `#[test]`, so it runs on every PR via the existing `cargo test` CI job with no
//! new CI config, and fails loudly on a gross regression without needing the criterion
//! bench (`benches/mask_pipeline.rs`) to be run by a human first.
//!
//! Two deliberate differences from the detector gate, both because `mask` is a *full*
//! pipeline, not the pure hot path:
//!
//! 1. **A single realistic artefact, not the 200x-repeated reference buffer.** A real
//!    `mask` call masks one artefact; concatenating 200 copies would intern hundreds of
//!    values per call (each a SQLCipher write) and measure batch throughput, not
//!    per-call latency.
//! 2. **A larger CI-slack factor.** `mask` adds vault SQLCipher writes and a per-call
//!    fsync'd audit append on top of detection, none of which the 25ms *hot-path*
//!    detection budget covers. This gate only catches catastrophic regressions (an
//!    accidental O(n^2), a lost regex compilation cache); T10 owns precise p95 tracking.

use std::path::PathBuf;
use std::time::Instant;

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

/// One realistic mixed artefact (the shape `latency_gate.rs`'s reference block uses, as a
/// single unit): email, IP, phone, IBAN, sort code, IPv6, and a high-entropy secret.
const REFERENCE_ARTEFACT: &str = r#"
2026-07-15T10:22:31Z INFO  request from 10.0.0.42 user=jane.doe@example.com
  Authorization Bearer FAKEBENCHSECRET9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a
  customer phone +1-415-555-2671, iban GB29 NWBK 6016 1331 9268 19, sort code 12-34-56
  ipv6 peer 2001:0db8:85a3:0000:0000:8a2e:0370:7334
  the quick brown fox jumps over the lazy dog while nothing sensitive happens here
"#;

#[test]
fn full_mask_pipeline_p95_stays_within_a_generous_ci_safe_margin() {
    const BUDGET_MS: u128 = 25; // the interface contract's hot-path detection budget
    const CI_SLACK_FACTOR: u128 = 12; // loose on purpose -- adds vault + fsync'd audit I/O
    const ITERATIONS: usize = 100;

    let dir = TempDir::new().expect("temp dir");
    let engine = LayeredPolicyEngine::load(PolicyLayers {
        global: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    let ns = Namespace::Repo(RepoId("latency-gate".to_string()));

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

    // Warm up: first-call regex compilation and SQLCipher page setup shouldn't count
    // against the steady-state latency every subsequent call actually gets.
    for _ in 0..5 {
        let _ = mask(&input, &ctx, &policy, &ns).expect("mask succeeds");
    }

    let mut samples: Vec<u128> = (0..ITERATIONS)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(mask(&input, &ctx, &policy, &ns).expect("mask succeeds"));
            start.elapsed().as_millis()
        })
        .collect();
    samples.sort_unstable();

    let p95_index = (samples.len() as f64 * 0.95) as usize;
    let p95 = samples[p95_index.min(samples.len() - 1)];

    assert!(
        p95 <= BUDGET_MS * CI_SLACK_FACTOR,
        "full mask pipeline p95 regressed: {p95}ms over {ITERATIONS} iterations, \
         budget is {BUDGET_MS}ms x{CI_SLACK_FACTOR} CI-safety margin = {}ms. \
         Full samples (sorted): {samples:?}",
        BUDGET_MS * CI_SLACK_FACTOR
    );
}
