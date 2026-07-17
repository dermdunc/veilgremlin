# Teaching the fan-out to test itself

**2026-07-16**

With T03 merged, the next five tasks were about to build in parallel: detectors already exist, but the placeholder-keying logic, the file parsers, the vault, the audit sink, and the policy engine all still had to be built against each other's frozen contracts without touching each other's code. Before dispatching any of them, it seemed worth asking a basic question. If the whole point of VeilGremlin is to be an invisible control with the latency discipline of a trading system, how would any of that actually get tested as it's built, rather than assumed until the very end?

Two real gaps turned up. The latency budget existed in the interface contract but was only checked by making sure the code compiled, nothing measured it on every PR the way a regression gate should. And every Wave B crate was going to be built and tested in total isolation against the frozen contract, with no cross-crate integration exercised until Task T07 stitches everything together, several tasks downstream from where a shape mismatch would actually be introduced.

Both got fixed before dispatch. A plain test, no fancy benchmarking framework, now asserts the whole detector suite stays within a generous multiple of the real latency budget, and it runs on every PR already, for free, as a regression backstop rather than a precise measurement (that precision is still Task T10's job). The task specs for the keying, parsing, and CLI tasks all picked up a new requirement: test against the real detector output that already exists, not just hand-built mock values, and the CLI task gained something more unusual, a requirement that a human actually run the finished hooks interactively and say, on the record, whether it felt invisible or not. Latency budgets are necessary. They are not the same thing as a person not noticing the tool is there.

Then came the part that mattered most: instead of waiting for some future formal evaluation harness to be the first time real data touched the detectors, a small read-only tool got built to run the five detectors, right now, against real Hekton content. Not synthetic fixtures. A hundred and ninety-seven real files, real docs and logs and YAML, across VeilGremlin and one of its neighboring labs. The tool never prints or stores what it finds, only counts and latencies, on purpose, since the whole point is measuring the detectors without becoming a new place secrets could leak.

Latency came back clean. Precision didn't. The entropy detector alone produced 2,468 findings and the phone detector 783, both wildly out of proportion to the twelve emails and sixty-nine IPs found in the same corpus. Something was clearly wrong, and figuring out exactly what turned into its own story.

Full record: [`docs/decisions.md`](../decisions.md#2026-07-16---dogfooding-plan-codex--a-real-ci-enforced-latency-gate--a-real-detector-census).
