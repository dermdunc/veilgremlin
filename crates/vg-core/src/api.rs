//! Library API owned by `vg-core`: `scan`/`mask`/`rehydrate`/`benchmark`, plus the
//! supporting types those signatures need (`Input`, `Context`, `Policy`, `Actor`,
//! `Destination`, `Corpus`, `Metrics`).
//!
//! T02 froze the signatures and implemented the one piece of real logic that does not
//! depend on any Wave B crate: `rehydrate`'s destination hard-deny gate. **T07 (this
//! task) wires the rest**: `scan` (parse → detect) and `mask` (classify artefact →
//! parse → detect → resolve overlaps → classify/mask each entity → audit), composing the
//! Wave B trait objects `vg-core` reaches only through `Context`/`Policy` — see
//! `docs/architecture/agent-factory-plan.md` §3. `vg-core` still does not depend on any
//! implementing crate except as dev-dependencies for the integration tests (the T04
//! precedent); the real detectors/parsers/vault/policy/audit arrive as trait objects.
//!
//! **Contract change (T07, recorded in `docs/architecture/interface-contracts.md` §2):**
//! `mask` gained a `ctx: &Context` parameter — `mask(input, ctx, policy, ns)`. The frozen
//! signature had no way to reach the detectors/parsers `scan` gets via `ctx`, and the
//! sanctioned fix per the contract-change protocol (`agent-factory-plan.md` §6) is an
//! explicit parameter, not smuggling detectors into `Policy` or pre-computing findings.

use std::collections::BTreeMap;
use std::time::Instant;

use crate::audit::AuditEvent;
use crate::error::{MaskError, RehydrateDenied};
use crate::ids::ActorId;
use crate::traits::{
    ArtefactHint, ArtefactKind, AuditSink, Detector, Parser, PolicyEngine, Secret, VaultStore,
};
use crate::types::{
    EntityCounts, EntityType, Finding, HandlingClass, MappingRef, MaskStats, MaskedPack, Namespace,
    Span,
};

/// Raw bytes plus whatever hint the caller already has about their shape.
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub buf: Vec<u8>,
    pub hint: ArtefactHint,
}

/// The detectors and parsers `scan` runs, borrowed as trait objects so `vg-core` never
/// depends on the Wave B crates that implement them.
pub struct Context<'a> {
    pub parsers: &'a [&'a dyn Parser],
    pub detectors: &'a [&'a dyn Detector],
}

/// A resolved policy plus the vault/audit handles `mask` needs to classify, intern, and
/// record its decision. Bundled into one struct (rather than three separate `mask`
/// parameters) because `mask` always needs all three together.
pub struct Policy {
    pub engine: Box<dyn PolicyEngine>,
    pub vault: Box<dyn VaultStore>,
    pub audit: Box<dyn AuditSink>,
}

/// Where a (un)masked value is headed. `RemoteModelPrompt` and `ObservabilitySink` are
/// **hard-deny**: see [`rehydrate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Destination {
    LocalPatch,
    LocalTestFixture,
    LocalExplanationBuffer,
    RemoteModelPrompt,
    ObservabilitySink,
}

impl Destination {
    /// Stable key for `PolicyEngine::destination_allows_masked_only` lookups —
    /// deliberately a separate type ([`DestinationId`]) from `Destination` itself, since
    /// policy dictionaries key on a stable string, not the richer runtime enum.
    pub fn id(&self) -> DestinationId {
        let s = match self {
            Destination::LocalPatch => "local-patch",
            Destination::LocalTestFixture => "local-test-fixture",
            Destination::LocalExplanationBuffer => "local-explanation-buffer",
            Destination::RemoteModelPrompt => "remote-model-prompt",
            Destination::ObservabilitySink => "observability-sink",
        };
        DestinationId(s.to_string())
    }

    /// True for destinations `rehydrate` denies regardless of actor or `PolicyEngine`
    /// impl — the "hard-deny... regardless of actor" invariant from
    /// `interface-contracts.md` §2.
    fn is_hard_deny(&self) -> bool {
        matches!(
            self,
            Destination::RemoteModelPrompt | Destination::ObservabilitySink
        )
    }
}

/// Lightweight key `PolicyEngine` impls use to look up destination policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DestinationId(pub String);

/// Who is requesting a demask, checked by `PolicyEngine::demask_allowed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub id: ActorId,
    pub roles: Vec<String>,
}

/// A seeded evaluation corpus for `benchmark` (Task T10 populates real corpora).
#[derive(Debug, Clone, Default)]
pub struct Corpus {
    pub samples: Vec<CorpusSample>,
}

#[derive(Debug, Clone)]
pub struct CorpusSample {
    pub input: Input,
    pub expected_findings: Vec<Finding>,
}

/// Go/No-Go metrics, per `agent-factory-plan.md` §8.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Metrics {
    pub recall: f64,
    pub precision: f64,
    pub false_positive_rate: f64,
    pub p95_latency_us: u64,
}

/// Parses `input` with the first applicable parser (spans are enrichment only), then runs
/// **every** detector over the full raw buffer and returns all findings.
///
/// **Spans never gate detection** (T08 review, hard requirement): parser spans are passed
/// through to `Detector::detect` for structure-awareness, but detectors always scan the
/// whole buffer — the two documented YAML under-spanning reproducers (a `#` inside a
/// quoted value; single-quoted flow scalars, see `docs/decisions.md`) would become
/// missed-secret bugs if detection were span-gated.
pub fn scan(input: &Input, ctx: &Context) -> Vec<Finding> {
    let spans = parser_spans(input, ctx);
    let mut findings = Vec::new();
    for detector in ctx.detectors {
        findings.extend(detector.detect(&input.buf, &spans));
    }
    findings
}

/// Detects, classifies, and masks `input` per `policy`: classifies the artefact (blocking
/// its content entirely if the policy says `Block`), scans every non-blocked artefact for
/// entities regardless of its artefact class, resolves overlapping findings, then masks /
/// redacts / passes each entity per its policy class, interning only reversible (`Mask`)
/// values into the vault. Emits one `AuditEvent` (a `Block` for a blocked artefact,
/// otherwise a `Scan`), both writing it via `policy.audit` and returning it.
///
/// The `ctx` parameter is the T07 contract change (see the module doc): without it `mask`
/// could not reach the detectors/parsers `scan` composes.
pub fn mask(
    input: &Input,
    ctx: &Context,
    policy: &Policy,
    ns: &Namespace,
) -> Result<(MaskedPack, Vec<MappingRef>, AuditEvent), MaskError> {
    let policy_version = policy.engine.version().to_string();

    // 1. Artefact-level Block FIRST, before the content is parsed or touched at all
    //    (doubt-pass finding: parsing a Block-classed artefact rested on the unstated
    //    assumption that no parser ever copies content into its output — `ArtefactKind::
    //    SourceCode(String)`/`NodeKind::Field(String)` are content-capable by type, and
    //    the CSV parser already clones header names into `Field`. `classify_artefact`
    //    reads only the hint, so checking it first costs nothing and removes the whole
    //    class). The Block event records `ArtefactKind::Unknown`: kind provenance would
    //    require parsing, and not touching blocked content outranks provenance detail.
    if policy.engine.classify_artefact(&input.hint) == HandlingClass::Block {
        let event = AuditEvent::Block {
            artefact: ArtefactKind::Unknown,
            reason: "artefact class is Block in resolved policy".to_string(),
        };
        policy.audit.write(event.clone())?;
        let pack = MaskedPack {
            text: String::new(),
            mapping_refs: Vec::new(),
            stats: MaskStats {
                counts: EntityCounts::default(),
                blocked_artefacts: 1,
            },
            policy_version,
        };
        return Ok((pack, Vec::new(), event));
    }

    // 2. Every non-Block artefact is entity-scanned regardless of its artefact class —
    //    `Pass` means "send after entity masking", never "skip detection" (T06 review,
    //    hard requirement: skipping here would make `Pass` fail open).
    // 3-4. Parse for enrichment spans, then detect: every detector over the FULL buffer
    //    (spans are enrichment only, never a gate — T08 review).
    let spans = parser_spans(input, ctx);
    let detect_start = Instant::now();
    let mut findings = Vec::new();
    for detector in ctx.detectors {
        findings.extend(detector.detect(&input.buf, &spans));
    }
    let latency_us = detect_start.elapsed().as_micros() as u64;

    // Runtime span guard (doubt-pass finding): `Context` accepts any `&dyn Detector`, and
    // conformance is checked in each implementing crate's own tests, not at this seam. A
    // non-conformant span would panic the slice below (no pack, no audit event — the
    // worst failure shape for a masking tool), and a zero-width span would splice in a
    // phantom placeholder and intern the empty string. Drop both, loudly in debug.
    findings.retain(|f| {
        let ok = f.span.start < f.span.end && f.span.end <= input.buf.len();
        debug_assert!(
            ok || f.span.start == f.span.end,
            "non-conformant detector span {:?} from {:?}",
            f.span,
            f.detector
        );
        ok
    });

    // Detection counts for the Scan audit event come from the RAW findings, before
    // overlap resolution (Codex round-2 fix: resolve_overlaps now emits *fragments*, so
    // counting its output recorded one trimmed `Secret` finding as N detections —
    // corrupting the "what did detection find" audit metric the counts exist to answer).
    let mut detected_counts: BTreeMap<EntityType, usize> = BTreeMap::new();
    for finding in &findings {
        *detected_counts
            .entry(finding.entity_type.clone())
            .or_insert(0) += 1;
    }

    // 5. Resolve overlaps: no byte is masked twice, and no detected byte is dropped —
    //    a losing finding is trimmed to its uncovered fragments, never discarded whole.
    // 6. Classify and intern in FORWARD document order (so vault ordinals read top-to-
    //    bottom: the first email in the buffer is EMAIL_001), collecting replacements;
    //    splices are applied back-to-front afterwards so offsets stay valid.
    let ordered = resolve_overlaps(findings);

    let mut mapping_refs: Vec<MappingRef> = Vec::new();
    let mut handled_counts: BTreeMap<EntityType, usize> = BTreeMap::new();
    let mut replacements: Vec<(Span, String)> = Vec::new();

    // On a mid-pipeline failure, values interned so far are durably in the vault; write a
    // best-effort partial Scan event so the audit trail accounts for them (doubt-pass
    // finding: `?` straight out of the loop left persisted mappings with zero audit
    // record). The best-effort write's own error is ignored — the original error wins.
    let mut intern_error: Option<MaskError> = None;

    for finding in &ordered {
        let ty = finding.entity_type.clone();
        let raw = &input.buf[finding.span.start..finding.span.end];
        // Contract-shape note: `Secret` is a `String` (frozen at T02), so a non-UTF-8
        // span cannot round-trip byte-exactly. Unreachable with the current all-ASCII
        // detectors (every T03 pattern matches pure-ASCII bytes); a future non-ASCII
        // detector needs a contract revisit, recorded in docs/decisions.md.
        let value = String::from_utf8_lossy(raw).into_owned();

        let replacement: Option<String> = match policy.engine.classify_entity(ty.clone()) {
            HandlingClass::Mask => {
                match policy.vault.intern(&Secret::new(value), ty.clone(), ns) {
                    Ok(placeholder) => {
                        // Dedup (doubt-pass finding): a value seen N times returns the
                        // same MappingRef N times; the pack lists each mapping once.
                        if !mapping_refs.contains(&placeholder.mapping_ref) {
                            mapping_refs.push(placeholder.mapping_ref);
                        }
                        Some(placeholder.display)
                    }
                    Err(e) => {
                        intern_error = Some(e.into());
                        break;
                    }
                }
            }
            // Never intern: the "irreversible class is never vault-stored" acceptance
            // criterion is tested, not assumed.
            HandlingClass::IrreversibleRedact => Some(redaction_marker(&ty)),
            // Entity-level Block: redact this span only — do not fail the whole artefact
            // for one entity — and count it like any other handled entity.
            HandlingClass::Block => Some(redaction_marker(&ty)),
            // Pass: leave the bytes untouched, and don't count a no-op as handled —
            // it was already counted in detected_counts above (from the RAW findings):
            // the Scan audit event answers "what did detection find", not only "what did
            // we replace" (doubt-pass finding; raw-not-fragment counting per Codex round).
            HandlingClass::Pass => None,
        };

        if let Some(replacement) = replacement {
            *handled_counts.entry(ty).or_insert(0) += 1;
            replacements.push((finding.span.clone(), replacement));
        }
    }

    // 7. One `AuditEvent::Scan`: counts are ALL detections (including policy-Passed
    //    ones — the audit trail must be able to answer "what was found"); the pack's
    //    `MaskStats.counts` below are the *handled* subset (what was actually replaced).
    let event = AuditEvent::Scan {
        counts: EntityCounts(detected_counts),
        detector_version: detector_version(ctx),
        latency_us,
    };

    if let Some(err) = intern_error {
        let _ = policy.audit.write(event); // best-effort partial record; original error wins
        return Err(err);
    }
    policy.audit.write(event.clone())?;

    // Apply splices back-to-front so earlier offsets stay valid.
    let mut out = input.buf.clone();
    replacements.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    for (span, replacement) in replacements {
        out.splice(span.start..span.end, replacement.into_bytes());
    }
    let text = String::from_utf8_lossy(&out).into_owned();

    let pack = MaskedPack {
        text,
        mapping_refs: mapping_refs.clone(),
        stats: MaskStats {
            counts: EntityCounts(handled_counts),
            blocked_artefacts: 0,
        },
        policy_version,
    };
    Ok((pack, mapping_refs, event))
}

/// Spans from the first parser whose `can_parse` claims `input.hint`, or none. Shared by
/// [`scan`] and [`mask`]; the first-match rule mirrors `vg_parsers::all_parsers`'s own
/// documented "caller takes the first `can_parse` match" contract.
fn parser_spans(input: &Input, ctx: &Context) -> Vec<Span> {
    ctx.parsers
        .iter()
        .find(|p| p.can_parse(&input.hint))
        .map(|p| p.parse(&input.buf).spans)
        .unwrap_or_default()
}

/// Resolves overlapping findings so no two accepted findings cover the same bytes — and
/// no detected byte is ever dropped: the more specific entity type wins over the generic
/// entropy `Secret`, and among equally specific findings the longer span wins (T03's docs
/// anticipate exactly the `Email`-over-`Secret` case). A losing finding is **trimmed to
/// its fragments not covered by higher-priority findings**, never discarded whole.
///
/// The trim (vs. the original accept-or-drop) is a doubt-pass High fix: the entropy
/// detector's own documented tokenizer residual merges a secret with an adjacent
/// email/host into one `Secret` span (`user-secret@host` shapes); accept-or-drop let the
/// `Email` tail win and silently discarded the `Secret` head — bytes inside a *detected*
/// finding surviving raw into the masked output. Trimming masks both: the email as
/// `EMAIL_NNN`, the uncovered head as its own `Secret` fragment.
///
/// Returned findings are sorted in forward document order (ascending span start), which
/// the caller relies on for top-to-bottom vault ordinal minting.
fn resolve_overlaps(mut findings: Vec<Finding>) -> Vec<Finding> {
    findings.sort_by(|a, b| {
        specificity(&b.entity_type)
            .cmp(&specificity(&a.entity_type))
            .then((b.span.end - b.span.start).cmp(&(a.span.end - a.span.start)))
            // Stable, deterministic tiebreak so the resolution never depends on detector
            // ordering.
            .then(a.span.start.cmp(&b.span.start))
            .then(a.entity_type.cmp(&b.entity_type))
    });

    let mut accepted: Vec<Finding> = Vec::new();
    for finding in findings {
        // Subtract every already-accepted interval from this finding's span; keep each
        // surviving fragment as a finding of the same type. An exact duplicate (or a
        // fully-covered lower-priority finding) yields no fragments and drops out — that
        // is the "never double-mask" half of the contract.
        let mut fragments = vec![(finding.span.start, finding.span.end)];
        for acc in &accepted {
            let mut next = Vec::new();
            for (start, end) in fragments {
                if acc.span.start >= end || acc.span.end <= start {
                    next.push((start, end)); // no overlap with this accepted span
                } else {
                    if start < acc.span.start {
                        next.push((start, acc.span.start)); // fragment left of the winner
                    }
                    if acc.span.end < end {
                        next.push((acc.span.end, end)); // fragment right of the winner
                    }
                }
            }
            fragments = next;
            if fragments.is_empty() {
                break;
            }
        }
        for (start, end) in fragments {
            debug_assert!(start < end);
            accepted.push(Finding {
                span: Span {
                    start,
                    end,
                    node_kind: finding.span.node_kind.clone(),
                },
                ..finding.clone()
            });
        }
    }
    accepted.sort_by_key(|f| f.span.start);
    accepted
}

/// Priority for overlap resolution. The entropy detector's catch-all `Secret` is the one
/// generic type — any concretely-typed finding (`Email`, `Iban`, `Password`, …) is more
/// specific and outranks it when they cover the same bytes.
fn specificity(ty: &EntityType) -> u8 {
    match ty {
        EntityType::Secret => 0,
        _ => 1,
    }
}

/// The fixed typed marker an `IrreversibleRedact`/entity-`Block` value is replaced with
/// (`[REDACTED:PASSWORD]`). Deliberately carries the type tag, never the value.
fn redaction_marker(ty: &EntityType) -> String {
    let tag = match ty {
        EntityType::Person => "PERSON",
        EntityType::Email => "EMAIL",
        EntityType::Phone => "PHONE",
        EntityType::Address => "ADDRESS",
        EntityType::Postcode => "POSTCODE",
        EntityType::EmployeeId => "EMPLOYEE_ID",
        EntityType::CustomerId => "CUSTOMER_ID",
        EntityType::AccountId => "ACCOUNT_ID",
        EntityType::Iban => "IBAN",
        EntityType::SortCode => "SORT_CODE",
        EntityType::InternalIp => "INTERNAL_IP",
        EntityType::Hostname => "HOSTNAME",
        EntityType::ApiKey => "API_KEY",
        EntityType::TraceId => "TRACE_ID",
        EntityType::Password => "PASSWORD",
        EntityType::PrivateKey => "PRIVATE_KEY",
        EntityType::Secret => "SECRET",
        EntityType::AccessToken => "ACCESS_TOKEN",
        EntityType::Custom(name) => return format!("[REDACTED:CUSTOM:{name}]"),
    };
    format!("[REDACTED:{tag}]")
}

/// Provenance string for the `Scan` audit event: the sorted detector ids joined with `+`
/// (e.g. `email+entropy+ip`). A stable, human-legible record of which detector set
/// produced the counts, without embedding any raw value.
fn detector_version(ctx: &Context) -> String {
    let mut ids: Vec<String> = ctx.detectors.iter().map(|d| d.id().0).collect();
    ids.sort();
    ids.join("+")
}

/// Reverses a masked placeholder back to its raw value for an authorised, local
/// destination.
///
/// The destination hard-deny gate below does not depend on any Wave B crate, so it is
/// implemented for real here rather than deferred: `RemoteModelPrompt` and
/// `ObservabilitySink` are denied unconditionally, before any `PolicyEngine` or
/// `VaultStore` is even consulted. Resolving an *allowed* destination still needs a
/// wired `VaultStore` + `PolicyEngine`, so that half is deferred to Task T07/T09.
pub fn rehydrate(
    masked: &str,
    dest: Destination,
    actor: &Actor,
) -> Result<String, RehydrateDenied> {
    if dest.is_hard_deny() {
        return Err(RehydrateDenied {
            destination: dest,
            actor: actor.id.clone(),
            reason: "destination is hard-deny in default policy".to_string(),
        });
    }
    let _ = masked;
    todo!("vault/policy-authorised resolution wired in Task T07/T09 (vg-cli demask gate)")
}

/// Runs `corpus` through the pipeline and reports Go/No-Go metrics. Wired in Task T10.
pub fn benchmark(corpus: &Corpus, policy: &Policy) -> Metrics {
    let _ = (corpus, policy);
    todo!("eval harness wired in Task T10 (vg-bench)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(id: &str) -> Actor {
        Actor {
            id: ActorId(id.to_string()),
            roles: vec!["admin".to_string()],
        }
    }

    #[test]
    fn rehydrate_denies_remote_model_prompt_regardless_of_actor() {
        let err = rehydrate(
            "{{EMAIL_1}}",
            Destination::RemoteModelPrompt,
            &actor("admin-1"),
        )
        .expect_err("RemoteModelPrompt must be hard-denied even for an admin actor");
        assert_eq!(err.destination, Destination::RemoteModelPrompt);
    }

    #[test]
    fn rehydrate_denies_observability_sink_regardless_of_actor() {
        let err = rehydrate(
            "{{EMAIL_1}}",
            Destination::ObservabilitySink,
            &actor("admin-1"),
        )
        .expect_err("ObservabilitySink must be hard-denied even for an admin actor");
        assert_eq!(err.destination, Destination::ObservabilitySink);
    }

    #[test]
    #[should_panic(expected = "wired in Task T07/T09")]
    fn rehydrate_allowed_destination_is_not_yet_wired() {
        // Documents current state rather than asserting a real resolution: the
        // vault/policy-authorised path is out of scope for T02 and lands in T07/T09.
        let _ = rehydrate("{{EMAIL_1}}", Destination::LocalPatch, &actor("admin-1"));
    }

    #[test]
    fn destination_id_is_stable_for_policy_lookups() {
        assert_eq!(
            Destination::RemoteModelPrompt.id(),
            DestinationId("remote-model-prompt".to_string())
        );
    }
}
