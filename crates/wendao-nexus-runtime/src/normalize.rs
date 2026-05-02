//! Normalization contracts for turning raw source payloads into documents.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use wendao_nexus_core::{
    AuthorityLevel, ExternalKnowledgeDocument, KnowledgeSection, KnowledgeSourceKind, LicenseInfo,
    NexusError, NexusResult, ProvenanceRecord, RawSourceDocument, SourceMetadata,
};

use crate::hash::sha256_content_hash;

/// Context supplied by the caller that knows source policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationContext {
    pub source_kind: KnowledgeSourceKind,
    pub authority_level: AuthorityLevel,
    pub title: Option<String>,
}

impl NormalizationContext {
    pub fn new(source_kind: KnowledgeSourceKind, authority_level: AuthorityLevel) -> Self {
        Self {
            source_kind,
            authority_level,
            title: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Normalizer boundary for source-specific extraction and metadata shaping.
#[async_trait]
pub trait KnowledgeDocumentNormalizer: Send + Sync {
    async fn normalize(
        &self,
        raw: RawSourceDocument,
        context: NormalizationContext,
    ) -> NexusResult<ExternalKnowledgeDocument>;
}

/// Deterministic UTF-8 text normalizer for tests and simple customer corpora.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlainTextNormalizer;

#[async_trait]
impl KnowledgeDocumentNormalizer for PlainTextNormalizer {
    async fn normalize(
        &self,
        raw: RawSourceDocument,
        context: NormalizationContext,
    ) -> NexusResult<ExternalKnowledgeDocument> {
        if !is_text_media_type(&raw.media_type) {
            return Err(NexusError::Unsupported {
                source_id: raw.source_id,
                operation: "plain text normalization",
            });
        }

        let body = String::from_utf8(raw.payload.clone()).map_err(|error| {
            NexusError::Normalize(format!(
                "source `{}` item `{}` is not valid UTF-8: {error}",
                raw.source_id, raw.external_id
            ))
        })?;
        let content_hash = raw
            .content_hash
            .clone()
            .unwrap_or_else(|| sha256_content_hash(&raw.payload));
        let metadata = source_metadata_from_raw(&raw);
        let title = context
            .title
            .clone()
            .or_else(|| raw.metadata.get("title").cloned())
            .unwrap_or_else(|| raw.external_id.clone());
        let provenance = provenance_from_raw(&raw, &context, &metadata, &content_hash);
        let license = license_from_raw(&raw);

        Ok(ExternalKnowledgeDocument {
            source_id: raw.source_id,
            external_id: raw.external_id,
            canonical_uri: raw.canonical_uri,
            title: title.clone(),
            body: body.clone(),
            sections: vec![KnowledgeSection {
                section_id: "body".to_string(),
                heading_path: vec![title],
                text: body,
                anchors: Vec::new(),
                citations: Vec::new(),
                tables: Vec::new(),
                figures: Vec::new(),
            }],
            metadata,
            provenance,
            license,
            fetched_at: raw.fetched_at,
            source_updated_at: raw.source_updated_at,
            content_hash,
        })
    }
}

fn is_text_media_type(media_type: &str) -> bool {
    matches!(
        media_type,
        "text/plain" | "text/markdown" | "text/html" | "application/json"
    )
}

fn source_metadata_from_raw(raw: &RawSourceDocument) -> SourceMetadata {
    SourceMetadata {
        authors: raw
            .metadata
            .get("authors")
            .map(|authors| split_list(authors))
            .unwrap_or_default(),
        published_at: raw
            .metadata
            .get("published_at")
            .and_then(|value| parse_rfc3339_utc(value)),
        updated_at: raw.source_updated_at.or_else(|| {
            raw.metadata
                .get("updated_at")
                .and_then(|value| parse_rfc3339_utc(value))
        }),
        doi: raw.metadata.get("doi").cloned(),
        pmid: raw.metadata.get("pmid").cloned(),
        mesh_terms: raw
            .metadata
            .get("mesh_terms")
            .map(|mesh_terms| split_list(mesh_terms))
            .unwrap_or_default(),
        jurisdiction: raw.metadata.get("jurisdiction").cloned(),
        tenant_id: raw.metadata.get("tenant_id").cloned(),
        acl_tags: raw
            .metadata
            .get("acl_tags")
            .map(|acl_tags| split_list(acl_tags))
            .unwrap_or_default(),
        extra: raw.metadata.clone(),
    }
}

fn provenance_from_raw(
    raw: &RawSourceDocument,
    context: &NormalizationContext,
    metadata: &SourceMetadata,
    content_hash: &str,
) -> ProvenanceRecord {
    ProvenanceRecord {
        source_id: raw.source_id.clone(),
        source_kind: context.source_kind.clone(),
        authority_level: context.authority_level,
        canonical_uri: raw.canonical_uri.clone(),
        version: raw.metadata.get("version").cloned(),
        revision_id: raw.metadata.get("revision_id").cloned(),
        doi: metadata.doi.clone(),
        pmid: metadata.pmid.clone(),
        jurisdiction: metadata.jurisdiction.clone(),
        published_at: metadata.published_at,
        fetched_at: raw.fetched_at,
        content_hash: content_hash.to_string(),
        trust_signals: Vec::new(),
    }
}

fn license_from_raw(raw: &RawSourceDocument) -> Option<LicenseInfo> {
    raw.metadata.get("license").map(|name| LicenseInfo {
        name: name.clone(),
        url: raw.metadata.get("license_url").cloned(),
        usage_policy: raw.metadata.get("license_usage_policy").cloned(),
    })
}

fn parse_rfc3339_utc(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
