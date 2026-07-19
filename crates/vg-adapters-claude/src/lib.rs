//! `vg-adapters-claude` — the Claude Code hook adapter plus the shared engine wiring the
//! `vg` CLI reuses.
//!
//! Two responsibilities live here:
//!
//! 1. **The hook adapter** ([`hook`]) — maps Claude Code hook events to `vg_core::mask`
//!    and the `interface-contracts.md` §8 **v1.3** protocol: exit `0` with empty stdout =
//!    pass-through; exit `0` with JSON stdout = the transform (`updatedInput` /
//!    `updatedToolOutput` / `decision:block` with masked resubmit text); exit `2` = block
//!    with the reason on stderr — the platform's only blocking exit code. It **never**
//!    calls `vault.resolve`; demask is the separate user-invoked `vg demask` flow.
//! 2. **The shared runtime** ([`state`] + [`runtime`] + [`pack`]) — the one place that
//!    resolves the `.veilgremlin/` state dir and opens the vault/policy/audit into a
//!    `vg_core::Policy`, so hooks and CLI commands mask/scan/demask through identical
//!    wiring. [`wrapper`] generates the hook settings `vg run` writes.
//!
//! Scaffolded in Task T01; implemented in Task T09.

pub mod hook;
pub mod pack;
pub mod runtime;
pub mod state;
pub mod wrapper;

pub use hook::{extract_subject, run_hook, HookEvent, HookResult};
pub use pack::{PackError, StoredNamespace, StoredPack};
pub use runtime::{open_vault, Engine, EngineError, DEFAULT_GLOBAL_POLICY, VAULT_KEY_ENV};
pub use state::{StatePaths, STATE_DIR_ENV, STATE_DIR_NAME};
pub use wrapper::{hook_settings_json, BEDROCK_ENV_VARS};
