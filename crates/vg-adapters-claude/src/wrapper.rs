//! Support for `vg run -- <cmd...>`: the settings file that wires VeilGremlin's hooks into
//! a wrapped Claude Code invocation, and the Bedrock env pass-through description.
//!
//! `vg run` deliberately builds **no HTTP client**. "Routing the masked request to Bedrock"
//! (`interface-contracts.md` §8) means configuring the *wrapped* Claude Code CLI: the hooks
//! (this file) mask everything Claude Code sends, and Claude Code's own Bedrock transport
//! (`CLAUDE_CODE_USE_BEDROCK=1` and friends, passed through from the caller's environment)
//! carries the already-masked request. VeilGremlin never talks to a model endpoint itself.

use std::path::Path;

use serde_json::{json, Value};

use crate::hook::HookEvent;

/// Environment variables `vg run` passes through untouched so the wrapped Claude Code CLI
/// can reach Bedrock exactly as it would without the wrapper. Listed for the pre-send
/// summary; `vg run` inherits the full environment regardless.
pub const BEDROCK_ENV_VARS: &[&str] = &[
    "CLAUDE_CODE_USE_BEDROCK",
    "AWS_REGION",
    "AWS_PROFILE",
    "AWS_BEARER_TOKEN_BEDROCK",
    "ANTHROPIC_BEDROCK_BASE_URL",
];

/// The `command` string a hook entry runs: the `vg` executable, pinned to this session's
/// state dir, dispatching the given event.
pub fn hook_command(vg_exe: &str, state_dir: &Path, event: HookEvent) -> String {
    // Both the executable path AND the state dir are quoted: current_exe() routinely
    // lives under paths with spaces ("/Users/John Smith/…"), and an unquoted exe makes
    // every hook silently fail to spawn (doubt-pass High).
    format!(
        "{} --state-dir {} hook {}",
        shell_quote(vg_exe),
        shell_quote(&state_dir.to_string_lossy()),
        event.cli_name()
    )
}

/// Builds the Claude Code settings JSON that registers all three VeilGremlin hooks, pinned
/// to `state_dir`. Written by `vg run` to `<state_dir>/claude-hooks.json`; the operator
/// points Claude Code at it (`claude --settings <file>`), per `docs/runbook-hooks.md`.
pub fn hook_settings_json(vg_exe: &str, state_dir: &Path) -> String {
    let mut hooks = serde_json::Map::new();
    for event in HookEvent::all() {
        let command = hook_command(vg_exe, state_dir, event);
        let entry = match event {
            // UserPromptSubmit has no tool matcher; the tool events match every tool.
            HookEvent::UserPromptSubmit => json!({
                "hooks": [{ "type": "command", "command": command }]
            }),
            HookEvent::PreToolUse | HookEvent::PostToolUse => json!({
                "matcher": "*",
                "hooks": [{ "type": "command", "command": command }]
            }),
        };
        hooks.insert(event.event_name().to_string(), json!([entry]));
    }
    let settings = json!({ "hooks": Value::Object(hooks) });
    serde_json::to_string_pretty(&settings).unwrap_or_else(|_| "{}".to_string())
}

/// Minimal single-quote shell-quoting for embedding a path in the hook command string.
fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-'))
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_json_registers_all_three_events() {
        let json = hook_settings_json("vg", Path::new("/tmp/.veilgremlin"));
        let v: Value = serde_json::from_str(&json).expect("valid json");
        let hooks = &v["hooks"];
        assert!(hooks.get("UserPromptSubmit").is_some());
        assert!(hooks.get("PreToolUse").is_some());
        assert!(hooks.get("PostToolUse").is_some());
        // The command threads the state dir and the event token.
        let cmd = hooks["UserPromptSubmit"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert!(cmd.contains("hook user-prompt-submit"), "{cmd}");
        assert!(cmd.contains("--state-dir"), "{cmd}");
    }

    #[test]
    fn shell_quote_wraps_paths_with_spaces() {
        assert_eq!(shell_quote("/tmp/a b"), "'/tmp/a b'");
        assert_eq!(shell_quote("/tmp/plain"), "/tmp/plain");
    }
}
