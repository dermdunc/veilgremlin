# The seam that needed one more argument

**2026-07-18 — T07, wiring the masking pipeline**

For three weeks `scan` and `mask` — the two functions the whole product exists to provide —
had been a single line each: `todo!()`. Everything they were supposed to call had been built
by other squads and merged: five detectors, the keying and vault, the audit log, the policy
engine, the parsers. T07's job was the last mile — make the two functions actually do the
thing.

The interesting part wasn't the wiring. It was that the frozen contract was subtly wrong, and
had been since the day it was frozen.

`scan(input, ctx)` takes a `Context` — the bag of detectors and parsers it runs. `mask(input,
policy, ns)` does not. But `mask`'s entire job is to *detect and then mask*: it needs the same
detectors `scan` uses. The frozen signature gave it no way to reach them. You could feel the
temptation of the two easy workarounds — stuff the detectors into the `Policy` object that was
already being passed in, or make the caller run `scan` first and hand `mask` the findings. Both
would have compiled. Both were wrong. The first conflates "what the policy says to do" with
"how you find the sensitive data" — two things that should never share a struct. The second
lets a caller hand `mask` a list of findings that doesn't match the input at all — a stale set,
or a hand-forged one — and `mask` would dutifully mask against a lie.

So we did the boring, correct thing: added `ctx: &Context` to `mask`, bumped the contract to
v1.1, and wrote down *why* the two shortcuts were rejected, so the next person who feels the
same temptation finds the argument already made. No caller had wired `mask` yet, so nothing
broke — the cost of the change was a documentation paragraph, paid now instead of during a
migration later.

The other thing worth remembering is how many of this pipeline's rules are the same rule wearing
different clothes: **don't fail open.** A `Pass` classification means "send it after masking" —
it does *not* mean "skip detection," or an artefact the policy waved through would carry its
secrets out unmasked. Detectors scan the *full* raw buffer, never just the regions a parser
found structure in — because a parser that under-spans a quoted YAML value (a documented, real
bug two squads ago) would otherwise become a secret the pipeline never even looked at. An
irreversibly-redacted password is *never* handed to the vault — not "we don't expect to," but
never, and there's a test that re-opens the vault from disk to prove the mapping count stayed
zero. Every one of those is the same instinct: when in doubt, the safe direction is *more*
masking, not less.

Written, as has become the pattern here, without a compiler in the room — `cargo` is
approval-gated in the headless dispatch, so the code is correct by reading the crates it calls,
not by watching it go green. The build happens at review. The tests are written to fail loudly
if any of the fail-open guardrails ever quietly stops holding.
