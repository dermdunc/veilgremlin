# Freezing the contract everyone else builds on

**2026-07-15**

Task T02's whole job was to freeze `vg-core`: the shared types, the trait seams every other crate would implement against (`Detector`, `Parser`, `VaultStore`, `PolicyEngine`, `AuditSink`), and a set of contract-conformance test helpers so five parallel squads could all build against the same seams without waiting on each other. This is the task that makes Wave B's parallelism possible at all, which also makes it the task where a mistake is most expensive to find late.

Dispatch actually did real work this time, unlike T01's stall, but got cut off by a tool timeout before it could close out formally. Rather than throw away a partially finished, seemingly-correct build and start over, the work got picked up in place: verified, tested, and finished by hand.

The review pass is where this one got interesting. `interface-contracts.md` was T02's own acceptance criterion, the document says so explicitly, and it turned out to have never actually been reconciled against the real code. Eleven types existed in the implementation that the frozen contract document didn't mention, and two places where the code had quietly deviated from what the contract said. A document that's supposed to be the frozen source of truth but silently drifted from reality is exactly the kind of thing that looks fine until squad three builds against the document, squad four builds against the code, and nobody notices the two don't agree until integration.

The second finding was a real bug, not a documentation gap: the mock vault implementation used in the conformance test suite, the same template every Wave B squad would read before writing their own vault code, ignored its namespace parameter entirely. A value stored under one session's namespace would happily resolve under a completely different one. Namespace isolation is supposed to be the guarantee that keeps one repo's secrets from leaking into another's masked output. The template meant to prove that guarantee worked was itself violating it, silently, in the one place every future squad would copy from.

Six more smaller conformance-helper gaps got found and fixed alongside it. All of it happened before Wave B ever touched the contract, which is the entire reason to freeze something before parallelizing five squads against it: so this is where a mistake like a broken namespace check gets caught, not somewhere downstream where five different crates have already built assumptions on top of it.

Full record: [`docs/decisions.md`](../decisions.md#2026-07-15---doubt-driven-development-on-the-t02-pr-contract-left-in-draft-a-real-vault-bug).
