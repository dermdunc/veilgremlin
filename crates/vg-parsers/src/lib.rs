//! `vg-parsers` — file-aware parsers implementing `vg_core::Parser`.
//!
//! Must be robust to malformed input: return best-effort spans, never panic
//! (`docs/architecture/interface-contracts.md` §4). Code parsing uses tree-sitter;
//! format parsers cover json/yaml/toml/sql/csv/log/diff/env.
//!
//! Scaffolded in Task T01; parser implementations land in Task T08.
