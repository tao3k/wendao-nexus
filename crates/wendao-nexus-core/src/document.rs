//! Raw and normalized document contracts for external knowledge ingestion.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::trust::ProvenanceRecord;

/// Raw payload fetched from an external source before normalization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawSourceDocument {
    pub source_id: String,
    pub external_id: String,
    pub canonical_uri: String,
    pub media_type: String,
    pub payload: Vec<u8>,
    pub fetched_at: DateTime<Utc>,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub content_hash: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

/// Structured resource set emitted by document extraction systems.
///
/// The shape intentionally mirrors Wendao's document extraction resource payload
/// without depending on `xiuxian-*` crates. Nexus treats this as a protocol
/// handoff shape; Wendao owns Docling execution, parsing, scheduling, and agent
/// routing.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedDocumentResourceSet {
    pub source_path: String,
    pub source_format: String,
    #[serde(default)]
    pub total_resources: usize,
    #[serde(default)]
    pub total_pages: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_at: Option<i64>,
    #[serde(default)]
    pub resources: Vec<ExtractedDocumentResource>,
}

/// One extracted document resource row.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedDocumentResource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default)]
    pub resource_type: String,
    #[serde(default)]
    pub resource_path: String,
    #[serde(default)]
    pub page_index: usize,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub element_id: String,
}

/// Normalized external document accepted by Wendao index planes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalKnowledgeDocument {
    pub source_id: String,
    pub external_id: String,
    pub canonical_uri: String,
    pub title: String,
    pub body: String,
    pub sections: Vec<KnowledgeSection>,
    pub metadata: SourceMetadata,
    pub provenance: ProvenanceRecord,
    pub license: Option<LicenseInfo>,
    pub fetched_at: DateTime<Utc>,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub content_hash: String,
}

/// Section-level unit for search, citation, graph expansion, and reranking.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSection {
    pub section_id: String,
    pub heading_path: Vec<String>,
    pub text: String,
    pub anchors: Vec<String>,
    pub citations: Vec<CitationRef>,
    pub tables: Vec<TableRef>,
    pub figures: Vec<FigureRef>,
}

/// Source metadata that should survive normalization.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub authors: Vec<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub mesh_terms: Vec<String>,
    pub jurisdiction: Option<String>,
    pub tenant_id: Option<String>,
    pub acl_tags: Vec<String>,
    pub extra: BTreeMap<String, String>,
}

/// Citation reference found inside a section.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CitationRef {
    pub citation_id: String,
    pub label: Option<String>,
    pub target_uri: Option<String>,
}

/// Table reference found inside a section.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TableRef {
    pub table_id: String,
    pub caption: Option<String>,
}

/// Figure reference found inside a section.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FigureRef {
    pub figure_id: String,
    pub caption: Option<String>,
}

/// License metadata attached to a document or source pack.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub name: String,
    pub url: Option<String>,
    pub usage_policy: Option<String>,
}
