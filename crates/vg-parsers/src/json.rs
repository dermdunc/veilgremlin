//! Tolerant JSON parser.
//!
//! This does **not** use `serde_json`'s tree parser. `serde_json` gives no byte offsets
//! for the values it parses, and it aborts at the first syntax error — the opposite of
//! what the [`Parser`](vg_core::Parser) contract wants (best-effort spans over malformed
//! input). Instead this is a hand-rolled, single-pass byte tokenizer: it finds every
//! string and scalar token and classifies each string as an object [`Key`](NodeKind::Key)
//! (a string immediately followed by `:`) or a [`StringLiteral`](NodeKind::StringLiteral)
//! value. Unbalanced brackets, an unterminated final string, or outright binary content
//! all degrade to "whatever tokens we could find", never a panic.

use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{hint_extension, language_is, mime_contains, span};

#[derive(Debug, Default, Clone, Copy)]
pub struct JsonParser;

impl Parser for JsonParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "json" || e == "jsonl" || e == "ndjson")
            || language_is(artefact, &["json", "jsonc", "jsonl"])
            || mime_contains(artefact, &["json"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        ParseResult {
            spans: scan(buf),
            artefact_kind: ArtefactKind::Json,
        }
    }
}

/// Finds the byte range of a JSON string token starting at `open` (which must be the
/// opening `"`). Returns `(content_start, content_end, close_after)` where the content
/// range excludes the quotes and `close_after` is the index just past the closing quote
/// (or `buf.len()` for an unterminated string — the best-effort case). Never indexes out
/// of bounds; a trailing backslash at end-of-buffer is treated as a literal, not an
/// escape that reads past the end.
fn scan_string(buf: &[u8], open: usize) -> (usize, usize, usize) {
    let content_start = open + 1;
    let mut i = content_start;
    while i < buf.len() {
        match buf[i] {
            b'\\' => {
                // Skip the escaped byte; if the backslash is the last byte, stop here
                // rather than reading past the end.
                i += 2;
            }
            b'"' => return (content_start, i, i + 1),
            _ => i += 1,
        }
    }
    // Unterminated: best-effort span to end of buffer.
    (content_start, buf.len(), buf.len())
}

/// True if the next non-whitespace byte at or after `i` is a `:`.
fn next_nonspace_is_colon(buf: &[u8], mut i: usize) -> bool {
    while i < buf.len() {
        match buf[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b':' => return true,
            _ => return false,
        }
    }
    false
}

fn is_scalar_byte(b: u8) -> bool {
    b.is_ascii_digit()
        || matches!(
            b,
            b'-' | b'+'
                | b'.'
                | b'e'
                | b'E'
                | b't'
                | b'r'
                | b'u'
                | b'f'
                | b'a'
                | b'l'
                | b's'
                | b'n'
        )
}

/// Tokenizes `buf` into key/value/string spans. `pub(crate)` because the YAML parser
/// reuses it for flow-style (`{a: 1}` / `[1, 2]`) documents, which are JSON-shaped and
/// which YAML's block-oriented line scanner does not itself decompose.
pub(crate) fn scan(buf: &[u8]) -> Vec<Span> {
    let mut spans = Vec::new();
    let len = buf.len();
    let mut i = 0;
    while i < len {
        match buf[i] {
            b'"' => {
                let (cs, ce, after) = scan_string(buf, i);
                let kind = if next_nonspace_is_colon(buf, after) {
                    NodeKind::Key
                } else {
                    NodeKind::StringLiteral
                };
                spans.push(span(cs, ce, kind, len));
                i = after;
            }
            b if is_scalar_byte(b) => {
                // Consume a run of scalar bytes (number / true / false / null). We tag it
                // Value without validating it's a real literal — a detector reading these
                // spans wants "this region is a value", not JSON-grammar correctness.
                let start = i;
                while i < len && is_scalar_byte(buf[i]) {
                    i += 1;
                }
                spans.push(span(start, i, NodeKind::Value, len));
            }
            _ => i += 1,
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use vg_core::conformance::assert_parser_never_panics;

    fn text<'a>(buf: &'a [u8], s: &Span) -> &'a str {
        std::str::from_utf8(&buf[s.start..s.end]).unwrap_or("<non-utf8>")
    }

    #[test]
    fn classifies_object_keys_and_string_values() {
        let buf = br#"{"email": "jane@example.com", "count": 3}"#;
        let result = JsonParser.parse(buf);
        assert_eq!(result.artefact_kind, ArtefactKind::Json);

        let keys: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Key))
            .map(|s| text(buf, s))
            .collect();
        assert!(keys.contains(&"email"), "keys were {keys:?}");
        assert!(keys.contains(&"count"), "keys were {keys:?}");

        let strings: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::StringLiteral))
            .map(|s| text(buf, s))
            .collect();
        assert!(
            strings.contains(&"jane@example.com"),
            "string values were {strings:?}"
        );
    }

    #[test]
    fn handles_escaped_quotes_inside_strings() {
        let buf = br#"{"msg": "he said \"hi\" to me"}"#;
        let result = JsonParser.parse(buf);
        let strings: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::StringLiteral))
            .map(|s| text(buf, s))
            .collect();
        assert_eq!(strings, vec![r#"he said \"hi\" to me"#]);
    }

    #[test]
    fn every_span_is_within_bounds() {
        let buf = br#"{"a": [1, 2, {"b": "c"}], "d": true, "e": null}"#;
        for s in JsonParser.parse(buf).spans {
            assert!(s.start <= s.end && s.end <= buf.len());
        }
    }

    #[test]
    fn unterminated_string_degrades_to_best_effort_span() {
        let buf = br#"{"key": "value with no closing quote"#;
        let result = JsonParser.parse(buf);
        // Must not panic and must produce a span for the still-open string, clamped to
        // the buffer end.
        assert!(result.spans.iter().all(|s| s.end <= buf.len()));
        assert!(result
            .spans
            .iter()
            .any(|s| s.node_kind == Some(NodeKind::StringLiteral)));
    }

    #[test]
    fn parser_never_panics_on_adversarial_json() {
        let cases: &[&[u8]] = &[
            b"",
            b"{",
            b"}",
            b"[[[[[[[[[[",
            b"{\"unterminated",
            b"\"\\",                         // string that is just a trailing escape
            &[0xFF, 0xFE, 0x00, b'{', b'"'], // binary masquerading as JSON
            b"{\"a\":\"\\u00",               // truncated unicode escape
            b":::::::::",
            b"null true false 123 -4.5e10",
        ];
        for &c in cases {
            assert_parser_never_panics(&JsonParser, c);
        }
        // A pathological wall of quotes and backslashes.
        assert_parser_never_panics(&JsonParser, &[b'\\'; 5000]);
        assert_parser_never_panics(&JsonParser, &[b'"'; 5000]);
    }
}
