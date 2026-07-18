//! `.env` / dotenv file parser.
//!
//! Each line is `KEY=VALUE` (optionally `export KEY=VALUE`), a `#` comment, or blank. The
//! key name becomes a [`Key`](NodeKind::Key) span, the value a [`Value`](NodeKind::Value)
//! span (with one pair of wrapping quotes stripped), and a whole-line comment a
//! [`Comment`](NodeKind::Comment) span. Values are where secrets live, so getting a clean
//! value span is the point. Inline `#` after a value is intentionally *not* treated as a
//! comment: dotenv tools disagree on that, and a value like `pass#word` or a URL fragment
//! must not be truncated. Malformed lines (`=`, no `=`, unterminated quote) degrade to
//! best-effort spans and never panic.

use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{hint_extension, hint_file_name, language_is, lines_with_offsets, span};

#[derive(Debug, Default, Clone, Copy)]
pub struct EnvParser;

impl Parser for EnvParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        // `.env`, `.env.local`, `.env.production`, etc. have no extension in the
        // `Path::extension` sense (the whole name is ".env…"), so match on file name.
        hint_file_name(artefact).is_some_and(|n| n == ".env" || n.starts_with(".env."))
            || hint_extension(artefact).is_some_and(|e| e == "env")
            || language_is(artefact, &["dotenv", "env"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        let len = buf.len();
        let mut spans = Vec::new();

        for (offset, line) in lines_with_offsets(buf) {
            let Some((cs, ce)) = trimmed_range(line) else {
                continue; // blank
            };
            let trimmed = &line[cs..ce];

            if trimmed.first() == Some(&b'#') {
                spans.push(span(offset + cs, offset + ce, NodeKind::Comment, len));
                continue;
            }

            // Strip an optional `export ` prefix from the key side.
            let mut key_start = cs;
            if trimmed.starts_with(b"export ") {
                key_start = cs + "export ".len();
                while key_start < ce && line[key_start].is_ascii_whitespace() {
                    key_start += 1;
                }
            }

            // Split on the first `=` within the line's content.
            match line[key_start..ce].iter().position(|&b| b == b'=') {
                Some(rel_eq) => {
                    let eq = key_start + rel_eq;
                    push_trimmed(&mut spans, line, key_start, eq, offset, NodeKind::Key, len);
                    if eq < ce {
                        push_value(&mut spans, line, eq + 1, ce, offset, len);
                    }
                }
                None => {
                    // A bare token with no `=` — best-effort Key so it's still visible.
                    push_trimmed(&mut spans, line, key_start, ce, offset, NodeKind::Key, len);
                }
            }
        }

        ParseResult {
            spans,
            artefact_kind: ArtefactKind::EnvFile,
        }
    }
}

fn trimmed_range(slice: &[u8]) -> Option<(usize, usize)> {
    let start = slice.iter().position(|b| !b.is_ascii_whitespace())?;
    let end = slice.iter().rposition(|b| !b.is_ascii_whitespace())? + 1;
    Some((start, end))
}

fn push_trimmed(
    spans: &mut Vec<Span>,
    line: &[u8],
    rel_start: usize,
    rel_end: usize,
    offset: usize,
    kind: NodeKind,
    len: usize,
) {
    let slice = &line[rel_start..rel_end];
    if let Some((ts, te)) = trimmed_range(slice) {
        spans.push(span(
            offset + rel_start + ts,
            offset + rel_start + te,
            kind,
            len,
        ));
    }
}

/// Emits a `Value` span over the value side, trimming surrounding whitespace and one pair
/// of wrapping single or double quotes.
fn push_value(
    spans: &mut Vec<Span>,
    line: &[u8],
    rel_start: usize,
    rel_end: usize,
    offset: usize,
    len: usize,
) {
    let slice = &line[rel_start..rel_end];
    let Some((mut ts, mut te)) = trimmed_range(slice) else {
        return;
    };
    // Strip a matched wrapping quote pair.
    if te - ts >= 2 {
        let first = slice[ts];
        let last = slice[te - 1];
        if (first == b'"' || first == b'\'') && first == last {
            ts += 1;
            te -= 1;
        }
    }
    spans.push(span(
        offset + rel_start + ts,
        offset + rel_start + te,
        NodeKind::Value,
        len,
    ));
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
    fn extracts_keys_values_and_comments() {
        let buf = b"# secrets\nexport API_KEY=\"sk-live-abc123\"\nDB_HOST=db.internal\nEMPTY=\n";
        let result = EnvParser.parse(buf);
        assert_eq!(result.artefact_kind, ArtefactKind::EnvFile);

        let keys = of_kind(buf, &result.spans, NodeKind::Key);
        assert!(keys.contains(&"API_KEY"), "keys: {keys:?}");
        assert!(keys.contains(&"DB_HOST"), "keys: {keys:?}");

        let values = of_kind(buf, &result.spans, NodeKind::Value);
        assert!(values.contains(&"sk-live-abc123"), "values: {values:?}");
        assert!(values.contains(&"db.internal"), "values: {values:?}");

        let comments = of_kind(buf, &result.spans, NodeKind::Comment);
        assert!(comments.iter().any(|c| c.contains("secrets")));
    }

    #[test]
    fn does_not_truncate_a_value_containing_a_hash() {
        let buf = b"PASSWORD=p#ssw0rd#123\n";
        let parsed = EnvParser.parse(buf);
        let values = of_kind(buf, &parsed.spans, NodeKind::Value);
        assert_eq!(values, vec!["p#ssw0rd#123"]);
    }

    #[test]
    fn every_span_is_within_bounds() {
        let buf = b"A=1\nexport B='x'\n=novalue\nNOEQUALS\nC=\"unterminated\n";
        for s in EnvParser.parse(buf).spans {
            assert!(s.start <= s.end && s.end <= buf.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_env() {
        let cases: &[&[u8]] = &[
            b"",
            b"=",
            b"export ",
            b"export =",
            b"KEY=\"unterminated",
            b"KEY='",
            b"\"\"=\"\"",
            b"   \n\t\n",
            &[0xFF, 0xFE, b'=', 0x00],
            b"A=1\r\n\xff\xfe",
        ];
        for &c in cases {
            assert_parser_never_panics(&EnvParser, c);
        }
        assert_parser_never_panics(&EnvParser, &[b'='; 5000]);
        assert_parser_never_panics(&EnvParser, &b"K=v\n".repeat(2000));
    }
}
