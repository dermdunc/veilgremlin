//! Shared helpers for the format parsers.
//!
//! Two concerns recur across every module here: (1) matching an [`ArtefactHint`] against a
//! parser's known file extensions / language ids / mime types in `can_parse`, and (2)
//! iterating a buffer line-by-line while keeping each line's absolute byte offset (so
//! spans point into the original buffer, not a per-line copy). Both live here so the
//! per-format modules stay focused on their own structure.

use vg_core::{ArtefactHint, NodeKind, Span};

/// Lower-cases and returns the file extension of `hint.path`, if any (without the dot).
pub(crate) fn hint_extension(hint: &ArtefactHint) -> Option<String> {
    hint.path
        .as_ref()
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

/// Returns the lower-cased final path component of `hint.path`, if any. Used by parsers
/// keyed on a full filename rather than an extension (e.g. `.env`, which has no
/// extension in the `Path::extension` sense).
pub(crate) fn hint_file_name(hint: &ArtefactHint) -> Option<String> {
    hint.path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase())
}

/// True if `hint`'s `language_id` (case-insensitive) is in `ids`.
pub(crate) fn language_is(hint: &ArtefactHint, ids: &[&str]) -> bool {
    hint.language_id
        .as_ref()
        .map(|l| {
            let l = l.to_ascii_lowercase();
            ids.iter().any(|id| *id == l)
        })
        .unwrap_or(false)
}

/// True if `hint`'s `mime_type` contains any of `needles` (case-insensitive substring).
pub(crate) fn mime_contains(hint: &ArtefactHint, needles: &[&str]) -> bool {
    hint.mime_type
        .as_ref()
        .map(|m| {
            let m = m.to_ascii_lowercase();
            needles.iter().any(|n| m.contains(n))
        })
        .unwrap_or(false)
}

/// Iterates `buf` line by line, yielding `(offset, line)` where `offset` is the absolute
/// byte index of the line's first byte in `buf` and `line` is the line's bytes with any
/// trailing `\n`/`\r\n` stripped. A trailing newline does not yield an extra empty line;
/// a buffer with no trailing newline still yields its final partial line.
pub(crate) fn lines_with_offsets(buf: &[u8]) -> Vec<(usize, &[u8])> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < buf.len() {
        if buf[i] == b'\n' {
            let mut end = i;
            if end > start && buf[end - 1] == b'\r' {
                end -= 1;
            }
            out.push((start, &buf[start..end]));
            start = i + 1;
        }
        i += 1;
    }
    if start < buf.len() {
        let mut end = buf.len();
        if end > start && buf[end - 1] == b'\r' {
            end -= 1;
        }
        out.push((start, &buf[start..end]));
    }
    out
}

/// Builds a span, clamping `end` to `len` and ensuring `start <= end`. Every parser routes
/// span construction through here so an off-by-one in some format's own scanning logic can
/// never produce an out-of-bounds span (the [`Parser`](vg_core::Parser) contract's
/// invariant that later pipeline code relies on when slicing by these spans).
pub(crate) fn span(start: usize, end: usize, kind: NodeKind, len: usize) -> Span {
    let end = end.min(len);
    let start = start.min(end);
    Span {
        start,
        end,
        node_kind: Some(kind),
    }
}
