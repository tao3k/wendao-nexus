//! Error types shared across `Wendao Nexus` crates.

use thiserror::Error;

/// Shared result type for Nexus contracts and runtime integrations.
pub type NexusResult<T> = Result<T, NexusError>;

/// Errors that can cross crate boundaries without exposing backend internals.
#[derive(Debug, Error)]
pub enum NexusError {
    #[error("source `{source_id}` item `{external_id}` was not found")]
    NotFound {
        source_id: String,
        external_id: String,
    },

    #[error("connector `{source_id}` does not support {operation}")]
    Unsupported {
        source_id: String,
        operation: &'static str,
    },

    #[error("invalid source configuration: {0}")]
    InvalidSource(String),

    #[error("registry error: {0}")]
    Registry(String),

    #[error("artifact store error: {0}")]
    Artifact(String),

    #[error("normalization error: {0}")]
    Normalize(String),

    #[error("sync error: {0}")]
    Sync(String),
}
