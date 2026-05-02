//! Sync runtime and registry facades for Wendao Nexus.
//!
//! Runtime code owns scheduling, job state, checkpoint state, and content-hash
//! dedup. Durable backends can implement the registry traits without changing
//! connector or core contracts.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);

/// Content hash helpers for dedup registries.
pub mod hash;
/// Normalization contracts for raw source payloads.
pub mod normalize;
/// Local mirror query facade for normalized documents.
pub mod query;
/// Registry traits and the in-memory registry implementation.
pub mod registry;
/// Minimal source sync runtime over pluggable registries.
pub mod runtime;

pub use hash::sha256_content_hash;
pub use normalize::{KnowledgeDocumentNormalizer, NormalizationContext, PlainTextNormalizer};
pub use query::{InMemoryKnowledgeStore, LocalKnowledgeStore};
pub use registry::{CheckpointRegistry, ContentHashRegistry, InMemoryNexusRegistry, JobRegistry};
pub use runtime::{
    DiscoveryOutcome, FetchOutcome, IngestOutcome, NexusSyncRuntime, NormalizedIngestOutcome,
};
