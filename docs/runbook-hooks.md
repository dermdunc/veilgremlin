# Runbook: wiring VeilGremlin's hooks into a real Claude Code session

This is the walkthrough for the T09 acceptance criterion that no benchmark can satisfy: a
human runs a real interactive Claude Code session with the hooks live, against realistic
masking-worthy content, and gives an honest verdict on whether the control is *invisible* —
no perceptible added latency, no workflow friction. The verdict (either way) is recorded in
`docs/decisions.md`; if it IS perceptible, that is a finding to fix, not a criterion to relax.

## 1. Build and stage

```bash
cd <veilgremlin repo>
cargo build --release -p vg-cli
export PATH="$PWD/target/release:$PATH"   # or copy target/release/vg somewhere on PATH
```

## 2. Pick a working directory and start the wrapper

`vg run` resolves (or creates) a repo-local `.veilgremlin/` state dir, writes the hook
settings file, prints the pre-send summary, and starts the wrapped command with the hooks
registered:

```bash
cd <some repo you're comfortable testing in>
vg run -- claude
```

What just happened, visible in the summary lines:

- `.veilgremlin/` now holds the vault DB, the audit log, the policy layers (a default
  `global.policy.json` was bootstrapped if you had none), and `claude-hooks.json`.
- Because the wrapped command starts with `claude` and had no `--settings`, the wrapper
  appended `--settings .veilgremlin/claude-hooks.json` automatically. (Wrapping anything
  else, or supplying your own settings file, skips that.)
- Bedrock: any of `CLAUDE_CODE_USE_BEDROCK` / `AWS_REGION` / `AWS_PROFILE` /
  `AWS_BEARER_TOKEN_BEDROCK` / `ANTHROPIC_BEDROCK_BASE_URL` already in your environment
  pass straight through to the wrapped CLI — the summary lists which are set. VeilGremlin
  itself never talks to a model endpoint.

Add `.veilgremlin/` to the repo's `.gitignore` if it isn't already.

## 3. Exercise it with masking-worthy content

In the wrapped session, try prompts and tool flows that carry real-shaped (not real!)
sensitive content, e.g.:

- a prompt containing an email address and a phone number ("summarize this complaint from
  jane.doe@example.com, callback +1-415-555-2671");
- asking Claude to read a log file you seeded with an IBAN and a high-entropy fake token;
- asking it to write a `.env` file (the artefact-level Block should refuse the tool call —
  exit 2 with a policy reason, visible in the session as a blocked hook).

What to expect mechanically (contract v1.3):

- **Tool input/output is masked invisibly** — `PreToolUse` substitutes the masked tool
  input via `updatedInput` before the tool runs; `PostToolUse` substitutes the masked
  result via `updatedToolOutput` before the model sees it. You should see placeholders in
  what Claude reads/writes, with no interruption.
- **A sensitive *prompt* is blocked, not rewritten** — Claude Code cannot rewrite a prompt
  in flight, so the hook rejects it and shows you the masked version to resubmit. This is
  the one deliberately visible seam; judge in step 5 whether the friction is acceptable.
- **A policy Block** (e.g. writing a `.env`) exits 2 — the tool call is refused with the
  reason visible in the session.

## 4. Verify the guarantees held

```bash
vg audit 5                       # recent events: scans, blocks, demask decisions
vg vault stats                   # how many mappings were interned (never values)
ls .veilgremlin/packs/           # persisted packs, one per transformed artefact
vg demask --from .veilgremlin/packs/<uuid>.json --to local-patch     # explicit reversal
vg demask --from .veilgremlin/packs/<uuid>.json --to remote-model-prompt  # must be DENIED
```

## 5. The verdict (the actual acceptance criterion)

While using the session, judge honestly:

1. **Latency** — did prompt submission or tool use feel slower than a bare session? (Every
   hook invocation pays a process spawn + SQLCipher open + policy load; the numbers say
   ~tens of ms, but the criterion is *perception*, so the answer comes from you, not the
   bench.)
2. **Friction** — did masking ever get in the way (over-masked prompt confusing the model,
   a Block firing on something you legitimately needed, placeholder noise in replies)?
3. Record the session date, what you did, and the verdict verbatim in `docs/decisions.md`
   under the T09 entry — including "it felt slow" or "the model got confused by
   placeholders" if true. A perceptible control is a real finding.

## Troubleshooting

- **Hook seems inert** — confirm the wrapped CLI actually received `--settings`; run the
  hook by hand: `echo '{"hook_event_name":"UserPromptSubmit","prompt":"mail a@b.com"}' |
  vg hook user-prompt-submit; echo $?` (expect exit 0 and a `{"decision":"block",...}`
  JSON whose reason carries `mail EMAIL_001`).
- **Keychain prompts** — first use creates a real macOS keychain entry for the vault key;
  approve it once. The `VG_VAULT_KEY_HEX` env var is a test seam that bypasses the
  keychain — do not use it outside tests/CI.
- **Wrong state dir** — hooks and CLI resolve the nearest `.veilgremlin` walking up from
  the cwd; `--state-dir`/`VG_STATE_DIR` pin it explicitly.
