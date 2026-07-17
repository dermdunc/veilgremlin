# Scaffolding a privacy shield before it has any privacy logic

**2026-06-30**

VeilGremlin started as ten architecture decisions and zero lines of Rust.

The pitch is simple to say and hard to build: keep real PII and sensitive enterprise identifiers out of a cloud model's context window when a developer is vibe-coding with Claude Code, Codex, Cursor, or whatever comes next, without the developer noticing it's there. Local-first. Masking automatic. Demasking explicit, local, and audited. The cloud model only ever sees placeholders that behave like the real thing.

Ten ADRs went down before any code did: Rust for the core (memory safety, no GC, small enough to actually review), SQLCipher for the local vault, a deterministic-detector hot path with an optional warm-path NER model for the cases regex can't catch, typed placeholders instead of synthetic fake data, and a policy layer in plain YAML/TOML for now rather than reaching for something like Cedar before there's a reason to. The build method itself got its own decision: contract-first, with squads owning one crate each once the interfaces freeze. That last one matters more than it sounds. It's the difference between "we'll figure out the seams as we go" and "the seams are fixed before anyone starts building on either side of them," and it's the decision that made everything from Task T02 onward possible to parallelize at all.

Nothing runs yet. That's fine. The whole point of a Hekton factory-output scaffold is that the classification, the governance scaffolding, and the documentation contract exist before the first real feature does, so that when the building starts, it starts on solid ground instead of retrofitting process onto code that already has bad habits.

Full ADR log: [`docs/decisions.md`](../decisions.md).
