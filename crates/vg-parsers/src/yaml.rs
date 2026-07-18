//! YAML parser: `serde_yaml` for well-formedness, a tolerant line scanner for spans.
//!
//! `serde_yaml` (like `serde_json`/`toml`'s value trees) parses into an owned `Value`
//! with **no byte offsets**, so it cannot on its own answer "where in the buffer is this
//! key". It is still load-bearing here in two ways: (1) it is exercised on every parse,
//! including the adversarial never-panic battery, so the third-party parser's panic-safety
//! is verified alongside ours; and (2) when a document is *valid* YAML but written in
//! flow style (`{a: 1, b: 2}` / `[1, 2]` — YAML being a JSON superset), the block-oriented
//! line scanner finds no `key:` structure, and we fall back to the JSON tokenizer for
//! spans. Block-style YAML (the common case) gets its keys/values/comments from the line
//! scanner directly. Malformed input never panics — `serde_yaml` returns `Err`, the line
//! scanner returns whatever it found.

use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{hint_extension, language_is, lines_with_offsets, mime_contains, span};

#[derive(Debug, Default, Clone, Copy)]
pub struct YamlParser;

impl Parser for YamlParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "yaml" || e == "yml")
            || language_is(artefact, &["yaml", "yml"])
            || mime_contains(artefact, &["yaml", "x-yaml"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        // Exercise serde_yaml as the well-formedness gate. We do not unwrap: malformed
        // YAML yields Err and we simply proceed with best-effort line spans.
        let well_formed = serde_yaml::from_slice::<serde_yaml::Value>(buf).is_ok();

        let mut spans = scan_lines(buf);
        if spans.is_empty() && well_formed {
            // Valid YAML with no block-style `key:` structure — most likely flow style,
            // which is JSON-shaped. Reuse the JSON tokenizer.
            spans = crate::json::scan(buf);
        }

        ParseResult {
            spans,
            artefact_kind: ArtefactKind::Yaml,
        }
    }
}

/// Relative `[start, end)` of `slice` with leading/trailing ASCII whitespace removed.
/// Returns `None` if the slice is entirely whitespace.
fn trimmed_range(slice: &[u8]) -> Option<(usize, usize)> {
    let start = slice.iter().position(|b| !b.is_ascii_whitespace())?;
    let end = slice.iter().rposition(|b| !b.is_ascii_whitespace())? + 1;
    Some((start, end))
}

/// Position of the first `#` that begins a comment: at line start, or preceded by
/// whitespace. A `#` glued to non-whitespace (e.g. inside `http://x#frag` or an unquoted
/// value) is not treated as a comment start — best-effort, matching YAML's own rule
/// closely enough for span purposes.
fn comment_start(line: &[u8]) -> Option<usize> {
    for (i, &b) in line.iter().enumerate() {
        if b == b'#' && (i == 0 || line[i - 1].is_ascii_whitespace()) {
            return Some(i);
        }
    }
    None
}

fn scan_lines(buf: &[u8]) -> Vec<Span> {
    let len = buf.len();
    let mut spans = Vec::new();

    for (offset, line) in lines_with_offsets(buf) {
        // Split off any trailing comment first, and emit a span for it.
        let content_end = match comment_start(line) {
            Some(c) => {
                spans.push(span(
                    offset + c,
                    offset + line.len(),
                    NodeKind::Comment,
                    len,
                ));
                c
            }
            None => line.len(),
        };
        let content = &line[..content_end];

        let Some((cs, ce)) = trimmed_range(content) else {
            continue; // blank or comment-only line
        };
        let trimmed = &content[cs..ce];

        // A document/directive marker line (`---`, `...`, `%YAML`) has no key/value.
        if trimmed == b"---" || trimmed == b"..." || trimmed.first() == Some(&b'%') {
            continue;
        }

        // Strip a leading list marker `- ` (possibly repeated: `- - item`).
        let mut item_start = cs;
        loop {
            let rest = &content[item_start..ce];
            if rest.first() == Some(&b'-')
                && rest
                    .get(1)
                    .map(|b| b.is_ascii_whitespace())
                    .unwrap_or(false)
            {
                item_start += 2;
                while item_start < ce && content[item_start].is_ascii_whitespace() {
                    item_start += 1;
                }
            } else if rest == b"-" {
                // A bare `-` introducing a nested block on following lines.
                item_start = ce;
                break;
            } else {
                break;
            }
        }
        if item_start >= ce {
            continue;
        }

        // Look for a `key:` separator: a colon followed by whitespace or end-of-content.
        let seg = &content[item_start..ce];

        // A segment that opens with a flow-style indicator (`{` or `[`) is JSON-shaped, not
        // a block `key:`. Leave it for the JSON-tokenizer fallback rather than slicing
        // `{"host"` off as a bogus "key" (verify + review finding, 2026-07-17: the block
        // scanner matched the first `:` in `{"host": ...}` and emitted `{"host"` as a key,
        // so `spans` was never empty and the flow-style fallback never fired). A line like
        // `config: {host: db}` opens with `config`, not the indicator, so it is unaffected.
        if matches!(seg.first(), Some(b'{') | Some(b'[')) {
            continue;
        }

        let colon = find_key_colon(seg);
        match colon {
            Some(rel_colon) => {
                let key_abs_start = offset + item_start;
                let key_abs_end = offset + item_start + rel_colon;
                if key_abs_end > key_abs_start {
                    spans.push(span_from_trim(
                        content,
                        item_start,
                        item_start + rel_colon,
                        offset,
                        NodeKind::Key,
                        len,
                    ));
                }
                // Value = everything after the colon, trimmed.
                let val_start = item_start + rel_colon + 1;
                if val_start < ce {
                    spans.push(span_from_trim(
                        content,
                        val_start,
                        ce,
                        offset,
                        NodeKind::Value,
                        len,
                    ));
                }
            }
            None => {
                // No key: a plain list-item scalar or a continuation value.
                spans.push(span_from_trim(
                    content,
                    item_start,
                    ce,
                    offset,
                    NodeKind::Value,
                    len,
                ));
            }
        }
    }

    spans
}

/// Emits a span over `content[rel_start..rel_end]` with interior whitespace trimmed,
/// translated to absolute offsets via `offset`. Skips emission (returns a zero-length
/// span that the caller still bounds-checks) only if the range is all whitespace.
fn span_from_trim(
    content: &[u8],
    rel_start: usize,
    rel_end: usize,
    offset: usize,
    kind: NodeKind,
    len: usize,
) -> Span {
    let slice = &content[rel_start..rel_end];
    match trimmed_range(slice) {
        Some((ts, te)) => span(offset + rel_start + ts, offset + rel_start + te, kind, len),
        None => span(offset + rel_start, offset + rel_start, kind, len),
    }
}

/// Finds the byte offset of the `key: value` separator colon within `seg`: the first `:`
/// that is followed by an ASCII whitespace byte or is the last byte of `seg`. Colons
/// embedded in a value (`time: 12:30` — the second colon) are not separators. Returns
/// `None` when there is no such colon (a bare scalar).
fn find_key_colon(seg: &[u8]) -> Option<usize> {
    for (i, &b) in seg.iter().enumerate() {
        if b == b':' {
            match seg.get(i + 1) {
                None => return Some(i),
                Some(n) if n.is_ascii_whitespace() => return Some(i),
                _ => {}
            }
        }
    }
    None
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

    #[test]
    fn extracts_block_keys_values_and_comments() {
        let buf = b"# config\nname: jane\nemail: jane@example.com  # primary\nport: 8080\n";
        let result = YamlParser.parse(buf);
        assert_eq!(result.artefact_kind, ArtefactKind::Yaml);

        let keys = of_kind(buf, &result.spans, NodeKind::Key);
        assert!(keys.contains(&"name"), "keys: {keys:?}");
        assert!(keys.contains(&"email"), "keys: {keys:?}");
        assert!(keys.contains(&"port"), "keys: {keys:?}");

        let values = of_kind(buf, &result.spans, NodeKind::Value);
        assert!(values.contains(&"jane@example.com"), "values: {values:?}");

        let comments = of_kind(buf, &result.spans, NodeKind::Comment);
        assert!(comments.iter().any(|c| c.contains("config")));
        assert!(comments.iter().any(|c| c.contains("primary")));
    }

    #[test]
    fn handles_list_items() {
        let buf = b"users:\n  - alice@example.com\n  - bob@example.com\n";
        let parsed = YamlParser.parse(buf);
        let values = of_kind(buf, &parsed.spans, NodeKind::Value);
        assert!(values.contains(&"alice@example.com"), "values: {values:?}");
        assert!(values.contains(&"bob@example.com"), "values: {values:?}");
    }

    #[test]
    fn does_not_split_a_time_value_at_its_inner_colon() {
        let buf = b"start: 12:30:00\n";
        let spans = YamlParser.parse(buf).spans;
        let values = of_kind(buf, &spans, NodeKind::Value);
        assert!(values.contains(&"12:30:00"), "values: {values:?}");
        let keys = of_kind(buf, &spans, NodeKind::Key);
        assert_eq!(keys, vec!["start"]);
    }

    #[test]
    fn falls_back_to_json_tokenizer_for_flow_style() {
        let buf = br#"{"host": "db.internal", "port": 5432}"#;
        let spans = YamlParser.parse(buf).spans;
        let keys = of_kind(buf, &spans, NodeKind::Key);
        assert!(keys.contains(&"host"), "keys: {keys:?}");
    }

    #[test]
    fn every_span_is_within_bounds() {
        let buf = b"a: 1\nb:\n  - x\n  - y\nc: # empty\n";
        for s in YamlParser.parse(buf).spans {
            assert!(s.start <= s.end && s.end <= buf.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_yaml() {
        let cases: &[&[u8]] = &[
            b"",
            b":",
            b"   ",
            b"-",
            b"- - - -",
            b"key:",
            b": value",
            b"a:\n b:\n  c:\n   d:",
            b"\t\t\t: broken tabs",
            &[0xFF, 0xFE, b':', b' ', 0x00],
            b"key: value\r\n\xff\xfe",
            b"#\n#\n#\n",
        ];
        for &c in cases {
            assert_parser_never_panics(&YamlParser, c);
        }
        assert_parser_never_panics(&YamlParser, &[b':'; 5000]);
        assert_parser_never_panics(&YamlParser, &[b'-'; 5000]);
    }
}
