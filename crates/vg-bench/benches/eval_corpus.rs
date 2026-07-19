//! Criterion hot-path bench: scan the whole labelled seeded corpus through the real
//! detector/parser registries (Task T10). Complements the report harness's wall-clock
//! measurements with a stable microbenchmark of detection throughput over realistic mixed
//! artefacts, and gives the workspace bench job something real to compile and run.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use vg_bench::LabelledCorpus;
use vg_core::{scan, Context, Detector, Parser};

fn bench_corpus_scan(c: &mut Criterion) {
    let corpus = LabelledCorpus::embedded().expect("embedded corpus loads");
    let detectors = vg_detectors::all_detectors();
    let parsers = vg_parsers::all_parsers();
    let det_refs: Vec<&dyn Detector> = detectors.iter().map(|d| d.as_ref()).collect();
    let par_refs: Vec<&dyn Parser> = parsers.iter().map(|p| p.as_ref()).collect();
    let ctx = Context {
        parsers: &par_refs,
        detectors: &det_refs,
    };

    c.bench_function("scan_seeded_corpus", |b| {
        b.iter(|| {
            for sample in &corpus.samples {
                black_box(scan(&sample.input, &ctx));
            }
        })
    });
}

criterion_group!(benches, bench_corpus_scan);
criterion_main!(benches);
