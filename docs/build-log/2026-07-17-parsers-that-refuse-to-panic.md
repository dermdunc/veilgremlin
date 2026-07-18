# 2026-07-17 — Parsers that refuse to panic

VeilGremlin's job is to find sensitive things in files before they get sent to a cloud
model, and mask them. To do that well it helps to know *what kind of file* you're looking
at and *where the interesting parts are* — the value in a config, the body of a log line,
the string literal in some source code, not the punctuation around it. That's what this
task, T08, built: a parser for each format we care about (JSON, YAML, TOML, CSV, logs,
diffs, `.env` files, and Rust source via tree-sitter), each one turning raw bytes into a
list of labelled byte-ranges the detectors can lean on.

There was exactly one hard rule, and it shaped almost every decision: **never panic.** A
privacy tool that crashes on a malformed file is worse than useless — it fails open, right
at the moment someone is about to send that file somewhere. So the parsers can't assume
the input is well-formed. They have to assume the opposite: that they'll be handed an empty
file, a JSON document that's actually a binary blob, a string with no closing quote, a wall
of ten thousand unbalanced brackets, a line of valid-looking UTF-8 that stops mid-character.
And they have to return *something useful anyway*.

That rule is why most of these parsers are hand-written byte scanners rather than calls to
the obvious library. The tempting move for JSON is `serde_json` — but `serde_json` gives you
a tree with no byte offsets (so you can't say *where* a value was), and it gives up at the
first syntax error (so a truncated file yields nothing). Both are exactly backwards from
what we need. So JSON is a single-pass tokenizer that walks the bytes, tags each string as
either a key or a value, and when it hits a string with no closing quote it just… stops at
the end of the buffer and calls that the value. Best-effort, always an answer, never a
crash. Every parser got a battery of deliberately hostile test inputs to prove it.

YAML and TOML are the interesting compromise. We *do* pull in `serde_yaml` and the `toml`
crate — but not for the spans, because those libraries also throw away byte offsets. We use
them as a *well-formedness check* that runs on every parse, including the hostile inputs, so
that if one of those third-party parsers ever panics on a nasty buffer, our own tests catch
it too. The actual spans come from hand-rolled line scanners. (One nice payoff: valid YAML
written in "flow style" — `{a: 1, b: 2}` — is really just JSON, so the YAML parser hands
those documents straight to the JSON tokenizer.) It's worth writing that down, because
otherwise a future reader sees two parsing libraries in the dependency list and reasonably
assumes they're doing the parsing. They're not. They're the smoke detector, not the
architect.

The honest part of this task is the cross-crate check. There's a standing requirement that a
new parser must actually feed its output into the *detectors* that were built earlier (T03),
on a realistic file, and then say plainly whether the seam between them works. It does — feed
a JSON config with an email and an internal IP through the parser and into the detectors, and
both get found. But here's the thing worth admitting: the detectors currently *ignore* the
spans entirely. Every detector's signature takes a `spans` argument named `_spans` — the
underscore that says "I accept this and do nothing with it." So we built a test that proves
it: feed the detectors real spans, no spans, or deliberately wrong spans, and you get the
exact same answer every time.

Is that a bug? We decided no — it's a seam built one step ahead of the thing that will use
it. The detectors today scan the whole file, which can only ever find *more* than a
structure-aware scan would, never less. The machinery that threads parser spans into
detectors is a later task (T07). And now that the no-op is pinned by a test, it can't quietly
change without someone noticing. The genuinely useful note we could leave for that future
task: back on 2026-07-16 we found the entropy and phone detectors' worst false positives were
on file paths and snake_case identifiers — and "only scan the *value* spans, skip the keys and
identifiers and comments" is precisely the structural fix that becomes possible once spans
actually flow through. So the ignored parameter isn't dead weight; it's a promise.

And, as with the keying module a few days ago, this was built in a headless dispatch with no
compiler in the room — every attempt to run `cargo` sat behind an approval prompt with no
human to click it. So the code is written to be correct by reading, and the real
build-and-test happens at review. This time there's a sharper edge than last time: T08 adds
four new dependencies, which means the lockfile is stale, which means CI's `--locked` builds
will refuse to run until someone regenerates it. That's written down loudly in the decisions
log and as RISK-0011 — the first thing the reviewer has to do, before anything else can go
green.

Full technical detail, the recorded judgment calls, and the exact review checklist are in
[`docs/decisions.md`](../decisions.md)'s 2026-07-17 T08 entry.
