# VeilGremlin — Interface Contracts (v1, freeze target)

**Status:** DRAFT until end of Wave A (Task T02), then **FROZEN**. After freeze, changes go through the contract-change protocol in `agent-factory-plan.md` §6 and bump the version.

These are the seams that let squads build in parallel. They are illustrative Rust signatures — Squad 0 owns the canonical definitions in `vg-core`. Other squads implement against these traits and **do not** depend on each other's internals.

> Phase 1 is in-process (library + CLI). The optional local daemon API (`POST /scan`, `/mask`, `/rehydrate`, `/policy/evaluate`, `GET /audit/{id}`, `/health`) is deferred to when an adapter needs a long-running service; its shapes mirror the library API below.

---

## 1. Shared types (owned by `vg-core`)

```rust
/// Classification of a detected entity.
pub enum EntityType {
    Person, Email, Phone, Address, Postcode,
    EmployeeId, CustomerId, AccountId, Iban, SortCode,
    InternalIp, Hostname, ApiKey, TraceId,
    Password, PrivateKey, Secret, AccessToken,
    // extensible via policy dictionaries
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

**Invariant (tested):** `MaskedPack` must never contain a raw detected value or a vault key. Squad 5/7 add a property test.

---

## 2. Library API (owned by `vg-core`)

```rust
pub fn scan(input: &Input, ctx: &Context) -> Vec<Finding>;

pub fn mask(input: &Input, policy: &Policy, ns: &Namespace)
    -> Result<(MaskedPack, Vec<MappingRef>, AuditEvent), MaskError>;

pub fn rehydrate(masked: &str, dest: Destination, actor: &Actor)
    -> Result<String, RehydrateDenied>;

pub fn benchmark(corpus: &Corpus, policy: &Policy) -> Metrics;
```

`Destination` includes `LocalPatch`, `LocalTestFixture`, `LocalExplanationBuffer`, `RemoteModelPrompt`, `ObservabilitySink`. The last two are **hard-deny** in default policy; `rehydrate` returns `RehydrateDenied` for them regardless of actor.

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
Contract: deterministic; bounded matching; `detect` is benchmarked (p95 < 25 ms on the reference buffer). Warm-path NER (GLiNER) implements a separate `Enricher` trait, never `Detector`, and is only invoked off the hot path.

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
Contract: must be robust to malformed input (return best-effort spans, never panic). Code parsing uses tree-sitter; format parsers for json/yaml/toml/sql/csv/log/diff/env.

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

pub struct PolicyLayers { pub global: Path, pub repo: Option<Path>, pub session: Option<Path> }
```
Contract: 3-layer resolution (session overrides repo overrides global); signed-pack verification before load (stub in Phase 1, enforced later); `demask_allowed` returns false for `RemoteModelPrompt`/`ObservabilitySink` in default policy.

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
Contract: append-only; **no raw values** in any variant (refs/counts/versions only); property-tested.

---

## 8. Adapter contract (implemented by `vg-adapters-claude`, consumes `vg-core`)

- Claude Code hooks map to: `UserPromptSubmit` → `mask(prompt)`; `PreToolUse`/`PostToolUse` → `mask(tool_io)`; pre-request → assemble `MaskedPack` only.
- Exit codes: `0` pass-through, `2` transformed (masked), `1` block (with reason to stderr) — matching Claude Code hook semantics.
- The wrapper (`vg run -- claude ...`) prints the pre-send summary from `MaskStats` and routes the masked request to Bedrock.
- The adapter never calls `vault.resolve` directly; demask is a separate user-invoked `vg demask` flow through `rehydrate`.

---

## Versioning
- **v1** — this document, frozen at end of Wave A.
- Increment on any breaking change to a public type/trait above. Record the bump in `../decisions.md` and notify downstream squads.
