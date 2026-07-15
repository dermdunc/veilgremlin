//! Identifier newtypes used across the shared types and trait seams.
//!
//! `SessionId` is UUID-backed because a session is ephemeral, generated per run.
//! `RepoId`/`OrgId` are string-backed because they are stable, human-legible identities
//! (a repo path/remote slug, a tenant name) rather than generated per call. `DetectorId`
//! and `ActorId` are string-backed for the same reason: they show up verbatim in audit
//! trails and should be readable there. `AuditId` is UUID-backed: it is generated fresh
//! by `AuditSink::write` for each event.

use uuid::Uuid;

/// Identity of one build/session run; scopes [`crate::Namespace::Session`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

/// Stable identity of a repository; scopes [`crate::Namespace::Repo`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoId(pub String);

/// Stable identity of an organisation/tenant; scopes [`crate::Namespace::Org`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrgId(pub String);

/// Stable identity of a `Detector` impl, recorded on every `Finding` for audit provenance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DetectorId(pub String);

/// Identity of the actor requesting a demask, checked by `PolicyEngine::demask_allowed`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId(pub String);

/// Identity of a persisted `AuditEvent`, returned by `AuditSink::write`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditId(pub Uuid);
