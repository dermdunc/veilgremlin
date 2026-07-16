//! A real, CI-enforced latency regression gate for the hot path — not the criterion
//! bench (`benches/detectors.rs`), which is compile-checked only in CI today and needs
//! a human to run it locally to see a number. This is a plain `#[test]`, so it runs
//! wherever `cargo test` already runs (every PR, per T01's CI), with zero CI config
//! changes needed.
//!
//! Deliberately coarse, not a tight perf gate: shared CI runners are noisy (thermal
//! throttling, neighbor contention), so a tight bound here would be a flaky test, not a
//! useful one. This asserts against 4x the interface contract's p95 < 25ms budget —
//! loose enough to never false-positive on CI jitter, tight enough to catch an actual
//! regression (an accidentally-uncompiled regex, a hot-path allocation, an O(n^2)
//! detector) before it lands, rather than only at Task T10's eventual eval harness with
//! real baseline management. T10 owns the *precise* p95/p99 tracking with a real
//! baseline; this only owns "did something get catastrophically slower."
use std::time::Instant;

use vg_detectors::all_detectors;

const TEMPLATE_BLOCK: &str = r#"
2026-07-15T10:22:31Z INFO  request from 10.0.0.42 user=jane.doe@example.com
  Authorization: Bearer FAKE-BENCH-SECRET-9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8c7d6e
  customer phone +1-415-555-2671, iban GB29 NWBK 6016 1331 9268 19, sort code 12-34-56
  ipv6 peer 2001:0db8:85a3:0000:0000:8a2e:0370:7334
  the quick brown fox jumps over the lazy dog while nothing sensitive happens here
"#;

fn reference_buffer() -> Vec<u8> {
    TEMPLATE_BLOCK.repeat(200).into_bytes()
}

#[test]
fn all_detectors_p95_stays_within_a_generous_ci_safe_margin_of_budget() {
    const BUDGET_MS: u128 = 25;
    const CI_SLACK_FACTOR: u128 = 4; // loose on purpose -- see module docs
    const ITERATIONS: usize = 200;

    let buf = reference_buffer();
    let detectors = all_detectors();

    // Warm up: first-call OnceLock regex compilation shouldn't count against the
    // hot-path budget every subsequent call actually gets.
    for detector in &detectors {
        let _ = detector.detect(&buf, &[]);
    }

    let mut samples: Vec<u128> = (0..ITERATIONS)
        .map(|_| {
            let start = Instant::now();
            for detector in &detectors {
                std::hint::black_box(detector.detect(std::hint::black_box(&buf), &[]));
            }
            start.elapsed().as_millis()
        })
        .collect();
    samples.sort_unstable();

    let p95_index = (samples.len() as f64 * 0.95) as usize;
    let p95 = samples[p95_index.min(samples.len() - 1)];

    assert!(
        p95 <= BUDGET_MS * CI_SLACK_FACTOR,
        "all_detectors p95 latency regressed: {p95}ms over {ITERATIONS} iterations, \
         budget is {BUDGET_MS}ms x{CI_SLACK_FACTOR} CI-safety margin = {}ms. \
         Full samples (sorted): {samples:?}",
        BUDGET_MS * CI_SLACK_FACTOR
    );
}
