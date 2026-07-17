//! Read-only "detector census": run vg-detectors against real Hekton artifacts to find
//! edge cases and measure real-world latency ahead of Task T07's pipeline wiring and
//! Task T10's formal eval harness. Dogfooding plan: see `docs/decisions.md` (2026-07-16).
//!
//! **Never prints or stores matched values** -- only counts, span lengths, detector
//! IDs, entity types, file paths, and latency. This is bug-fuel/evidence gathering, not
//! a masking-correctness test: `scan()`/`mask()` are still `todo!()` until T07, so this
//! cannot validate placeholder stability, vault safety, or policy behavior -- only
//! whether the five real detectors behave sanely against real agent-factory traffic.
//!
//! Usage: `cargo run --example census -- <root-dir> [<root-dir> ...]`

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use vg_detectors::all_detectors;

const SKIP_DIRS: &[&str] = &[
    "target",
    ".git",
    "node_modules",
    ".worktrees",
    "mind-palace",
    ".control-tower",
];
const SCAN_EXTENSIONS: &[&str] = &["md", "yaml", "yml", "toml", "log", "txt"];

#[derive(Default)]
struct DetectorStats {
    finding_count: usize,
    span_len_total: usize,
    span_len_max: usize,
}

fn walk(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if SKIP_DIRS.contains(&name) {
                    continue;
                }
            }
            walk(&path, files);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if SCAN_EXTENSIONS.contains(&ext) {
                files.push(path);
            }
        }
    }
}

fn main() {
    let roots: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if roots.is_empty() {
        eprintln!("usage: census <root-dir> [<root-dir> ...]");
        std::process::exit(1);
    }

    let mut files = Vec::new();
    for root in &roots {
        walk(root, &mut files);
    }
    files.sort();

    let detectors = all_detectors();
    let mut per_detector: BTreeMap<String, DetectorStats> = BTreeMap::new();
    let mut per_entity_type: BTreeMap<String, usize> = BTreeMap::new();
    // (path, finding_count, micros) -- deliberately no matched text captured anywhere.
    let mut file_finding_counts: Vec<(PathBuf, usize, u128)> = Vec::new();
    let mut total_bytes = 0usize;
    let mut total_micros: u128 = 0;
    let mut unreadable = 0usize;

    for path in &files {
        let buf = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => {
                unreadable += 1;
                continue;
            }
        };
        total_bytes += buf.len();

        let start = Instant::now();
        let mut file_finding_count = 0;
        for detector in &detectors {
            let findings = detector.detect(&buf, &[]);
            file_finding_count += findings.len();
            let stats = per_detector.entry(detector.id().0.clone()).or_default();
            for f in &findings {
                stats.finding_count += 1;
                let len = f.span.end.saturating_sub(f.span.start);
                stats.span_len_total += len;
                stats.span_len_max = stats.span_len_max.max(len);
                *per_entity_type
                    .entry(format!("{:?}", f.entity_type))
                    .or_default() += 1;
            }
        }
        let elapsed = start.elapsed().as_micros();
        total_micros += elapsed;
        file_finding_counts.push((path.clone(), file_finding_count, elapsed));
    }

    file_finding_counts.sort_by_key(|entry| std::cmp::Reverse(entry.1));

    println!("=== VeilGremlin Detector Census ===");
    println!("Roots scanned: {roots:?}");
    println!(
        "Files scanned: {} ({total_bytes} bytes total, {unreadable} unreadable/skipped)",
        files.len()
    );
    println!();
    println!("--- Per-detector totals ---");
    for (id, stats) in &per_detector {
        let avg = stats
            .span_len_total
            .checked_div(stats.finding_count)
            .unwrap_or(0);
        println!(
            "  {id:<16} findings={:<6} avg_span_len={avg:<4} max_span_len={}",
            stats.finding_count, stats.span_len_max
        );
    }
    println!();
    println!("--- Per-entity-type totals ---");
    for (ty, count) in &per_entity_type {
        println!("  {ty:<16} {count}");
    }
    println!();
    let file_count = files.len().max(1) as f64;
    println!("--- Latency ---");
    println!(
        "  total: {:.2}ms across {} files",
        total_micros as f64 / 1000.0,
        files.len()
    );
    println!(
        "  avg per file: {:.3}ms",
        (total_micros as f64 / 1000.0) / file_count
    );
    println!();
    println!(
        "--- Top 15 files by finding count (path + count + latency only, no matched values) ---"
    );
    for (path, count, micros) in file_finding_counts.iter().take(15) {
        println!(
            "  {:<70} findings={count:<5} {:.3}ms",
            path.display(),
            *micros as f64 / 1000.0
        );
    }
}
