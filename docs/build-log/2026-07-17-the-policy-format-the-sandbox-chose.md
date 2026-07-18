# 2026-07-17 — The policy format the sandbox chose

The task was clear enough: implement the policy engine — the component that decides what
gets masked, what gets blocked, and, most importantly, what is *never* allowed to be
un-masked. It even told us which config format to reach for: YAML or TOML, both reasonable,
pick one.

We picked neither. We shipped JSON. Here's why, because the reason is more interesting than
the choice.

## A decision made by a locked door

VeilGremlin's build agents run in a sandbox that, this session, would not let `cargo` run at
all. Not "run slowly" — every invocation came back with *requires approval* and there was no
one on the other end of the approval to say yes. Same for `rustfmt`. So the crate had to be
written, formatted to the compiler's taste, and reasoned correct entirely by hand, with no
build, no test run, no linter. (This is not new here — the keying module two days earlier had
the same handicap. See that build-log entry.)

That constraint quietly rewrote the format decision. Rust projects pin every dependency,
down to an exact checksum, in a `Cargo.lock` file — and `cargo build --locked`, which CI
runs, *fails* if that file doesn't already account for everything the code imports. Adding
`toml` or `serde_yaml` means adding a handful of new pinned packages (a TOML parser drags in
four or five of its own). Normally you just run `cargo build` once and it updates the lock for
you. We couldn't run cargo. So we couldn't update the lock. So we couldn't add those crates
without hand-writing package entries and checksums we had no way to compute or verify.

JSON, it turned out, was already in the room. `serde_json` was sitting in the lock file
already — pulled in months ago by the benchmarking library, fully pinned, ready to use. Choosing
it added *zero* new locked packages. The only change to the lock was two lines wiring the
policy crate to dependencies that were already there.

## Not a rejection, a deferral

The honest framing matters, so it went into `docs/decisions.md` in full: this deviates from
the project's own ADR-007 ("native YAML/TOML now"). It's a Phase-1 deviation forced by the
environment, not a considered rejection of TOML. The on-disk schema is plain serde structs, so
the format crate is one swap away — when a build environment that can regenerate the lock comes
along, we either switch to TOML or amend ADR-007 to bless JSON. That follow-up is written down
where it won't get lost.

## The part that isn't plumbing

The rest of a policy engine is configuration plumbing: read three files, let the session layer
override the repo layer override the global layer, look things up in maps. All of that got
built. But one rule in this crate is not plumbing, and the spec was blunt about it: demasking
toward a remote model prompt, or toward an observability sink, must be denied. Always. For any
actor. That's the whole point of the tool — it exists so a secret masked on the way out can't
be quietly un-masked back into a cloud model's context window.

So that rule does not live in the config. A policy file cannot turn it on. It's a hard `false`
in the code, checked *before* the pack is even consulted, and there's a test that feeds the
engine a deliberately malicious pack — one that sets `demask_allowed: true` for exactly those
two destinations — and asserts the engine denies them anyway. Config plumbing can be wrong and
you fix the config. This one couldn't be allowed to depend on the config being right.

Everything here is hand-verified, not machine-verified — CI still has to run the real gate.
But if there was one line in the crate worth reading three times before trusting the eyes
instead of the compiler, it was that one.

*Full technical record: `docs/decisions.md`, 2026-07-17 T06 entry.*
