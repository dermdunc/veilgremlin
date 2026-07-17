# The vault that had to remember its counting

**2026-07-17 — Task T05, `vg-vault`**

VeilGremlin's job is to swap real sensitive values (`jane@example.com`) for stable, boring
placeholders (`EMAIL_001`) before anything reaches a cloud model, and to swap them back
locally when someone with permission asks. The vault is the part that stores that mapping. Up
to today it was an empty stub crate with a one-paragraph doc comment promising a future. This
task built the real thing: an encrypted, on-disk store implementing the `VaultStore` contract
the rest of the system was already written against.

Most of it is the kind of work you'd expect. The whole database file is encrypted with
SQLCipher (AES-256). The key that unlocks it never touches the disk in plaintext — it lives in
the macOS Keychain, generated fresh the first time you open a vault and fetched from the
Keychain every time after. Raw values go in wrapped in `Secret`, the type that scrubs itself
from memory when dropped. The actual placeholder maths — the salted HMAC that turns a value
into a stable key — isn't reimplemented here; it calls straight into the keying module built
last task, precisely so two parts of the system can never disagree about what placeholder a
value should get.

The interesting part is a single sentence buried in the task: the vault's ordinal counters
must be *reseeded from its own stored records when it opens*. Here's why that sentence matters.

The `EMAIL_001`, `EMAIL_002`, `EMAIL_003` numbering is handed out by an in-memory counter. It
counts up as it sees new email addresses. But "in-memory" means it forgets everything when the
process exits. So imagine: you mask a document, the vault records `alice@…` → `EMAIL_001` and
`bob@…` → `EMAIL_002`, and stores both on disk. You quit. You come back tomorrow. The counter,
freshly created, is back at zero. You mask a new document containing `carol@…`. The counter
confidently hands out `EMAIL_001` — a label that already belongs, permanently, to Alice. Now
two different people share one placeholder. Every promise the tool makes about stable,
auditable, reversible masking is quietly broken, and nothing crashes to tell you.

The fix is to make the vault, on open, read back the highest number it ever assigned for each
namespace and category, and fast-forward the counter past it. Carol gets `EMAIL_003`, where she
belongs. This was flagged during the *previous* task's review — the reviewer noticed the keying
module had no way to be reseeded and wrote a loud note: don't let T05 skip this. T05 did not
skip it. It did require adding one small, additive method to the keying module (`seed_ordinal`)
that last task hadn't needed to exist yet — the counter's state was private with no way in.

There's a second subtlety that falls out of the first. When you re-intern a value the vault has
already seen, it must return the *old* number, not mint a new one — otherwise reseeding buys you
nothing. So the vault checks its durable records first (keyed by the HMAC, which is computed
without touching the counter at all) and only asks the counter for a fresh number once it's
certain the value is genuinely new. Look up first, count second. Get that order wrong and the
counter drifts every time a familiar value comes back.

The honest caveat, same as last task: there was no Rust compiler reachable in the room where
this was built. The code is written against interfaces that *are* verified, the tests are
written, and there's an integration test that literally opens a vault, quits it, reopens it, and
checks Carol gets `003` — but none of it has been run yet. That happens at review. If the last
two tasks are any guide, the compiler and a fresh pair of eyes will find something. That's the
point of writing it all down first.
