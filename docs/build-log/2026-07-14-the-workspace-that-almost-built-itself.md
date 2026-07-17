# The workspace that almost built itself

**2026-07-14**

Task T01 was supposed to be the easy one: scaffold a nine-crate Cargo workspace, wire up CI, add a supply-chain skeleton. No detection logic, no policy engine, just the container everything else would live in. The plan was to let the factory build it unattended through the real dispatch pipeline, agentic-control-tower talking to engine-gateway-lab talking to a headless Claude session.

It got about as far as checking whether the Rust toolchain was installed, then stopped and asked for permission to run that check. Headless mode has no one to ask. The dispatch sat there, technically still running, accomplishing nothing, until someone noticed and killed it.

That's not really a VeilGremlin bug. It's a gap in how unattended dispatch handles a Bash permission prompt with nobody home to approve it, and it got flagged upward for the tooling that owns that problem. But it also meant T01 wasn't going to build itself that day, so it got built directly instead: the workspace, the CI pipeline (fmt, clippy with warnings denied, cargo-deny, cargo-audit, a locked build), and a release skeleton with SBOM and signing stubs.

The more interesting part came after, when two rounds of adversarial review went looking for what the first pass missed. Round one found that the CI's dependency-audit job was actually failing on the real GitHub Actions run, a Docker-based action that doesn't work on the macOS runner it had been assigned to, despite every local check saying green. Round two, a colder second opinion from a different model, found six more things: no test job in CI at all, an unpinned Rust toolchain, a stale claim in the risk log, reproducibility scripts that had never been updated to actually check for the toolchain they depend on, and hardcoded dependency versions duplicated across every crate instead of centralized once. All six got fixed before the PR merged.

The lesson that stuck: "it built on my machine" and "the CI job is green" are two different claims, and only one of them was actually being checked until someone went looking.

Full record: [`docs/decisions.md`](../decisions.md#2026-07-14---go-live-dispatch-real-and-t01-built-directly-after-a-dispatch-mechanism-gap).
