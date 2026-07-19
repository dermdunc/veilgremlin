//! Redaction-safe plain-text rendering of a [`Report`]. Prints refs, counts, rates, sample
//! names, and placeholder-shaped decoys only — never a labelled or masked value.

use std::fmt::Write as _;

use crate::report::{Report, Verdict};

/// Render the full Go/No-Go report as text.
pub fn render(report: &Report) -> String {
    let mut o = String::new();
    let verdict = report.verdict();

    let _ = writeln!(o, "VeilGremlin — Go/No-Go eval report");
    let _ = writeln!(o, "==================================");
    let _ = writeln!(o, "corpus samples: {}", report.sample_count);
    let _ = writeln!(o, "verdict:        {}", verdict_label(verdict));
    let _ = writeln!(o);

    // ---- Go/No-Go gates ----
    let _ = writeln!(o, "Go/No-Go gates");
    let _ = writeln!(o, "--------------");
    for g in &report.gates {
        let mark = match g.passed {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "N/A ",
        };
        let _ = writeln!(
            o,
            "  [{mark}] {:<26} {:<28} {}",
            g.name, g.criterion, g.measured
        );
    }
    let _ = writeln!(o);

    // ---- Headline metrics (frozen API) ----
    let m = &report.metrics;
    let _ = writeln!(o, "Metrics (vg_core::benchmark)");
    let _ = writeln!(o, "----------------------------");
    let _ = writeln!(o, "  recall (effective):    {:.1}%", m.recall * 100.0);
    let _ = writeln!(o, "  precision:             {:.1}%", m.precision * 100.0);
    let _ = writeln!(
        o,
        "  false-positive rate:   {:.1}%",
        m.false_positive_rate * 100.0
    );
    let _ = writeln!(
        o,
        "  p95 latency:           {:.2} ms in-process detection{}",
        m.p95_latency_us as f64 / 1000.0,
        match &report.cold_hook {
            Some(ch) => format!(" · {:.2} ms cold hook e2e", ch.p95_us as f64 / 1000.0),
            None => String::new(),
        }
    );
    let _ = writeln!(
        o,
        "  caveats: small-N — FP/recall gates resolve in steps of ~1/findings; the gated \
         FP definition (type-equality) differs from the banked per-detector one \
         (overlap-any-label); see decisions.md T10."
    );
    let _ = writeln!(
        o,
        "  secret recall {:.1}% ({}/{}) · other-PII recall {:.1}% ({}/{})",
        rate(report.secret_recall.matched, report.secret_recall.total) * 100.0,
        report.secret_recall.matched,
        report.secret_recall.total,
        rate(report.pii_recall.matched, report.pii_recall.total) * 100.0,
        report.pii_recall.matched,
        report.pii_recall.total,
    );
    let _ = writeln!(o);

    // ---- Banked measurement 1: detector false positives ----
    let _ = writeln!(o, "1. Detector false-positive rates (banked from Wave B)");
    let _ = writeln!(
        o,
        "   (a finding overlapping no ground-truth label of any type)"
    );
    for d in &report.detector_fp {
        let flag = if d.detector == "entropy" || d.detector == "phone" {
            " <- banked"
        } else {
            ""
        };
        let _ = writeln!(
            o,
            "     {:<14} {:>3} FP / {:>3} findings = {:>5.1}% ({} on benign slice){}",
            d.detector,
            d.false_positives,
            d.total,
            d.rate() * 100.0,
            d.benign_slice_fp,
            flag
        );
    }
    let _ = writeln!(o);

    // ---- Banked measurement 2: zero-raw-PII property ----
    let z = &report.zero_raw_pii;
    let _ = writeln!(
        o,
        "2. Zero-raw-PII property (§1 invariant over every sample)"
    );
    let _ = writeln!(
        o,
        "     {}/{} samples pass; {} violation(s){}",
        z.checked - z.violations.len(),
        z.checked,
        z.violations.len(),
        if z.violations.is_empty() {
            String::new()
        } else {
            format!(": {}", z.violations.join(", "))
        }
    );
    let _ = writeln!(o);

    // ---- Banked measurement 3: display-collision incidence ----
    let corrupted = report.collisions.iter().filter(|c| c.corrupted).count();
    let _ = writeln!(o, "3. Display-collision incidence (mask→demask round-trip)");
    let _ = writeln!(
        o,
        "     {corrupted} of {} collision sample(s) corrupted",
        report.collisions.len()
    );
    for c in &report.collisions {
        let minted = if c.decoy_minted {
            " (decoy minted)"
        } else {
            ""
        };
        let _ = writeln!(
            o,
            "       {:<26} decoys [{}] -> {}{}",
            c.sample,
            c.decoys.join(", "),
            if c.corrupted { "CORRUPTED" } else { "clean" },
            minted
        );
    }
    if corrupted > 0 {
        let _ = writeln!(
            o,
            "     >0 on realistic slices — RECOMMEND T11: collision-avoiding minting (skip an"
        );
        let _ = writeln!(
            o,
            "     ordinal whose display already occurs in the raw buffer at intern time)."
        );
    }
    let _ = writeln!(o);

    // ---- Banked measurement 4: dotenv without a path hint ----
    let _ = writeln!(
        o,
        "4. Env-shaped content WITHOUT a file_path hint (entity detection only)"
    );
    for r in &report.dotenv_no_hint {
        let _ = writeln!(
            o,
            "     {:<26} caught {}/{} labelled; {} residual value(s) only an artefact Block would catch",
            r.sample, r.labelled_caught, r.labelled_total, r.residual
        );
    }
    let _ = writeln!(
        o,
        "     Residual risk: `echo X > .env` sets no path hint, so the artefact Block cannot"
    );
    let _ = writeln!(
        o,
        "     fire; short/structured secrets below the entropy floor pass through unmasked."
    );
    let _ = writeln!(o);

    // ---- Banked measurement 5: cold hook latency ----
    let _ = writeln!(o, "5. Cold `vg hook` end-to-end latency (binary level)");
    match &report.cold_hook {
        Some(ch) => {
            let _ = writeln!(
                o,
                "     over {} cold invocations: p50 {:.2} ms · p95 {:.2} ms · max {:.2} ms",
                ch.iterations,
                ch.p50_us as f64 / 1000.0,
                ch.p95_us as f64 / 1000.0,
                ch.max_us as f64 / 1000.0
            );
            let _ = writeln!(
                o,
                "     split: full path = process spawn + SQLCipher open + policy load + mask;"
            );
            let _ = writeln!(
                o,
                "     the in-process mask/detect portion alone is p95 {:.2} ms (measurement 0).",
                report.in_process_p95_us as f64 / 1000.0
            );
        }
        None => {
            let _ = writeln!(
                o,
                "     not measured (no `vg` binary supplied; run via `vg bench` for this number)."
            );
            let _ = writeln!(
                o,
                "     in-process mask/detect portion p95 {:.2} ms is measured regardless.",
                report.in_process_p95_us as f64 / 1000.0
            );
        }
    }
    let _ = writeln!(o);

    // ---- Banked measurement 6: dead policy branches ----
    let _ = writeln!(o, "6. Dead policy-branch detection");
    if report.dead_policy_branches.is_empty() {
        let _ = writeln!(o, "     none — every policy branch is reachable.");
    } else {
        for b in &report.dead_policy_branches {
            let _ = writeln!(
                o,
                "     DEAD {} [{}]: {}",
                b.selector,
                b.keys.join(", "),
                b.reason
            );
        }
    }
    let _ = writeln!(o);

    // ---- Structural guards ----
    let _ = writeln!(
        o,
        "7. Structural guards (mechanism fired, not just values found)"
    );
    for c in &report.structural {
        let _ = writeln!(
            o,
            "     {:<26} {:<26} {}",
            c.name,
            c.sample,
            if c.passed { "PASS" } else { "FAIL" }
        );
    }
    let _ = writeln!(o);

    let _ = writeln!(
        o,
        "Not evaluated by this harness (manual/CI gates): signed binary + SBOM +"
    );
    let _ = writeln!(
        o,
        "provenance + offline install; audit-pack quality; demo-patch reproduction."
    );

    o
}

fn rate(n: usize, d: usize) -> f64 {
    if d == 0 {
        1.0
    } else {
        n as f64 / d as f64
    }
}

fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Go => "GO",
        Verdict::NoGo => "NO-GO",
        Verdict::Incomplete => "INCOMPLETE (a gate could not be measured)",
    }
}
