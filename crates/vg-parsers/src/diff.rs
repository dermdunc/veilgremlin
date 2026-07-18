//! Git unified-diff parser.
//!
//! Recognises the line shapes of `git diff` output and tags each with structural context
//! a detector can use: file-header paths (`--- a/x`, `+++ b/x`) as
//! [`Field("path")`](NodeKind::Field), hunk headers (`@@ … @@`) as
//! [`Other("hunk")`](NodeKind::Other), and — the payload that matters for scanning —
//! added lines (`+…`) as [`Field("added")`](NodeKind::Field), removed lines (`-…`) as
//! [`Field("removed")`](NodeKind::Field), and context lines as [`Value`](NodeKind::Value),
//! each span covering the line's content *after* its one-character diff marker. Purely
//! line-based; malformed or non-diff input yields best-effort spans and never panics.

use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{hint_extension, language_is, lines_with_offsets, mime_contains, span};

#[derive(Debug, Default, Clone, Copy)]
pub struct DiffParser;

impl Parser for DiffParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "diff" || e == "patch")
            || language_is(artefact, &["diff", "patch"])
            || mime_contains(artefact, &["x-diff", "x-patch"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        let len = buf.len();
        let mut spans = Vec::new();

        for (offset, line) in lines_with_offsets(buf) {
            if line.is_empty() {
                continue;
            }
            // File headers: `--- a/path` / `+++ b/path`. Checked before the generic `-`/`+`
            // content rules, since these three-char markers would otherwise be misread as
            // removed/added content lines.
            if let Some(rest) = line
                .strip_prefix(b"--- ")
                .or_else(|| line.strip_prefix(b"+++ "))
            {
                let content_off = line.len() - rest.len();
                push_trimmed(
                    &mut spans,
                    rest,
                    offset + content_off,
                    NodeKind::Field("path".into()),
                    len,
                );
                continue;
            }
            if line.starts_with(b"@@") {
                spans.push(span(
                    offset,
                    offset + line.len(),
                    NodeKind::Other("hunk".into()),
                    len,
                ));
                continue;
            }
            if line.starts_with(b"diff --git")
                || line.starts_with(b"index ")
                || line.starts_with(b"new file")
                || line.starts_with(b"deleted file")
                || line.starts_with(b"rename ")
                || line.starts_with(b"similarity ")
            {
                // Metadata line; a Value span keeps its content scannable without a
                // dedicated structural kind.
                spans.push(span(offset, offset + line.len(), NodeKind::Value, len));
                continue;
            }
            match line[0] {
                b'+' => push_content(
                    &mut spans,
                    line,
                    offset,
                    NodeKind::Field("added".into()),
                    len,
                ),
                b'-' => push_content(
                    &mut spans,
                    line,
                    offset,
                    NodeKind::Field("removed".into()),
                    len,
                ),
                b' ' => push_content(&mut spans, line, offset, NodeKind::Value, len),
                _ => spans.push(span(offset, offset + line.len(), NodeKind::Value, len)),
            }
        }

        ParseResult {
            spans,
            artefact_kind: ArtefactKind::Diff,
        }
    }
}

/// Emits a span over a diff content line's payload — everything after the leading
/// one-character marker (`+`, `-`, or a space).
fn push_content(spans: &mut Vec<Span>, line: &[u8], offset: usize, kind: NodeKind, len: usize) {
    // Skip the single marker byte; the rest (possibly empty) is the content.
    let start = offset + 1;
    let end = offset + line.len();
    if end > start {
        spans.push(span(start, end, kind, len));
    }
}

/// Emits a span over `slice` with leading/trailing ASCII whitespace trimmed, at absolute
/// `base` offset. No emission if `slice` is all whitespace.
fn push_trimmed(spans: &mut Vec<Span>, slice: &[u8], base: usize, kind: NodeKind, len: usize) {
    let Some(start) = slice.iter().position(|b| !b.is_ascii_whitespace()) else {
        return;
    };
    let end = slice
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .unwrap()
        + 1;
    spans.push(span(base + start, base + end, kind, len));
}

#[cfg(test)]
mod tests {
    use super::*;
    use vg_core::conformance::assert_parser_never_panics;

    fn text<'a>(buf: &'a [u8], s: &Span) -> &'a str {
        std::str::from_utf8(&buf[s.start..s.end]).unwrap_or("<non-utf8>")
    }

    fn of_kind<'a>(buf: &'a [u8], spans: &'a [Span], kind: NodeKind) -> Vec<&'a str> {
        spans
            .iter()
            .filter(|s| s.node_kind == Some(kind.clone()))
            .map(|s| text(buf, s))
            .collect()
    }

    const SAMPLE: &[u8] = b"\
diff --git a/config.py b/config.py
index e69de29..4b825dc 100644
--- a/config.py
+++ b/config.py
@@ -1,3 +1,4 @@
 import os
-ADMIN = \"old@example.com\"
+ADMIN = \"jane@example.com\"
+API_KEY = \"sk-live-abc123\"
";

    #[test]
    fn classifies_paths_hunks_added_and_removed() {
        let result = DiffParser.parse(SAMPLE);
        assert_eq!(result.artefact_kind, ArtefactKind::Diff);

        let paths = of_kind(SAMPLE, &result.spans, NodeKind::Field("path".into()));
        assert!(paths.contains(&"a/config.py"), "paths: {paths:?}");
        assert!(paths.contains(&"b/config.py"), "paths: {paths:?}");

        let added = of_kind(SAMPLE, &result.spans, NodeKind::Field("added".into()));
        assert!(
            added.iter().any(|a| a.contains("jane@example.com")),
            "added: {added:?}"
        );
        assert!(
            added.iter().any(|a| a.contains("sk-live-abc123")),
            "added: {added:?}"
        );

        let removed = of_kind(SAMPLE, &result.spans, NodeKind::Field("removed".into()));
        assert!(
            removed.iter().any(|r| r.contains("old@example.com")),
            "removed: {removed:?}"
        );

        let hunks = of_kind(SAMPLE, &result.spans, NodeKind::Other("hunk".into()));
        assert_eq!(hunks.len(), 1);
    }

    #[test]
    fn every_span_is_within_bounds() {
        for s in DiffParser.parse(SAMPLE).spans {
            assert!(s.start <= s.end && s.end <= SAMPLE.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_diffs() {
        let cases: &[&[u8]] = &[
            b"",
            b"+",
            b"-",
            b"@@",
            b"--- ",
            b"+++ ",
            b"@@ -1 +1 malformed hunk with no closing",
            b"+++++++\n-------\n",
            &[b'+', 0xFF, 0xFE, 0x00],
            b"--- a/x\r\n+++ b/x\r\n\xff\xfe",
        ];
        for &c in cases {
            assert_parser_never_panics(&DiffParser, c);
        }
        assert_parser_never_panics(&DiffParser, &b"+line\n".repeat(2000));
    }
}
