//! Labelled seeded corpus: the on-disk manifest shape, the loader, and the in-memory
//! [`LabelledSample`]/[`LabelledCorpus`] the report layer scores.
//!
//! The manifest (`corpus/seeded/manifest.json`) declares each sample's raw `content` plus
//! ground-truth `expected` labels as `{ type, value }` pairs. The loader turns each label
//! into a `vg_core::Finding` by finding **every** occurrence of `value` in `content` and
//! recording its byte span — so labels are written by value, never by hand-counted offset.
//!
//! The corpus ships compiled into the binary via [`LabelledCorpus::embedded`], so `vg bench`
//! works with no corpus file on disk; [`LabelledCorpus::from_json`] loads an external one.

use serde::Deserialize;

use vg_core::{ArtefactHint, DetectorId, EntityType, Finding, Input, Span};

/// The committed seeded corpus, compiled in so the installed `vg` binary carries it.
const EMBEDDED_MANIFEST: &str = include_str!("../../../corpus/seeded/manifest.json");

/// A parsed, span-resolved corpus ready for the harness.
#[derive(Debug, Clone)]
pub struct LabelledCorpus {
    pub samples: Vec<LabelledSample>,
}

/// One corpus sample with its ground-truth findings resolved to spans, plus the metadata
/// the banked measurements select on.
#[derive(Debug, Clone)]
pub struct LabelledSample {
    pub name: String,
    pub description: String,
    /// The raw buffer + hint, exactly as the pipeline would receive it.
    pub input: Input,
    /// Ground-truth findings (type + span; `confidence`/`detector` are placeholders — the
    /// scorer compares type and span only).
    pub expected: Vec<Finding>,
    /// `(type, value)` for each label — the raw values the zero-raw-PII property checks are
    /// absent from a mask output (only for values the policy actually masks/redacts).
    pub expected_values: Vec<(EntityType, String)>,
    /// Slice tags selecting this sample into a banked measurement.
    pub slices: Vec<String>,
    /// Placeholder-shaped literals present in `content` that must survive a mask→demask
    /// round-trip (display-collision measurement).
    pub decoys: Vec<String>,
    /// Sensitive values entity detection is known not to catch — the dotenv-no-hint
    /// residual. Never scored in the global metrics.
    pub residual_secrets: Vec<String>,
}

impl LabelledSample {
    /// Whether this sample carries the given slice tag.
    pub fn in_slice(&self, slice: &str) -> bool {
        self.slices.iter().any(|s| s == slice)
    }

    /// The raw `content` as UTF-8 (the manifest is authored as text, so this never fails for
    /// the shipped corpus; a `?`-free helper keeps the round-trip measurements readable).
    pub fn content(&self) -> String {
        String::from_utf8_lossy(&self.input.buf).into_owned()
    }
}

impl LabelledCorpus {
    /// The seeded corpus compiled into the binary.
    pub fn embedded() -> Result<Self, CorpusError> {
        Self::from_json(EMBEDDED_MANIFEST)
    }

    /// Parse a manifest from JSON text and resolve every label to a span.
    pub fn from_json(json: &str) -> Result<Self, CorpusError> {
        let manifest: Manifest =
            serde_json::from_str(json).map_err(|e| CorpusError::Parse(e.to_string()))?;
        let mut samples = Vec::with_capacity(manifest.samples.len());
        for raw in manifest.samples {
            samples.push(raw.resolve()?);
        }
        Ok(Self { samples })
    }

    /// The `vg_core::Corpus` the frozen `benchmark` API scores.
    pub fn to_core_corpus(&self) -> vg_core::Corpus {
        vg_core::Corpus {
            samples: self
                .samples
                .iter()
                .map(|s| vg_core::CorpusSample {
                    input: s.input.clone(),
                    expected_findings: s.expected.clone(),
                })
                .collect(),
        }
    }

    /// Samples carrying `slice`.
    pub fn in_slice<'a>(&'a self, slice: &'a str) -> impl Iterator<Item = &'a LabelledSample> {
        self.samples.iter().filter(move |s| s.in_slice(slice))
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[allow(dead_code)]
    version: u32,
    samples: Vec<RawSample>,
}

#[derive(Debug, Deserialize)]
struct RawSample {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    artefact: Option<RawArtefact>,
    content: String,
    #[serde(default)]
    expected: Vec<RawLabel>,
    #[serde(default)]
    slices: Vec<String>,
    #[serde(default)]
    decoys: Vec<String>,
    #[serde(default)]
    residual_secrets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawArtefact {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    language_id: Option<String>,
    #[serde(default)]
    mime_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawLabel {
    #[serde(rename = "type")]
    entity_type: String,
    value: String,
}

impl RawSample {
    fn resolve(self) -> Result<LabelledSample, CorpusError> {
        let hint = self
            .artefact
            .map(RawArtefact::into_hint)
            .unwrap_or_default();
        let buf = self.content.clone().into_bytes();

        let mut expected = Vec::new();
        let mut expected_values = Vec::new();
        for label in &self.expected {
            let ty = parse_entity_type(&label.entity_type)
                .ok_or_else(|| CorpusError::UnknownEntityType(label.entity_type.clone()))?;
            let spans = find_all(self.content.as_bytes(), label.value.as_bytes());
            if spans.is_empty() {
                return Err(CorpusError::LabelNotFound {
                    sample: self.name.clone(),
                    value: label.value.clone(),
                });
            }
            for (start, end) in spans {
                expected.push(Finding {
                    entity_type: ty.clone(),
                    span: Span {
                        start,
                        end,
                        node_kind: None,
                    },
                    // Placeholder provenance: the scorer never reads these.
                    confidence: 1.0,
                    detector: DetectorId("corpus-label".to_string()),
                });
            }
            expected_values.push((ty, label.value.clone()));
        }

        Ok(LabelledSample {
            name: self.name,
            description: self.description,
            input: Input { buf, hint },
            expected,
            expected_values,
            slices: self.slices,
            decoys: self.decoys,
            residual_secrets: self.residual_secrets,
        })
    }
}

impl RawArtefact {
    fn into_hint(self) -> ArtefactHint {
        ArtefactHint {
            path: self.path.map(Into::into),
            language_id: self.language_id,
            mime_type: self.mime_type,
        }
    }
}

/// Every non-overlapping occurrence of `needle` in `haystack`, as `(start, end)` byte
/// ranges. Overlap between two *different* labels is fine; occurrences of one label do not
/// overlap each other (advance past each match).
fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut from = 0;
    while from + needle.len() <= haystack.len() {
        if &haystack[from..from + needle.len()] == needle {
            out.push((from, from + needle.len()));
            from += needle.len();
        } else {
            from += 1;
        }
    }
    out
}

/// Maps a manifest `type` string to an `EntityType`. Only the types a Phase-1 detector can
/// emit are accepted — a label for a type nothing detects would score a guaranteed miss and
/// is a corpus-authoring error, caught here at load.
fn parse_entity_type(s: &str) -> Option<EntityType> {
    match s {
        "Email" => Some(EntityType::Email),
        "Phone" => Some(EntityType::Phone),
        "InternalIp" => Some(EntityType::InternalIp),
        "Iban" => Some(EntityType::Iban),
        "SortCode" => Some(EntityType::SortCode),
        "Secret" => Some(EntityType::Secret),
        _ => None,
    }
}

/// A corpus load failure, named so a reviewer can fix the manifest directly.
#[derive(Debug)]
pub enum CorpusError {
    Parse(String),
    UnknownEntityType(String),
    LabelNotFound { sample: String, value: String },
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorpusError::Parse(e) => write!(f, "corpus manifest parse error: {e}"),
            CorpusError::UnknownEntityType(t) => write!(
                f,
                "corpus label type {t:?} is not a Phase-1 detectable type \
                 (Email|Phone|InternalIp|Iban|SortCode|Secret)"
            ),
            CorpusError::LabelNotFound { sample, value } => write!(
                f,
                "corpus sample {sample:?} labels a value that does not occur in its content \
                 (label value withheld to keep errors redaction-safe; len={})",
                value.len()
            ),
        }
    }
}

impl std::error::Error for CorpusError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_corpus_loads_and_resolves_every_label() {
        let corpus = LabelledCorpus::embedded().expect("embedded corpus loads");
        assert!(!corpus.samples.is_empty(), "corpus must not be empty");
        // Every labelled value resolved to at least one span (else `resolve` would error).
        for sample in &corpus.samples {
            assert!(
                sample.expected.len() >= sample.expected_values.len(),
                "sample {} lost labels during span resolution",
                sample.name
            );
        }
    }

    #[test]
    fn find_all_finds_repeated_occurrences_without_overlap() {
        assert_eq!(find_all(b"aXaXa", b"aXa"), vec![(0, 3)]);
        assert_eq!(find_all(b"abcabc", b"abc"), vec![(0, 3), (3, 6)]);
        assert_eq!(find_all(b"abc", b""), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn expected_slices_are_present_in_the_corpus() {
        let corpus = LabelledCorpus::embedded().expect("load");
        for slice in [
            "multi-entity",
            "yaml-underspan",
            "dotfile",
            "dotenv-no-hint",
            "json-payload",
            "benign-lookalike",
            "display-collision",
        ] {
            assert!(
                corpus.in_slice(slice).next().is_some(),
                "corpus is missing a sample for slice {slice:?}"
            );
        }
    }
}
