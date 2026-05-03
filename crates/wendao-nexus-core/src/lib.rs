//! Core contracts for Wendao Nexus.
//!
//! `wendao-nexus-core` models external source identity, normalized documents,
//! provenance, connector boundaries, query contracts, and sync job state. It
//! does not own storage, network clients, document parsers, Docling execution,
//! or runtime orchestration.

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
    SOURCE_METADATA_ACL_TAGS_KEY, SOURCE_METADATA_AMENDMENT_VERSION_KEY,
    SOURCE_METADATA_ARTICLE_KEY, SOURCE_METADATA_AUTHORS_KEY, SOURCE_METADATA_CROP_KEY,
    SOURCE_METADATA_DEMAND_SIGNAL_KEY, SOURCE_METADATA_DEPARTMENT_KEY,
    SOURCE_METADATA_DOCUMENT_KIND_KEY, SOURCE_METADATA_DOI_KEY, SOURCE_METADATA_EFFECTIVE_AT_KEY,
    SOURCE_METADATA_JURISDICTION_KEY, SOURCE_METADATA_LICENSE_KEY, SOURCE_METADATA_LICENSE_URL_KEY,
    SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY, SOURCE_METADATA_MESH_TERMS_KEY,
    SOURCE_METADATA_OWNER_TEAM_KEY, SOURCE_METADATA_PMID_KEY, SOURCE_METADATA_PRICE_DATE_KEY,
    SOURCE_METADATA_PUBLISHED_AT_KEY, SOURCE_METADATA_REGION_KEY, SOURCE_METADATA_REVISION_ID_KEY,
    SOURCE_METADATA_SECTION_KEY, SOURCE_METADATA_STATUTE_KEY, SOURCE_METADATA_SUPPLY_SIGNAL_KEY,
    SOURCE_METADATA_TENANT_ID_KEY, SOURCE_METADATA_TITLE_KEY, SOURCE_METADATA_UPDATED_AT_KEY,
    SOURCE_METADATA_VERSION_KEY, SOURCE_METADATA_WEATHER_WINDOW_KEY, SourceMetadata, TableRef,
};
pub use error::{NexusError, NexusResult};
pub use query::{
    EVIDENCE_KIND_METADATA_KEY, EvidenceConflictMode, EvidenceKind, EvidenceRecord,
    ExternalKnowledgeCompareRequest, ExternalKnowledgeOpenRequest, ExternalKnowledgeRefreshRequest,
    ExternalKnowledgeSearchRequest, ExternalKnowledgeSearchResponse,
};
pub use source::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceKind, NexusSourceRecord,
    SOURCE_PACK_DISPLAY_NAME_METADATA_KEY, SOURCE_PACK_DOMAIN_METADATA_KEY,
    SOURCE_PACK_FIXTURE_PATH_METADATA_KEY, SOURCE_PACK_ID_METADATA_KEY,
    SOURCE_PACK_PRODUCER_METADATA_KEY, SOURCE_PACK_SCHEMA_VERSION_METADATA_KEY,
    SOURCE_PACK_VERSION_METADATA_KEY, SourceCapabilities, SourceChange, SourceCheckpoint,
    SourceCursor, SourceDomain, SourceItemRef,
};
pub use sync::{NexusJobKind, NexusJobRecord, NexusJobStatus};
pub use trust::{EvidenceBoundary, ProvenanceBundle, ProvenanceRecord};
