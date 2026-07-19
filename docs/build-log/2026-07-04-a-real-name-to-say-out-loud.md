# Going public before there was anything to leak

**2026-07-04**

Four days after the repo was scaffolded, before a single feature existed, VeilGremlin was made public.

Doing the visibility flip this early, before any real implementation existed, was the safer order, and for a privacy tool it is almost the point. There was nothing yet to accidentally leak: no code to have missed reviewing for a stray secret, no internal path baked into a config, no test fixture holding a real value. Going public on day four, instead of after a few hundred commits already assumed a private home, means every later commit is written knowing the whole world can read it.

That discipline is worth stating out loud because it is exactly what this product is about. A tool whose whole pitch is "do not let real identifiers leak" should hold its own repository to the same bar, and the cheapest moment to start is before there is anything in the tree to check.

Full record: [`docs/decisions.md`](../decisions.md#2026-07-04---repo-visibility-flipped-to-public).
