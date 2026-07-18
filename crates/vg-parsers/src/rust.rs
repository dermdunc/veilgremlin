//! Rust source-code parser, backed by tree-sitter.
//!
//! Rust is the chosen source language (per the task's "pick something simple and common"
//! guidance; recorded in `docs/decisions.md`). Tree-sitter is error-tolerant by design —
//! it produces a full syntax tree with `ERROR`/`MISSING` nodes rather than failing on
//! malformed input — which is exactly the [`Parser`](vg_core::Parser) contract's
//! best-effort-never-panic requirement. We walk the tree and emit spans for the node
//! kinds a detector cares about: identifiers ([`Identifier`](NodeKind::Identifier)),
//! string/char literals ([`StringLiteral`](NodeKind::StringLiteral)), and comments
//! ([`Comment`](NodeKind::Comment)). `parse` returns `Option<Tree>`; a `None` (or a
//! failed language load, which cannot happen with a static grammar but is handled anyway)
//! yields an empty span list, never a panic.

use tree_sitter::{Node, Parser as TsParser};
use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{hint_extension, language_is, mime_contains};

/// The `SourceCode` tag this parser attaches, so downstream code keys on a stable string.
const LANGUAGE_ID: &str = "rust";

#[derive(Debug, Default, Clone, Copy)]
pub struct RustParser;

impl Parser for RustParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "rs")
            || language_is(artefact, &["rust", "rs"])
            || mime_contains(artefact, &["rust", "x-rust"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        ParseResult {
            spans: scan(buf),
            artefact_kind: ArtefactKind::SourceCode(LANGUAGE_ID.to_string()),
        }
    }
}

fn scan(buf: &[u8]) -> Vec<Span> {
    let mut parser = TsParser::new();
    if parser.set_language(&tree_sitter_rust::language()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(buf, None) else {
        return Vec::new();
    };

    let len = buf.len();
    let mut spans = Vec::new();
    let mut cursor = tree.walk();

    // Iterative DFS over every node, so a deeply nested source file can't blow the stack.
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if let Some(kind) = node_kind_for(&node) {
            let (start, end) = (node.start_byte(), node.end_byte());
            // Tree-sitter byte offsets are always within the input, but clamp defensively
            // so the Parser span-bounds invariant holds unconditionally.
            let end = end.min(len);
            let start = start.min(end);
            spans.push(Span {
                start,
                end,
                node_kind: Some(kind),
            });
        }
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }

    spans
}

/// Maps a tree-sitter node kind to the [`NodeKind`] we expose, or `None` for structural
/// nodes a detector doesn't need a span for.
fn node_kind_for(node: &Node) -> Option<NodeKind> {
    match node.kind() {
        "identifier" | "field_identifier" | "type_identifier" | "shorthand_field_identifier" => {
            Some(NodeKind::Identifier)
        }
        "string_literal" | "raw_string_literal" | "char_literal" => Some(NodeKind::StringLiteral),
        "line_comment" | "block_comment" => Some(NodeKind::Comment),
        _ => None,
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

    const SAMPLE: &[u8] = br#"
// connect to the internal db
fn connect() {
    let host = "db.internal";
    let email = "jane@example.com";
    log(host, email);
}
"#;

    #[test]
    fn artefact_kind_is_source_code_rust() {
        let result = RustParser.parse(SAMPLE);
        assert_eq!(
            result.artefact_kind,
            ArtefactKind::SourceCode("rust".to_string())
        );
    }

    #[test]
    fn extracts_string_literals_identifiers_and_comments() {
        let result = RustParser.parse(SAMPLE);

        let strings = of_kind(SAMPLE, &result.spans, NodeKind::StringLiteral);
        assert!(
            strings.iter().any(|s| s.contains("jane@example.com")),
            "strings: {strings:?}"
        );
        assert!(
            strings.iter().any(|s| s.contains("db.internal")),
            "strings: {strings:?}"
        );

        let idents = of_kind(SAMPLE, &result.spans, NodeKind::Identifier);
        assert!(idents.contains(&"connect"), "idents: {idents:?}");
        assert!(idents.contains(&"host"), "idents: {idents:?}");

        let comments = of_kind(SAMPLE, &result.spans, NodeKind::Comment);
        assert!(comments.iter().any(|c| c.contains("internal db")));
    }

    #[test]
    fn every_span_is_within_bounds() {
        for s in RustParser.parse(SAMPLE).spans {
            assert!(s.start <= s.end && s.end <= SAMPLE.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_rust() {
        let cases: &[&[u8]] = &[
            b"",
            b"fn",
            b"fn main() {",                 // unbalanced brace
            b"let x = \"unterminated",      // unterminated string
            b"}}}}}}}}}}",                  // stray closers
            b"fn f() { let s = r#\"raw",    // unterminated raw string
            &[0xFF, 0xFE, 0x00, 0x80],      // binary
            b"fn f() { \xff\xfe }",         // valid-ish rust wrapping invalid UTF-8
            "let π = \"café\";".as_bytes(), // multibyte UTF-8 identifiers/strings
        ];
        for &c in cases {
            assert_parser_never_panics(&RustParser, c);
        }
        assert_parser_never_panics(&RustParser, &[b'{'; 5000]);
        assert_parser_never_panics(&RustParser, &b"fn f(){}\n".repeat(2000));
    }
}
