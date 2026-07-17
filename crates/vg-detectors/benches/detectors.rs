//! Criterion bench for the five T03 detectors against a synthetic reference buffer,
//! checking the interface contract's p95 < 25ms hot-path budget
//! (`docs/architecture/interface-contracts.md` §3). CI only compile-checks this
//! (`cargo bench --workspace --locked --no-run`); running it for real (precise,
//! statistically-sound measurement) is a manual/local step until Task T10 wires up real
//! baseline management.
//!
//! `tests/latency_gate.rs` is the coarse, CI-enforced companion to this bench: a plain
//! `#[test]` (so it runs on every PR via the existing `cargo test` CI job, no new CI
//! config needed) that fails loudly on a gross regression, without needing this
//! criterion bench to be run by a human first.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

fn bench_all_detectors(c: &mut Criterion) {
    let buf = reference_buffer();
    let detectors = all_detectors();
    c.bench_function("all_detectors/reference_buffer", |b| {
        b.iter(|| {
            for detector in &detectors {
                black_box(detector.detect(black_box(&buf), &[]));
            }
        })
    });
}

criterion_group!(benches, bench_all_detectors);
criterion_main!(benches);
