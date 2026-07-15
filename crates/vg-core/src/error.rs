//! Error types for the library API and trait seams.

use thiserror::Error;

use crate::api::Destination;
use crate::ids::ActorId;

#[derive(Debug, Error)]
pub enum MaskError {
    #[error("policy error: {0}")]
    Policy(#[from] PolicyError),
    #[error("vault error: {0}")]
    Vault(#[from] VaultError),
    #[error("audit error: {0}")]
    Audit(#[from] AuditError),
}

/// Returned by [`crate::rehydrate`] when the destination or actor is not authorised.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("rehydrate denied for {actor:?} -> {destination:?}: {reason}")]
pub struct RehydrateDenied {
    pub destination: Destination,
    pub actor: ActorId,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("mapping not found")]
    NotFound,
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("io error: {0}")]
    Io(String),
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("failed to load policy: {0}")]
    Load(String),
    #[error("policy signature verification failed: {0}")]
    Verify(String),
}

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("failed to write audit event: {0}")]
    Write(String),
}
