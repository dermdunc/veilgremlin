//! `vg-adapters-claude` — Claude Code hook adapter, consumes `vg-core`.
//!
//! Hooks map to: `UserPromptSubmit` → `mask(prompt)`; `PreToolUse`/`PostToolUse` →
//! `mask(tool_io)`. Exit codes match Claude Code hook semantics: `0` pass-through,
//! `2` transformed (masked), `1` block. Never calls `vault.resolve` directly — demask
//! is a separate user-invoked `vg demask` flow (`docs/architecture/interface-contracts.md` §8).
//!
//! Scaffolded in Task T01; implementation lands in Task T09.
