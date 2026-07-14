//! Placeholder criterion bench (Task T01 scaffold). Real hot-path benches
//! (p95 < 25ms budget per `docs/architecture/interface-contracts.md` §3) land
//! alongside each detector/parser crate in Tasks T03/T04/T08.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_placeholder(c: &mut Criterion) {
    c.bench_function("placeholder", |b| b.iter(|| black_box(1 + 1)));
}

criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
