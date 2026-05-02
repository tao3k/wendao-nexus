//! Core contracts for Wendao Nexus.
//!
//! `wendao-nexus-core` models external source identity, normalized documents,
//! provenance, connector boundaries, query contracts, and sync job state. It
//! does not own storage, network clients, or runtime orchestration.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);

/// Authority and trust policy types for filtering evidence.
pub mod authority;
/// Connector trait shared by external source adapters.
pub mod connector;
/// Raw and normalized external knowledge document contracts.
pub mod document;
/// Cross-crate error and result types.
pub mod error;
/// Agent-facing query request and evidence response contracts.
pub mod query;
/// Source identity, capability, cursor, checkpoint, and delta contracts.
pub mod source;
/// Runtime-independent sync job contracts.
pub mod sync;
/// Provenance bundle and evidence boundary contracts.
pub mod trust;

pub use authority::{AuthorityLevel, TrustPolicy, TrustSignal};
pub use connector::KnowledgeSourceConnector;
pub use document::{
    CitationRef, ExternalKnowledgeDocument, ExtractedDocumentResource,
    ExtractedDocumentResourceSet, FigureRef, KnowledgeSection, LicenseInfo, RawSourceDocument,
    SourceMetadata, TableRef,
};
pub use error::{NexusError, NexusResult};
pub use query::{
    EvidenceConflictMode, EvidenceRecord, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeOpenRequest, ExternalKnowledgeRefreshRequest, ExternalKnowledgeSearchRequest,
    ExternalKnowledgeSearchResponse,
};
pub use source::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceKind, SourceCapabilities, SourceChange,
    SourceCheckpoint, SourceCursor, SourceItemRef,
};
pub use sync::{NexusJobKind, NexusJobRecord, NexusJobStatus};
pub use trust::{EvidenceBoundary, ProvenanceBundle, ProvenanceRecord};
