//! Raw and normalized document contracts for external knowledge ingestion.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::query::{EVIDENCE_KIND_METADATA_KEY, EvidenceKind};
use crate::trust::ProvenanceRecord;

/// Standard raw metadata key for document titles.
pub const SOURCE_METADATA_TITLE_KEY: &str = "title";
/// Standard raw metadata key for author lists.
pub const SOURCE_METADATA_AUTHORS_KEY: &str = "authors";
/// Standard raw metadata key for publication timestamps.
pub const SOURCE_METADATA_PUBLISHED_AT_KEY: &str = "published_at";
/// Standard raw metadata key for source update timestamps.
pub const SOURCE_METADATA_UPDATED_AT_KEY: &str = "updated_at";
/// Standard raw metadata key for DOI identifiers.
pub const SOURCE_METADATA_DOI_KEY: &str = "doi";
/// Standard raw metadata key for PMID identifiers.
pub const SOURCE_METADATA_PMID_KEY: &str = "pmid";
/// Standard raw metadata key for MeSH term lists.
pub const SOURCE_METADATA_MESH_TERMS_KEY: &str = "mesh_terms";
/// Standard raw metadata key for legal jurisdictions.
pub const SOURCE_METADATA_JURISDICTION_KEY: &str = "jurisdiction";
/// Standard raw metadata key for private-corpus tenant ids.
pub const SOURCE_METADATA_TENANT_ID_KEY: &str = "tenant_id";
/// Standard raw metadata key for private-corpus departments.
pub const SOURCE_METADATA_DEPARTMENT_KEY: &str = "department";
/// Standard raw metadata key for source document kinds.
pub const SOURCE_METADATA_DOCUMENT_KIND_KEY: &str = "document_kind";
/// Standard raw metadata key for source owner teams.
pub const SOURCE_METADATA_OWNER_TEAM_KEY: &str = "owner_team";
/// Standard raw metadata key for private-corpus ACL tag lists.
pub const SOURCE_METADATA_ACL_TAGS_KEY: &str = "acl_tags";
/// Standard raw metadata key for source versions.
pub const SOURCE_METADATA_VERSION_KEY: &str = "version";
/// Standard raw metadata key for source revision ids.
pub const SOURCE_METADATA_REVISION_ID_KEY: &str = "revision_id";
/// Standard raw metadata key for license names.
pub const SOURCE_METADATA_LICENSE_KEY: &str = "license";
/// Standard raw metadata key for license URLs.
pub const SOURCE_METADATA_LICENSE_URL_KEY: &str = "license_url";
/// Standard raw metadata key for license usage policies.
pub const SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY: &str = "license_usage_policy";
/// Standard raw metadata key for domain effective timestamps.
pub const SOURCE_METADATA_EFFECTIVE_AT_KEY: &str = "effective_at";
/// Standard raw metadata key for legal statute names.
pub const SOURCE_METADATA_STATUTE_KEY: &str = "statute";
/// Standard raw metadata key for legal article identifiers.
pub const SOURCE_METADATA_ARTICLE_KEY: &str = "article";
/// Standard raw metadata key for legal section labels.
pub const SOURCE_METADATA_SECTION_KEY: &str = "section";
/// Standard raw metadata key for legal amendment versions.
pub const SOURCE_METADATA_AMENDMENT_VERSION_KEY: &str = "amendment_version";
/// Standard raw metadata key for agriculture or market regions.
pub const SOURCE_METADATA_REGION_KEY: &str = "region";
/// Standard raw metadata key for agriculture crop identifiers.
pub const SOURCE_METADATA_CROP_KEY: &str = "crop";
/// Standard raw metadata key for agriculture market price dates.
pub const SOURCE_METADATA_PRICE_DATE_KEY: &str = "price_date";
/// Standard raw metadata key for agriculture weather windows.
pub const SOURCE_METADATA_WEATHER_WINDOW_KEY: &str = "weather_window";
/// Standard raw metadata key for agriculture supply signals.
pub const SOURCE_METADATA_SUPPLY_SIGNAL_KEY: &str = "supply_signal";
/// Standard raw metadata key for agriculture demand signals.
pub const SOURCE_METADATA_DEMAND_SIGNAL_KEY: &str = "demand_signal";

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

impl RawSourceDocument {
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    pub fn title_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_TITLE_KEY)
    }

    pub fn authors_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_AUTHORS_KEY)
    }

    pub fn published_at_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_PUBLISHED_AT_KEY)
    }

    pub fn updated_at_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_UPDATED_AT_KEY)
    }

    pub fn doi_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_DOI_KEY)
    }

    pub fn pmid_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_PMID_KEY)
    }

    pub fn mesh_terms_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_MESH_TERMS_KEY)
    }

    pub fn jurisdiction_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_JURISDICTION_KEY)
    }

    pub fn tenant_id_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_TENANT_ID_KEY)
    }

    pub fn department_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_DEPARTMENT_KEY)
    }

    pub fn document_kind_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_DOCUMENT_KIND_KEY)
    }

    pub fn owner_team_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_OWNER_TEAM_KEY)
    }

    pub fn acl_tags_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_ACL_TAGS_KEY)
    }

    pub fn version_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_VERSION_KEY)
    }

    pub fn revision_id_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_REVISION_ID_KEY)
    }

    pub fn license_name_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_LICENSE_KEY)
    }

    pub fn license_url_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_LICENSE_URL_KEY)
    }

    pub fn license_usage_policy_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY)
    }

    pub fn effective_at_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_EFFECTIVE_AT_KEY)
    }

    pub fn statute_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_STATUTE_KEY)
    }

    pub fn article_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_ARTICLE_KEY)
    }

    pub fn section_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_SECTION_KEY)
    }

    pub fn amendment_version_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_AMENDMENT_VERSION_KEY)
    }

    pub fn region_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_REGION_KEY)
    }

    pub fn crop_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_CROP_KEY)
    }

    pub fn price_date_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_PRICE_DATE_KEY)
    }

    pub fn weather_window_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_WEATHER_WINDOW_KEY)
    }

    pub fn supply_signal_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_SUPPLY_SIGNAL_KEY)
    }

    pub fn demand_signal_metadata(&self) -> Option<&str> {
        self.metadata_value(SOURCE_METADATA_DEMAND_SIGNAL_KEY)
    }
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

/// Normalized external document accepted for Wendao-side evidence handoff.
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

/// Section-level unit for downstream search, citation, and evidence display.
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

impl SourceMetadata {
    pub fn extra_value(&self, key: &str) -> Option<&str> {
        self.extra.get(key).map(String::as_str)
    }

    pub fn evidence_kind(&self) -> EvidenceKind {
        self.extra
            .get(EVIDENCE_KIND_METADATA_KEY)
            .map(EvidenceKind::from_label)
            .unwrap_or_default()
    }

    pub fn title_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_TITLE_KEY)
    }

    pub fn version_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_VERSION_KEY)
    }

    pub fn revision_id_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_REVISION_ID_KEY)
    }

    pub fn license_name_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_LICENSE_KEY)
    }

    pub fn license_url_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_LICENSE_URL_KEY)
    }

    pub fn license_usage_policy_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY)
    }

    pub fn effective_at_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_EFFECTIVE_AT_KEY)
    }

    pub fn statute_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_STATUTE_KEY)
    }

    pub fn article_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_ARTICLE_KEY)
    }

    pub fn section_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_SECTION_KEY)
    }

    pub fn amendment_version_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_AMENDMENT_VERSION_KEY)
    }

    pub fn department_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_DEPARTMENT_KEY)
    }

    pub fn document_kind_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_DOCUMENT_KIND_KEY)
    }

    pub fn owner_team_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_OWNER_TEAM_KEY)
    }

    pub fn region_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_REGION_KEY)
    }

    pub fn crop_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_CROP_KEY)
    }

    pub fn price_date_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_PRICE_DATE_KEY)
    }

    pub fn weather_window_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_WEATHER_WINDOW_KEY)
    }

    pub fn supply_signal_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_SUPPLY_SIGNAL_KEY)
    }

    pub fn demand_signal_metadata(&self) -> Option<&str> {
        self.extra_value(SOURCE_METADATA_DEMAND_SIGNAL_KEY)
    }
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
