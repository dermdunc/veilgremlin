//! `vg-parsers` — file-aware parsers implementing [`vg_core::Parser`].
//!
//! Must be robust to malformed input: return best-effort spans, never panic
//! (`docs/architecture/interface-contracts.md` §4). One module per format:
//! [`json`], [`yaml`], [`toml`], [`csv`], [`log`], [`diff`], [`env`], plus [`rust`] for
//! source code via tree-sitter. [`all_parsers`] is the composition point downstream code
//! (`vg-core::Context`, the CLI) enumerates rather than naming each parser individually.
//!
//! ## The never-panic contract, and how these modules keep it
//!
//! Every `parse` here is written to tolerate genuinely adversarial input — empty buffers,
//! truncated UTF-8, unbalanced delimiters, a "JSON" file that is actually binary. The
//! format modules that lean on hand-rolled byte scanners ([`json`], [`csv`], [`log`],
//! [`diff`], [`env`], plus the line-scan span extraction shared by [`yaml`]/[`toml`])
//! never index past a bounds check and route every span through [`util::span`], which
//! clamps to the buffer length. The two modules that call a third-party parser
//! ([`yaml`] via `serde_yaml`, [`toml`] via the `toml` crate, [`rust`] via tree-sitter)
//! only ever use error-returning entry points (`Result`/`Option`), never a panicking one,
//! and fall back to best-effort spans on error. The `parser_never_panics_*` tests in each
//! module drive `vg_core::conformance::assert_parser_never_panics` against a real battery
//! of adversarial buffers, not one happy-path fixture.
//!
//! Scaffolded in Task T01; parser implementations landed in Task T08.

pub mod csv;
pub mod diff;
pub mod env;
pub mod json;
pub mod log;
pub mod rust;
pub mod toml;
pub mod yaml;

mod util;

use vg_core::Parser;

/// All format parsers this crate provides, as trait objects. Order is significant only in
/// that `can_parse` is meant to be checked in sequence by a caller resolving an artefact
/// to a parser: the more specific hints (a `.env` filename, a `.log` extension) sit
/// alongside the structural ones, and a caller takes the first `can_parse` match.
pub fn all_parsers() -> Vec<Box<dyn Parser>> {
    vec![
        Box::new(json::JsonParser),
        Box::new(yaml::YamlParser),
        Box::new(toml::TomlParser),
        Box::new(csv::CsvParser),
        Box::new(env::EnvParser),
        Box::new(log::LogParser),
        Box::new(diff::DiffParser),
        Box::new(rust::RustParser),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_parsers_returns_every_format() {
        assert_eq!(all_parsers().len(), 8);
    }

    #[test]
    fn every_parser_is_panic_safe_on_a_shared_adversarial_battery() {
        // The buffers no single format "owns" but every parser must survive. Each module
        // additionally runs a format-specific battery; this is the belt-and-braces sweep
        // that guarantees no parser in the registry panics on the nastiest shared inputs.
        let battery: Vec<Vec<u8>> = vec![
            vec![],                                   // empty
            vec![0xFF, 0xFE, 0x00, 0x80],             // invalid UTF-8
            vec![0x00; 4096],                         // all NULs
            vec![b'{'; 10_000],                       // deeply unbalanced
            vec![b'"'; 8192],                         // a wall of unterminated quotes
            (0u8..=255).cycle().take(9000).collect(), // every byte value
            b"\xed\xa0\x80".to_vec(),                 // a lone UTF-16 surrogate in UTF-8 form
            b"key: value\n\xff\xfe truncated".to_vec(),
        ];
        for parser in all_parsers() {
            for buf in &battery {
                vg_core::conformance::assert_parser_never_panics(parser.as_ref(), buf);
            }
        }
    }
}
