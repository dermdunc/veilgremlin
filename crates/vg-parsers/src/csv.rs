//! Tolerant CSV parser (RFC-4180-ish).
//!
//! Hand-rolled rather than backed by the `csv` crate: we want a *field span into the
//! original buffer* for every cell, and we must degrade gracefully on malformed input
//! (an unterminated quoted field, a stray quote, a binary blob) rather than returning a
//! parse error. The first row's cells are tagged [`Key`](NodeKind::Key) (a header row);
//! every later cell is tagged [`Field`](NodeKind::Field) named by its column's header, so
//! a downstream detector can be column-aware (e.g. "the `email` column"). Quoted fields
//! have their surrounding quotes excluded from the span; `""` escapes and embedded
//! newlines inside quotes are handled. An unterminated final quote runs to end-of-buffer.

use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser, Span};

use crate::util::{hint_extension, language_is, mime_contains, span};

#[derive(Debug, Default, Clone, Copy)]
pub struct CsvParser;

impl Parser for CsvParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "csv" || e == "tsv")
            || language_is(artefact, &["csv", "tsv"])
            || mime_contains(artefact, &["csv", "tab-separated"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        ParseResult {
            spans: scan(buf),
            artefact_kind: ArtefactKind::Csv,
        }
    }
}

struct Cell {
    row: usize,
    col: usize,
    /// Raw byte range of the field, quotes included.
    start: usize,
    end: usize,
}

/// Tokenizes `buf` into cells, tracking row/column. Never indexes out of bounds; an
/// unterminated quoted field simply extends to the end of the buffer.
fn tokenize(buf: &[u8]) -> Vec<Cell> {
    let mut cells = Vec::new();
    let len = buf.len();
    let mut i = 0;
    let mut field_start = 0;
    let mut row = 0;
    let mut col = 0;
    let mut in_quotes = false;
    let mut seen_any = false; // any byte on the current (row,col) field

    while i < len {
        let b = buf[i];
        if in_quotes {
            if b == b'"' {
                if buf.get(i + 1) == Some(&b'"') {
                    i += 2; // escaped quote
                    continue;
                }
                in_quotes = false;
            }
            i += 1;
        } else {
            match b {
                b'"' if !seen_any => {
                    in_quotes = true;
                    seen_any = true;
                    i += 1;
                }
                b',' => {
                    cells.push(Cell {
                        row,
                        col,
                        start: field_start,
                        end: i,
                    });
                    col += 1;
                    i += 1;
                    field_start = i;
                    seen_any = false;
                }
                b'\n' => {
                    let mut end = i;
                    if end > field_start && buf[end - 1] == b'\r' {
                        end -= 1;
                    }
                    // Emit the final field of the row unless the row is completely empty
                    // (a blank line between records shouldn't manufacture a phantom cell).
                    if col > 0 || end > field_start {
                        cells.push(Cell {
                            row,
                            col,
                            start: field_start,
                            end,
                        });
                    }
                    row += 1;
                    col = 0;
                    i += 1;
                    field_start = i;
                    seen_any = false;
                }
                _ => {
                    seen_any = true;
                    i += 1;
                }
            }
        }
    }
    // Trailing field with no closing newline.
    if field_start < len || col > 0 {
        cells.push(Cell {
            row,
            col,
            start: field_start,
            end: len,
        });
    }
    cells
}

/// Strips one pair of wrapping double-quotes from a raw field range, if present.
fn content_range(buf: &[u8], start: usize, end: usize) -> (usize, usize) {
    if end.saturating_sub(start) >= 2 && buf[start] == b'"' && buf[end - 1] == b'"' {
        (start + 1, end - 1)
    } else {
        (start, end)
    }
}

fn scan(buf: &[u8]) -> Vec<Span> {
    let len = buf.len();
    let cells = tokenize(buf);

    // Header names from row 0, for tagging later rows' cells.
    let mut headers: Vec<String> = Vec::new();
    for c in cells.iter().filter(|c| c.row == 0) {
        let (cs, ce) = content_range(buf, c.start, c.end);
        headers.push(String::from_utf8_lossy(&buf[cs..ce]).into_owned());
    }

    cells
        .iter()
        .map(|c| {
            let (cs, ce) = content_range(buf, c.start, c.end);
            let kind = if c.row == 0 {
                NodeKind::Key
            } else {
                match headers.get(c.col) {
                    Some(name) if !name.is_empty() => NodeKind::Field(name.clone()),
                    _ => NodeKind::Value,
                }
            };
            span(cs, ce, kind, len)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vg_core::conformance::assert_parser_never_panics;

    fn text<'a>(buf: &'a [u8], s: &Span) -> &'a str {
        std::str::from_utf8(&buf[s.start..s.end]).unwrap_or("<non-utf8>")
    }

    #[test]
    fn tags_header_row_as_keys_and_body_cells_by_column_name() {
        let buf = b"name,email\njane,jane@example.com\nbob,bob@example.org\n";
        let result = CsvParser.parse(buf);
        assert_eq!(result.artefact_kind, ArtefactKind::Csv);

        let keys: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Key))
            .map(|s| text(buf, s))
            .collect();
        assert_eq!(keys, vec!["name", "email"]);

        // The email column cells carry Field("email").
        let email_cells: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Field("email".to_string())))
            .map(|s| text(buf, s))
            .collect();
        assert_eq!(email_cells, vec!["jane@example.com", "bob@example.org"]);
    }

    #[test]
    fn handles_quoted_fields_with_commas_and_escaped_quotes() {
        let buf = b"a,b\n\"hello, world\",\"she said \"\"hi\"\"\"\n";
        let result = CsvParser.parse(buf);
        let field_a: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Field("a".to_string())))
            .map(|s| text(buf, s))
            .collect();
        assert_eq!(field_a, vec!["hello, world"]);
    }

    #[test]
    fn every_span_is_within_bounds() {
        let buf = b"x,y,z\n1,2,3\n\"un,closed\n4,5";
        for s in CsvParser.parse(buf).spans {
            assert!(s.start <= s.end && s.end <= buf.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_csv() {
        let cases: &[&[u8]] = &[
            b"",
            b",",
            b",,,,,\n,,,,\n",
            b"\"",
            b"\"unterminated, quote, running, off",
            b"a,b,c",        // no trailing newline
            b"\"\"\"\"\"\"", // a run of quotes
            &[0xFF, 0xFE, b',', 0x00, b'\n', b','],
            b"a,b\r\n1,2\r\n",
        ];
        for &c in cases {
            assert_parser_never_panics(&CsvParser, c);
        }
        assert_parser_never_panics(&CsvParser, &[b','; 5000]);
        assert_parser_never_panics(&CsvParser, &[b'"'; 5001]);
    }
}
