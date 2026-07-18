//! Generic log-line parser.
//!
//! There is no single log format, so this is deliberately shape-based, not grammar-based:
//! for each line it emits a [`Value`](NodeKind::Value) span over the whole line (the
//! message body a detector will scan), plus a [`Field("timestamp")`](NodeKind::Field) and
//! [`Field("level")`](NodeKind::Field) span wherever a common timestamp (ISO-8601 or
//! syslog) or a severity word (`INFO`/`WARN`/`ERROR`/…) is recognised. Everything runs off
//! byte regexes over each line slice, so non-UTF-8 bytes and pathological input never
//! panic — a line that matches nothing simply yields its one `Value` span.

use std::sync::OnceLock;

use regex::bytes::{Regex, RegexBuilder};
use vg_core::{ArtefactHint, ArtefactKind, NodeKind, ParseResult, Parser};

use crate::util::{hint_extension, language_is, lines_with_offsets, mime_contains, span};

#[derive(Debug, Default, Clone, Copy)]
pub struct LogParser;

impl Parser for LogParser {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool {
        hint_extension(artefact).is_some_and(|e| e == "log")
            || language_is(artefact, &["log"])
            || mime_contains(artefact, &["x-log"])
    }

    fn parse(&self, buf: &[u8]) -> ParseResult {
        let len = buf.len();
        let mut spans = Vec::new();

        for (offset, line) in lines_with_offsets(buf) {
            if line.iter().all(|b| b.is_ascii_whitespace()) {
                continue;
            }
            // Whole-line message body.
            spans.push(span(offset, offset + line.len(), NodeKind::Value, len));

            if let Some(m) = timestamp_pattern().find(line) {
                spans.push(span(
                    offset + m.start(),
                    offset + m.end(),
                    NodeKind::Field("timestamp".to_string()),
                    len,
                ));
            }
            if let Some(m) = level_pattern().find(line) {
                spans.push(span(
                    offset + m.start(),
                    offset + m.end(),
                    NodeKind::Field("level".to_string()),
                    len,
                ));
            }
        }

        ParseResult {
            spans,
            artefact_kind: ArtefactKind::LogLine,
        }
    }
}

fn timestamp_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // ISO-8601-ish, or syslog "Mon DD HH:MM:SS".
        let iso = r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?";
        let syslog = r"[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}";
        RegexBuilder::new(&format!(r"(?:{iso})|(?:{syslog})"))
            .unicode(false)
            .build()
            .expect("log timestamp pattern is a valid, tested literal")
    })
}

fn level_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        RegexBuilder::new(r"\b(?:TRACE|DEBUG|INFO|NOTICE|WARN|WARNING|ERROR|ERR|FATAL|CRITICAL)\b")
            .case_insensitive(true)
            .unicode(false)
            .build()
            .expect("log level pattern is a valid, tested literal")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vg_core::conformance::assert_parser_never_panics;
    use vg_core::Span;

    fn text<'a>(buf: &'a [u8], s: &Span) -> &'a str {
        std::str::from_utf8(&buf[s.start..s.end]).unwrap_or("<non-utf8>")
    }

    fn of_field<'a>(buf: &'a [u8], spans: &'a [Span], name: &str) -> Vec<&'a str> {
        spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Field(name.to_string())))
            .map(|s| text(buf, s))
            .collect()
    }

    #[test]
    fn extracts_timestamp_level_and_message() {
        let buf = b"2026-07-17T08:14:22Z INFO user jane@example.com logged in\n";
        let result = LogParser.parse(buf);
        assert_eq!(result.artefact_kind, ArtefactKind::LogLine);

        assert_eq!(
            of_field(buf, &result.spans, "timestamp"),
            vec!["2026-07-17T08:14:22Z"]
        );
        assert_eq!(of_field(buf, &result.spans, "level"), vec!["INFO"]);

        // The whole line is available as a Value span for the detector to scan.
        let values: Vec<_> = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Value))
            .map(|s| text(buf, s))
            .collect();
        assert_eq!(values.len(), 1);
        assert!(values[0].contains("jane@example.com"));
    }

    #[test]
    fn handles_syslog_style_and_multiple_lines() {
        let buf =
            b"Jul 17 08:14:22 host sshd[123]: ERROR auth failure\nplain line with no metadata\n";
        let result = LogParser.parse(buf);
        assert_eq!(
            of_field(buf, &result.spans, "timestamp"),
            vec!["Jul 17 08:14:22"]
        );
        assert_eq!(of_field(buf, &result.spans, "level"), vec!["ERROR"]);
        // Two non-empty lines → two Value spans.
        let values = result
            .spans
            .iter()
            .filter(|s| s.node_kind == Some(NodeKind::Value))
            .count();
        assert_eq!(values, 2);
    }

    #[test]
    fn every_span_is_within_bounds() {
        let buf = b"2026-01-01 00:00:00 WARN x\n\n  \nfoo\n";
        for s in LogParser.parse(buf).spans {
            assert!(s.start <= s.end && s.end <= buf.len(), "bad span {s:?}");
        }
    }

    #[test]
    fn parser_never_panics_on_adversarial_logs() {
        let cases: &[&[u8]] = &[
            b"",
            b"\n\n\n",
            b"   \n\t\n",
            b"9999-99-99T99:99:99Z broken but regex-shaped",
            b"INFOINFOINFO no word boundary spam",
            &[0xFF, 0xFE, 0x00, b'I', b'N', b'F', b'O'],
            b"ERROR\r\n\xff\xfe truncated",
        ];
        for &c in cases {
            assert_parser_never_panics(&LogParser, c);
        }
        assert_parser_never_panics(&LogParser, &[b'\n'; 5000]);
        assert_parser_never_panics(&LogParser, &b"ERROR ".repeat(2000));
    }
}
