# Giving the repo a real name to say out loud

**2026-07-04**

The original scaffold routed VeilGremlin to the coderturtle GitHub account, private, on the general factory-output default. Four days later, before a single feature existed, that call got revisited and reversed.

The reasoning was simple once someone said it out loud: coderturtle is the agentic-engineering demo and workshop identity. VeilGremlin isn't a workshop or a demo. It's an enterprise architecture and governance control, the kind of thing you'd want attached to a professional identity, not a hobby account. So the repo moved: transferred from `coderturtle/veilgremlin` to `dermdunc/veilgremlin` (GitHub makes you accept a transfer by hand on the receiving side, no API shortcut there), and flipped from private to public in the same sitting rather than waiting for some future "ready to open-source" milestone that tends to arrive never.

Doing the visibility flip this early, before any real implementation existed, was the safer order. There was nothing yet to accidentally leak, no code to have missed reviewing for a stray secret or an internal path. Every project this factory builds is going to face this exact fork eventually: which identity does this belong under, and when does it go public. Getting the answer right at day four, instead of at day four hundred after a bunch of commits already assumed the wrong owner, is the whole reason this decision got its own entry instead of a footnote.

Full record, including the wider Hekton-side policy change this triggered: [`docs/decisions.md`](../decisions.md#2026-07-04---repo-ownership-moved-to-dermdunc-visibility-flipped-to-public).
