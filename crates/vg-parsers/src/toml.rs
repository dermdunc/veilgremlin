//! TOML parser: the `toml` crate for well-formedness, a tolerant line scanner for spans.
//!
//! Same shape as [`crate::yaml`]: `toml::from_str` parses into a `Value` with no byte
//! offsets, so it cannot locate a key in the buffer, but it is exercised on every parse
//! (including the adversarial battery) as the well-formedness gate. Spans — `[table]`
//! headers, `key = value` keys and values, `#` comments — come from a quote-aware line
//! scanner that never indexes out of bounds and treats an unterminated string as running
//! to end-of-line.

use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{
    hint_extension, hint_file_name, language_is, lines_with_offsets, mime_contains, span,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct TomlParser;

impl Parser for TomlParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "toml")
            || hint_file_name(artefact).is_some_and(|n| n == "cargo.lock")
            || language_is(artefact, &["toml"])
            || mime_contains(artefact, &["toml"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        // Exercise the toml crate as the well-formedness gate. Requires UTF-8; non-UTF-8
        // input is simply "not well-formed" and we proceed with best-effort line spans.
        // `::toml` (leading `::`) is the external crate, disambiguated from this module,
        // which is also named `toml`.
        let _well_formed = std::str::from_utf8(buf)
            .ok()
            .and_then(|s| ::toml::from_str::<::toml::Table>(s).ok())
            .is_some();

        ParseResult {
            spans: scan_lines(buf),
            artefact_kind: ArtefactKind::Toml,
        }
    }
}

fn trimmed_range(slice: &[u8]) -> Option<(usize, usize)> {
    let start = slice.iter().position(|b| !b.is_ascii_whitespace())?;
    let end = slice.iter().rposition(|b| !b.is_ascii_whitespace())? + 1;
    Some((start, end))
}

/// Quote-aware scan of a line, returning `(comment_start, eq_pos)`:
/// - `comment_start`: index of the first `#` not inside a string, if any.
/// - `eq_pos`: index of the first `=` not inside a string and before any comment, if any.
///
/// Tracks basic (`"`) and literal (`'`) strings; honours `\` escapes inside basic strings
/// only (TOML literal strings have no escapes). Never reads past the slice end.
fn scan_line(line: &[u8]) -> (Option<usize>, Option<usize>) {
    let mut comment = None;
    let mut eq = None;
    let mut i = 0;
    let mut in_basic = false;
    let mut in_literal = false;
    while i < line.len() {
        let b = line[i];
        if in_basic {
            match b {
                b'\\' => i += 1, // skip escaped byte (bounds-checked by the while)
                b'"' => in_basic = false,
                _ => {}
            }
        } else if in_literal {
            if b == b'\'' {
                in_literal = false;
            }
        } else {
            match b {
                b'"' => in_basic = true,
                b'\'' => in_literal = true,
                b'#' => {
                    comment = Some(i);
                    break;
                }
                b'=' if eq.is_none() => eq = Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    (comment, eq)
}

fn scan_lines(buf: &[u8]) -> Vec<Span> {
    let len = buf.len();
    let mut spans = Vec::new();

    for (offset, line) in lines_with_offsets(buf) {
        let (comment, eq) = scan_line(line);
        let content_end = comment.unwrap_or(line.len());
        if let Some(c) = comment {
            spans.push(span(
                offset + c,
                offset + line.len(),
                NodeKind::Comment,
                len,
            ));
        }
        let content = &line[..content_end];

        let Some((cs, ce)) = trimmed_range(content) else {
            continue;
        };
        let trimmed = &content[cs..ce];

        // `[table]` or `[[array.of.tables]]` header: tag the whole bracketed name a Key.
        if trimmed.first() == Some(&b'[') {
            spans.push(span(offset + cs, offset + ce, NodeKind::Key, len));
            continue;
        }

        // `key = value` — only when the `=` sits within the (pre-comment) content.
        match eq.filter(|&e| e < content_end) {
            Some(e) if e > cs => {
                push_trimmed(&mut spans, content, cs, e, offset, NodeKind::Key, len);
                if e + 1 < ce {
                    push_trimmed(&mut spans, content, e + 1, ce, offset, NodeKind::Value, len);
                }
            }
            _ => {
                // A bare fragment (e.g. an array element on its own line inside a
                // multi-line array). Best-effort Value.
                push_trimmed(&mut spans, content, cs, ce, offset, NodeKind::Value, len);
            }
        }
    }

    spans
}

fn push_trimmed(
    spans: &mut Vec<Span>,
    content: &[u8],
    rel_start: usize,
    rel_end: usize,
    offset: usize,
    kind: NodeKind,
    len: usize,
) {
    let slice = &content[rel_start..rel_end.min(content.len())];
    if let Some((ts, te)) = trimmed_range(slice) {
        spans.push(span(
            offset + rel_start + ts,
            offset + rel_start + te,
            kind,
            len,
        ));
    }
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
    fn extracts_tables_keys_values_and_comments() {
        let buf = b"# top comment\n[database]\nhost = \"db.internal\"\nport = 5432 # inline\n";
        let result = TomlParser.parse(buf);
        assert_eq!(result.artefact_kind, ArtefactKind::Toml);

        let keys = of_kind(buf, &result.spans, NodeKind::Key);
        assert!(keys.contains(&"[database]"), "keys: {keys:?}");
        assert!(keys.contains(&"host"), "keys: {keys:?}");
        assert!(keys.contains(&"port"), "keys: {keys:?}");

        let values = of_kind(buf, &result.spans, NodeKind::Value);
        assert!(values.contains(&"\"db.internal\""), "values: {values:?}");

        let comments = of_kind(buf, &result.spans, NodeKind::Comment);
        assert!(comments.iter().any(|c| c.contains("top comment")));
        assert!(comments.iter().any(|c| c.contains("inline")));
    }

    #[test]
    fn does_not_treat_a_hash_inside_a_string_as_a_comment() {
        let buf = b"url = \"http://example.com/#frag\"\n";
        let result = TomlParser.parse(buf);
        assert!(
            of_kind(buf, &result.spans, NodeKind::Comment).is_empty(),
            "a # inside a quoted value must not start a comment"
        );
        let values = of_kind(buf, &result.spans, NodeKind::Value);
        assert!(
            values.contains(&"\"http://example.com/#frag\""),
            "values: {values:?}"
        );
    }

    #[test]
    fn every_span_is_within_bounds() {
        let buf = b"[a.b]\nx = 1\ny = [\n  1,\n  2,\n]\nz = 'literal'\n";
        for s in TomlParser.parse(buf).spans {
            assert!(s.start <= s.end && s.end <= buf.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_toml() {
        let cases: &[&[u8]] = &[
            b"",
            b"=",
            b"[",
            b"]]]]",
            b"key = \"unterminated",
            b"key = 'unterminated",
            b"# just a comment",
            b"=====",
            b"[[[[",
            b"key = \"trailing escape \\",
            &[0xFF, 0xFE, b'=', 0x00],
            b"a = 1\r\n\xff\xfe",
        ];
        for &c in cases {
            assert_parser_never_panics(&TomlParser, c);
        }
        assert_parser_never_panics(&TomlParser, &[b'='; 5000]);
        assert_parser_never_panics(&TomlParser, &[b'"'; 5000]);
    }
}
