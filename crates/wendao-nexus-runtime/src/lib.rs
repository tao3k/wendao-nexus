//! Sync runtime and registry facades for Wendao Nexus.
//!
//! Runtime code owns scheduling, job state, checkpoint state, and content-hash
//! dedup. Wendao-side adapters can implement the registry traits without
//! changing connector or core contracts.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);

/// Artifact persistence boundary for raw and normalized Nexus payloads.
pub mod artifact;
/// Content hash helpers for dedup registries.
pub mod hash;
/// Normalization contracts for raw source payloads.
pub mod normalize;
/// Registry traits and the in-memory registry implementation.
pub mod registry;
/// Minimal source sync runtime over pluggable registries.
pub mod runtime;

pub use artifact::{
    ArtifactDescriptor, ArtifactKind, ArtifactPayload, ArtifactStore, ArtifactWrite,
    LocalFileArtifactStore,
};
pub use hash::sha256_content_hash;
pub use normalize::{KnowledgeDocumentNormalizer, NormalizationContext, PlainTextNormalizer};
pub use registry::{
    CheckpointRegistry, ContentHashRegistry, InMemoryNexusRegistry, JobRegistry, SourceRegistry,
};
pub use runtime::{
    ArtifactIngestOutcome, DiscoveryOutcome, FetchOutcome, IngestOutcome, NexusSyncRuntime,
    NormalizedIngestOutcome,
};
