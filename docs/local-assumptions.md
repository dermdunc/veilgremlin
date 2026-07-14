# Local Assumptions

Document machine-specific assumptions for this project.

## Expected Paths

- Hekton root: `~/hekton`
- Development root: `~/Development/hekton`
- Vault root: `~/vaults/hekton-mind-palace`

## Required Tools

- `git`
- `bash`
- Rust toolchain (`cargo`, `rustc`, `rustfmt`, `clippy`) — pinned via `rust-toolchain.toml`
  (added Task T01, 2026-07-14). CI installs this automatically; local dev needs it installed
  (`brew install rust` or `rustup`).
- `cargo-deny`, `cargo-audit` — supply-chain checks (added Task T01). `brew install cargo-deny
  cargo-audit`, or `cargo install`.

