# VeilGremlin seeded eval corpus

The labelled corpus the `vg-bench` harness scores for the Go/No-Go report (Task T10). One
file — [`manifest.json`](manifest.json) — holds every sample inline, so the whole corpus is
reviewable in one place and the loader has no path-resolution to get wrong.

## Synthetic-data rule (binding)

**No real PII, ever.** Every value here is synthetic by construction:

- Email domains are RFC 2606 reserved (`example.com`, `example.org`, `example.co.uk`).
- The IBAN is the published ISO 13616 example (`GB29 NWBK 6016 1331 9268 19`), not a real
  account.
- Secrets/tokens are fake-shaped random strings — high-entropy by construction, registrable
  by nobody.
- Phone numbers use reserved/example ranges (`+44 20 7946 0958` is Ofcom's drama-use London
  number; `+1-415-555-2671` uses the `555` fictional exchange).

## Sample schema

Each sample in `manifest.json`:

| field | meaning |
|---|---|
| `name` | stable id, used in the report |
| `description` | what the sample exercises |
| `artefact` | optional `{ path?, language_id?, mime_type? }` — the `ArtefactHint` for policy/parser selection. **Omitted = no hint** (the point of the dotenv-no-hint slice). |
| `content` | the raw buffer, verbatim |
| `expected` | ground-truth labels: `{ type, value }` pairs. The loader finds every occurrence of `value` in `content` and emits a labelled `Finding` (type + span). **Only entity types a Phase-1 detector actually emits** appear here (`Email`, `Phone`, `InternalIp`, `Iban`, `SortCode`, `Secret`) — labelling a type no detector emits would score a guaranteed miss. |
| `slices` | tags selecting the sample into a banked measurement (`benign-lookalike`, `dotenv-no-hint`, `display-collision`, …). |
| `decoys` | placeholder-shaped literals present in `content` that are **not** real values and must survive a mask→demask round-trip unchanged (display-collision measurement). |
| `residual_secrets` | sensitive values that entity detection is **known not to catch** (a short low-entropy password, a structured licence key). Used only by the dotenv-no-hint residual measurement to quantify what falls through without an artefact-level Block; **never** scored in the global metrics (they are out of the detectors' design scope, not detection misses). |

## What it must NOT do

The harness and report satisfy the same redaction discipline they measure: **no raw detected
value is ever printed** — the report shows refs, counts, and rates only. A corpus label's
`value` lives only in this file (as declared ground truth) and in the harness's in-memory
matching; it is never echoed to report output.
