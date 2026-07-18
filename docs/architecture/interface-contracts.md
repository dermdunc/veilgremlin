# VeilGremlin — Interface Contracts (v1.1, frozen)

**Status:** **FROZEN as of 2026-07-15 (Task T02); amended to v1.1 on 2026-07-18 (Task T07 — `mask` gained `ctx: &Context`, see §2).** Changes now go through the contract-change protocol in `agent-factory-plan.md` §6 and bump the version below. This document was reconciled against the actual `vg-core` code at freeze time (a doubt-driven-development pass on the T02 PR found it had drifted from the implementation before either landed) — every type and trait below now matches `crates/vg-core/src/{types,traits,api}.rs` exactly, including the supporting types (§0) the original draft's illustrative signatures used but never defined.

These are the seams that let squads build in parallel. They are illustrative Rust signatures — Squad 0 owns the canonical definitions in `vg-core`. Other squads implement against these traits and **do not** depend on each other's internals.

> Phase 1 is in-process (library + CLI). The optional local daemon API (`POST /scan`, `/mask`, `/rehydrate`, `/policy/evaluate`, `GET /audit/{id}`, `/health`) is deferred to when an adapter needs a long-running service; its shapes mirror the library API below.

---

## 0. Supporting types (owned by `vg-core`)

The draft version of this document used these types in illustrative signatures without
defining them. Added at freeze time so a squad reading this document alone (the entire point
of freezing it) doesn't need to read `vg-core`'s source to know their shape.

```rust
/// Hints a caller can supply about an artefact before/without parsing it.
pub struct ArtefactHint { pub path: Option<PathBuf>, pub language_id: Option<String>, pub mime_type: Option<String> }

/// The structural kind a Parser determines an artefact to be. `#[non_exhaustive]`.
pub enum ArtefactKind {
    Json, Yaml, Toml, Sql, Csv, LogLine, Diff, EnvFile,
    SourceCode(String), PlainText, Unknown,
}

/// Structural context a Parser attaches to a Span. `#[non_exhaustive]`.
pub enum NodeKind { Key, Value, Field(String), StringLiteral, Comment, Identifier, Other(String) }

/// Raw bytes plus whatever hint the caller already has about their shape.
pub struct Input { pub buf: Vec<u8>, pub hint: ArtefactHint }

/// The detectors and parsers `scan` runs, borrowed as trait objects so `vg-core` never
/// depends on the Wave B crates that implement them.
pub struct Context<'a> { pub parsers: &'a [&'a dyn Parser], pub detectors: &'a [&'a dyn Detector] }

/// A resolved policy plus the vault/audit handles `mask` needs.
pub struct Policy { pub engine: Box<dyn PolicyEngine>, pub vault: Box<dyn VaultStore>, pub audit: Box<dyn AuditSink> }

/// Where a (un)masked value is headed. `#[non_exhaustive]`.
pub enum Destination { LocalPatch, LocalTestFixture, LocalExplanationBuffer, RemoteModelPrompt, ObservabilitySink }

/// Stable key for PolicyEngine::destination_allows_masked_only lookups — a separate type
/// from Destination since policy dictionaries key on a stable string, not the runtime enum.
pub struct DestinationId(pub String);

/// Who is requesting a demask, checked by PolicyEngine::demask_allowed.
pub struct Actor { pub id: ActorId, pub roles: Vec<String> }

/// A seeded evaluation corpus for `benchmark` (Task T10 populates real corpora).
pub struct Corpus { pub samples: Vec<CorpusSample> }
pub struct CorpusSample { pub input: Input, pub expected_findings: Vec<Finding> }

/// Go/No-Go metrics, per agent-factory-plan.md §8.
pub struct Metrics { pub recall: f64, pub precision: f64, pub false_positive_rate: f64, pub p95_latency_us: u64 }
```

---

## 1. Shared types (owned by `vg-core`)

```rust
/// Classification of a detected entity. `#[non_exhaustive]`.
pub enum EntityType {
    Person, Email, Phone, Address, Postcode,
    EmployeeId, CustomerId, AccountId, Iban, SortCode,
    InternalIp, Hostname, ApiKey, TraceId,
    Password, PrivateKey, Secret, AccessToken,
    Custom(String), // a policy-dictionary-defined class; only genuinely new
                     // *fixed* classes need a contract-change PR, dictionary
                     // entries do not
}

/// What the policy says to do with a class of entity/artefact.
pub enum HandlingClass {
    Mask,                // reversible typed placeholder via vault
    IrreversibleRedact,  // one-way; never vault-stored
    Block,               // do not send (artefact-level)
    Pass,                // non-sensitive
}

/// Namespace for placeholder stability.
pub enum Namespace { Session(SessionId), Repo(RepoId), Org(OrgId) }

/// A single detection over an input buffer.
pub struct Finding {
    pub entity_type: EntityType,
    pub span: Span,            // byte range in the (parsed) input
    pub confidence: f32,       // 0.0..=1.0
    pub detector: DetectorId,  // provenance for audit
}

/// Byte span + optional structural context from a parser.
pub struct Span { pub start: usize, pub end: usize, pub node_kind: Option<NodeKind> }

/// The only thing serialized toward a model. Contains NO raw values.
pub struct MaskedPack {
    pub text: String,                 // placeholders substituted
    pub mapping_refs: Vec<MappingRef>,// opaque handles into the vault
    pub stats: MaskStats,             // counts by EntityType, blocked artefacts
    pub policy_version: String,
}

pub struct MappingRef(pub Uuid);      // handle only; never the value

pub struct AuditEvent { /* see vg-audit contract */ }
```

**Invariant (tested, not type-enforced):** `MaskedPack` must never contain a raw detected value or a vault key in `.text` or `.policy_version`. This is a testing/convention discipline, not a type-system guarantee — every field is `pub` with no smart constructor, so nothing stops code from hand-constructing one directly. `MappingRef` being an opaque `Uuid` (never a real key) makes the "no vault key" half true by construction; "no raw value" depends on `mask()`'s correctness and test coverage (`vg_core::conformance::assert_masked_pack_excludes_raw_values`). Squad 5/7 add a property test.

---

## 2. Library API (owned by `vg-core`)

```rust
pub fn scan(input: &Input, ctx: &Context) -> Vec<Finding>;

// v1.1 (2026-07-18, Task T07): `ctx: &Context` added — see the versioned note below.
pub fn mask(input: &Input, ctx: &Context, policy: &Policy, ns: &Namespace)
    -> Result<(MaskedPack, Vec<MappingRef>, AuditEvent), MaskError>;

pub fn rehydrate(masked: &str, dest: Destination, actor: &Actor)
    -> Result<String, RehydrateDenied>;

pub fn benchmark(corpus: &Corpus, policy: &Policy) -> Metrics;
```

`Destination` includes `LocalPatch`, `LocalTestFixture`, `LocalExplanationBuffer`, `RemoteModelPrompt`, `ObservabilitySink`. The last two are **hard-deny** in default policy; `rehydrate` returns `RehydrateDenied` for them regardless of actor.

> **Contract change v1 → v1.1 (2026-07-18, Task T07) — `mask` gained `ctx: &Context`.**
> The v1 signature `mask(input, policy, ns)` had no way to reach the detectors and parsers
> `scan` gets via `Context` — `mask`'s whole job is to detect-then-mask, so it needs the
> same detectors/parsers `scan` runs. Resolved via the contract-change protocol
> (`agent-factory-plan.md` §6) with the **sanctioned** fix: an explicit
> `mask(input, ctx, policy, ns)` parameter. Deliberately *not* done by smuggling detectors
> into `Policy` (which would conflate "what to do" with "how to find it") or by having
> callers pre-compute `Vec<Finding>` (which would duplicate `scan`'s logic at every call
> site and let a caller mask a stale/hand-forged finding set). No other signature changed;
> `scan`/`rehydrate`/`benchmark` are untouched. See `../decisions.md`'s 2026-07-18 T07 entry.

---

## 3. Detector trait (implemented by `vg-detectors`)

```rust
pub trait Detector: Send + Sync {
    fn id(&self) -> DetectorId;
    /// Hot path: must be allocation-aware, no I/O, no network, no ML.
    fn detect(&self, buf: &[u8], spans: &[Span]) -> Vec<Finding>;
    fn entity_types(&self) -> &[EntityType];
}
```
Contract: deterministic; bounded matching; every returned `Span` must be a valid byte range into the input buffer (`start <= end <= buf.len()`) — later pipeline code slices by these spans; `detect` is benchmarked (p95 < 25 ms on the reference buffer). Warm-path NER (GLiNER) implements a separate `Enricher` trait, never `Detector`, and is only invoked off the hot path. `vg_core::conformance::assert_detector_contract` checks determinism, declared-type membership, confidence range, and span bounds.

```rust
pub trait Enricher: Send + Sync {            // WARM PATH ONLY
    fn enrich(&self, text: &str) -> Vec<Finding>;
}
```

---

## 4. Parser trait (implemented by `vg-parsers`)

```rust
pub trait Parser: Send + Sync {
    fn can_parse(&self, artefact: &ArtefactHint) -> bool;
    /// Returns typed spans so detectors can be field/structure-aware.
    fn parse(&self, buf: &[u8]) -> ParseResult; // error-tolerant; never panics
}

pub struct ParseResult { pub spans: Vec<Span>, pub artefact_kind: ArtefactKind }
```
Contract: must be robust to malformed input (return best-effort spans, never panic). Code parsing uses tree-sitter; format parsers for json/yaml/toml/sql/csv/log/diff/env. `vg_core::conformance::assert_parser_never_panics` checks this against whatever buffers the caller supplies — the helper itself can't verify panic-safety in general, so exercise it with genuinely adversarial input (empty, truncated UTF-8, unbalanced delimiters), not just one happy-path buffer.

---

## 5. Vault trait (implemented by `vg-vault`)

```rust
pub trait VaultStore: Send + Sync {
    /// Returns existing placeholder or creates one. Stores encrypted mapping.
    fn intern(&self, value: &Secret, ty: EntityType, ns: &Namespace)
        -> Result<Placeholder, VaultError>;
    /// Reverse a placeholder to its raw value. Caller must already be policy-authorised.
    fn resolve(&self, p: &Placeholder, ns: &Namespace)
        -> Result<Secret, VaultError>;
    fn purge_expired(&self) -> Result<usize, VaultError>;
}

pub struct Placeholder { pub display: String, pub mapping_ref: MappingRef }
pub struct Secret(/* zeroized on drop */);
```
Contract: AES-256 at rest (SQLCipher); DB key wrapped by OS keychain, never persisted plaintext; `Secret` zeroizes on drop; `IrreversibleRedact` values are **never** passed to `intern`. Stable placeholder via salted HMAC over `(canonical(value), ty, ns)`.

**Namespace isolation is part of this contract, not a convention:** `ns` in `resolve` must match the `ns` the `Placeholder` was interned under. A value interned in one `Namespace` must never resolve when called with a different one — return `VaultError::NotFound` on a namespace mismatch, the same as an unknown mapping (never distinguish "wrong namespace" from "doesn't exist" to a caller). `vg_core::conformance::assert_vault_roundtrip` checks this explicitly; any impl, including test mocks, must not skip it. (Found and fixed at freeze time: the T02 conformance example's mock vault originally ignored `ns` on resolve entirely.)

**`Secret`'s zeroize-on-drop is cosmetic at the one exit point that matters:** `expose_secret(&self) -> &str` lets a caller copy the value out before drop, and `rehydrate`'s frozen signature (§2) returns an owned, non-zeroizing `String` for the allowed-destination path. This is inherent to the contract's shape, not an implementation bug — callers of `expose_secret`/`rehydrate`'s output are responsible for not persisting or logging the returned value.

---

## 6. Policy trait (implemented by `vg-policy`)

```rust
pub trait PolicyEngine: Send + Sync {
    fn load(layers: PolicyLayers) -> Result<Self, PolicyError> where Self: Sized;
    fn classify_artefact(&self, hint: &ArtefactHint) -> HandlingClass;
    fn classify_entity(&self, ty: EntityType) -> HandlingClass;
    fn destination_allows_masked_only(&self, dest: &DestinationId) -> bool;
    fn demask_allowed(&self, dest: Destination, actor: &Actor) -> bool;
    fn version(&self) -> &str;
}

pub struct PolicyLayers { pub global: PathBuf, pub repo: Option<PathBuf>, pub session: Option<PathBuf> }
```
Contract: 3-layer resolution (session overrides repo overrides global); signed-pack verification before load (stub in Phase 1, enforced later); `demask_allowed` returns false for `RemoteModelPrompt`/`ObservabilitySink` in default policy — checked by `vg_core::conformance::assert_policy_engine_denies_hard_deny_destinations`.

*(Fixed at freeze time: the draft used `Path`, which isn't valid here — `Path` is `?Sized` and can't be an owned struct field. `PathBuf` is what the actual code needed and uses.)*

---

## 7. Audit trait (implemented by `vg-audit`)

```rust
pub trait AuditSink: Send + Sync {
    fn write(&self, event: AuditEvent) -> Result<AuditId, AuditError>;
    fn get(&self, id: AuditId) -> Option<AuditEvent>;
}

pub enum AuditEvent {
    Scan { counts: EntityCounts, detector_version: String, latency_us: u64 },
    PolicyDecision { artefact: ArtefactKind, class: HandlingClass, policy_version: String },
    MappingCreated { mapping_ref: MappingRef, entity_type: EntityType },
    Block { artefact: ArtefactKind, reason: String },
    DemaskRequest { dest: Destination, actor: ActorId },
    DemaskDecision { dest: Destination, actor: ActorId, allowed: bool, policy_version: String },
    // ... provider destination, build_provenance_version
}
```
Contract: append-only; **no raw values** in any variant (refs/counts/versions only); property-tested via `vg_core::conformance::assert_audit_event_excludes_raw_values`, which checks both the literal raw value and its `Debug`-escaped form (a raw value containing control characters renders escaped in `{event:?}`, so checking only the unescaped literal would false-negative on exactly that class of leak).

---

## 8. Adapter contract (implemented by `vg-adapters-claude`, consumes `vg-core`)

- Claude Code hooks map to: `UserPromptSubmit` → `mask(prompt)`; `PreToolUse`/`PostToolUse` → `mask(tool_io)`; pre-request → assemble `MaskedPack` only.
- Exit codes: `0` pass-through, `2` transformed (masked), `1` block (with reason to stderr) — matching Claude Code hook semantics.
- The wrapper (`vg run -- claude ...`) prints the pre-send summary from `MaskStats` and routes the masked request to Bedrock.
- The adapter never calls `vault.resolve` directly; demask is a separate user-invoked `vg demask` flow through `rehydrate`.

---

## Versioning
- **v1** — this document, frozen 2026-07-15 (Task T02), reconciled against the actual `vg-core` code at freeze time. See `../decisions.md`'s 2026-07-15 entry for what changed between the original draft and this frozen version (added §0 supporting types; `EntityType::Custom`; `PolicyLayers` `Path`→`PathBuf`; namespace-isolation and zeroize-cosmetic notes on `VaultStore`/`Secret`; conformance-helper coverage notes).
- **v1.1** — 2026-07-18 (Task T07). `mask` gained a `ctx: &Context` parameter (`mask(input, ctx, policy, ns)`) so it can reach the detectors/parsers `scan` composes; nothing else changed. See §2's inline contract-change note and `../decisions.md`'s 2026-07-18 T07 entry. Downstream: no current caller existed (the CLI/adapters had not yet wired `mask`), so no call sites needed migrating — future callers pass the same `Context` they build for `scan`.
- Increment on any breaking change to a public type/trait above. Record the bump in `../decisions.md` and notify downstream squads.
