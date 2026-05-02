//! Local mirror query facade for normalized external knowledge documents.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use wendao_nexus_core::{
    EvidenceRecord, ExternalKnowledgeDocument, ExternalKnowledgeOpenRequest,
    ExternalKnowledgeSearchRequest, ExternalKnowledgeSearchResponse, NexusError, NexusResult,
    ProvenanceBundle, TrustPolicy,
};

/// Query and open facade for normalized documents already accepted into Nexus.
#[async_trait]
pub trait LocalKnowledgeStore: Send + Sync {
    async fn upsert_document(
        &self,
        document: ExternalKnowledgeDocument,
    ) -> NexusResult<ExternalKnowledgeDocument>;

    async fn get_document(
        &self,
        source_id: &str,
        external_id: &str,
    ) -> NexusResult<Option<ExternalKnowledgeDocument>>;

    async fn list_documents(
        &self,
        source_id: Option<&str>,
    ) -> NexusResult<Vec<ExternalKnowledgeDocument>>;

    async fn open_document(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> NexusResult<ExternalKnowledgeDocument>;

    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> NexusResult<ExternalKnowledgeSearchResponse>;
}

/// Deterministic in-memory local mirror for tests and early Wendao embedding.
#[derive(Clone, Default)]
pub struct InMemoryKnowledgeStore {
    inner: Arc<RwLock<KnowledgeStoreState>>,
}

#[derive(Default)]
struct KnowledgeStoreState {
    documents: BTreeMap<DocumentKey, ExternalKnowledgeDocument>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DocumentKey {
    source_id: String,
    external_id: String,
}

impl DocumentKey {
    fn new(source_id: impl Into<String>, external_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            external_id: external_id.into(),
        }
    }
}

impl InMemoryKnowledgeStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn read_state(&self) -> NexusResult<std::sync::RwLockReadGuard<'_, KnowledgeStoreState>> {
        self.inner.read().map_err(|_| {
            NexusError::Registry("in-memory knowledge store read lock poisoned".into())
        })
    }

    fn write_state(&self) -> NexusResult<std::sync::RwLockWriteGuard<'_, KnowledgeStoreState>> {
        self.inner.write().map_err(|_| {
            NexusError::Registry("in-memory knowledge store write lock poisoned".into())
        })
    }

    fn all_documents(&self) -> NexusResult<Vec<ExternalKnowledgeDocument>> {
        let state = self.read_state()?;
        Ok(state.documents.values().cloned().collect())
    }
}

#[async_trait]
impl LocalKnowledgeStore for InMemoryKnowledgeStore {
    async fn upsert_document(
        &self,
        document: ExternalKnowledgeDocument,
    ) -> NexusResult<ExternalKnowledgeDocument> {
        let mut state = self.write_state()?;
        let key = DocumentKey::new(document.source_id.clone(), document.external_id.clone());
        state.documents.insert(key, document.clone());
        Ok(document)
    }

    async fn get_document(
        &self,
        source_id: &str,
        external_id: &str,
    ) -> NexusResult<Option<ExternalKnowledgeDocument>> {
        let state = self.read_state()?;
        let key = DocumentKey::new(source_id, external_id);
        Ok(state.documents.get(&key).cloned())
    }

    async fn list_documents(
        &self,
        source_id: Option<&str>,
    ) -> NexusResult<Vec<ExternalKnowledgeDocument>> {
        let documents = self
            .all_documents()?
            .into_iter()
            .filter(|document| match source_id {
                Some(source_id) => document.source_id == source_id,
                None => true,
            })
            .collect();
        Ok(documents)
    }

    async fn open_document(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> NexusResult<ExternalKnowledgeDocument> {
        let mut document = self
            .get_document(&request.source_id, &request.external_id)
            .await?
            .ok_or(NexusError::NotFound {
                source_id: request.source_id,
                external_id: request.external_id,
            })?;

        if !request.include_sections {
            document.sections.clear();
        }

        Ok(document)
    }

    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> NexusResult<ExternalKnowledgeSearchResponse> {
        let mut records = self
            .all_documents()?
            .into_iter()
            .filter(|document| source_filter_allows(document, &request.sources))
            .filter(|document| trust_policy_allows(document, &request.trust_policy))
            .filter(|document| freshness_filter_allows(document, request.freshness_days))
            .filter(|document| document_matches_query(document, &request.query))
            .map(|document| evidence_record_from_document(&document, &request.query))
            .collect::<Vec<_>>();

        records.sort_by(|left, right| {
            right
                .provenance
                .primary
                .authority_level
                .cmp(&left.provenance.primary.authority_level)
                .then_with(|| left.source_id.cmp(&right.source_id))
                .then_with(|| left.external_id.cmp(&right.external_id))
        });
        records.truncate(request.limit);

        Ok(ExternalKnowledgeSearchResponse {
            query: request.query,
            records,
            generated_at: Utc::now(),
        })
    }
}

fn source_filter_allows(document: &ExternalKnowledgeDocument, sources: &[String]) -> bool {
    sources.is_empty()
        || sources
            .iter()
            .any(|source_id| source_id == &document.source_id)
}

fn trust_policy_allows(document: &ExternalKnowledgeDocument, policy: &TrustPolicy) -> bool {
    let authority = document.provenance.authority_level;
    authority >= policy.minimum_authority
        && (policy.allow_community_sources
            || authority != wendao_nexus_core::AuthorityLevel::Community)
}

fn freshness_filter_allows(
    document: &ExternalKnowledgeDocument,
    freshness_days: Option<u32>,
) -> bool {
    match freshness_days {
        Some(days) => document.fetched_at >= Utc::now() - Duration::days(days.into()),
        None => true,
    }
}

fn document_matches_query(document: &ExternalKnowledgeDocument, query: &str) -> bool {
    let terms = normalized_terms(query);
    if terms.is_empty() {
        return true;
    }

    let haystack = normalized_document_text(document);
    terms.iter().all(|term| haystack.contains(term))
}

fn evidence_record_from_document(
    document: &ExternalKnowledgeDocument,
    query: &str,
) -> EvidenceRecord {
    EvidenceRecord {
        source_id: document.source_id.clone(),
        external_id: document.external_id.clone(),
        title: document.title.clone(),
        snippet: best_snippet(document, query),
        score: Some("1.0".to_string()),
        provenance: ProvenanceBundle {
            primary: document.provenance.clone(),
            corroborating: Vec::new(),
            conflicting: Vec::new(),
        },
    }
}

fn best_snippet(document: &ExternalKnowledgeDocument, query: &str) -> String {
    let terms = normalized_terms(query);
    let candidate = document
        .sections
        .iter()
        .map(|section| section.text.as_str())
        .find(|text| {
            let normalized = normalize_text(text);
            terms.iter().all(|term| normalized.contains(term))
        })
        .unwrap_or(&document.body);

    truncate_snippet(candidate)
}

fn normalized_document_text(document: &ExternalKnowledgeDocument) -> String {
    let mut text = format!("{} {}", document.title, document.body);
    for section in &document.sections {
        text.push(' ');
        text.push_str(&section.heading_path.join(" "));
        text.push(' ');
        text.push_str(&section.text);
    }
    normalize_text(&text)
}

fn normalized_terms(query: &str) -> Vec<String> {
    normalize_text(query)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
}

fn truncate_snippet(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(240).collect()
}
