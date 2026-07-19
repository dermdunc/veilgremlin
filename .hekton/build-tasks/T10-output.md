# T10 output — vg-bench eval harness (contract v1.3 → v1.4)

**Run:** run-20260718-T10 (Opus; died mid-response — failure mode #4 — rescued in place).
Written by the rescue session; narrative in `docs/decisions.md` (2026-07-18 T10 entry).

Built: `vg-bench` (corpus loader with labelled 11-sample seeded corpus, isolated harness
over temp vault/policy/audit, report with the six banked measurements, renderer), wired
`vg_core::benchmark` (contract v1.4: gained `ctx`), `vg bench` CLI (exit 0 Go / 1 NoGo).

First verdict: **NO-GO** — FP 16.7% (entropy 13.3%, phone 40% — the deferred Wave B
numbers), placeholder-consistency 66.7%, display-collision corruption 1/3 confirmed;
zero-raw-PII 11/11, recalls 100%, cold-hook p95 22.44 ms all pass. The harness failing
the product against its own gates is the deliverable working. T11 owns the resulting
decisions. Validation: 221 tests / 0 failures; clippy/fmt clean.
