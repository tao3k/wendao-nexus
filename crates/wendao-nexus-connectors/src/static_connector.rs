//! Static connector used for deterministic `Wendao Nexus` tests.

use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::Utc;
use wendao_nexus_core::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, KnowledgeSourceKind, NexusError,
    NexusResult, RawSourceDocument, SourceCapabilities, SourceCheckpoint, SourceCursor,
    SourceItemRef,
};

/// Deterministic connector for tests and local embedding.
#[derive(Clone, Debug)]
pub struct StaticKnowledgeConnector {
    source_id: String,
    source_kind: KnowledgeSourceKind,
    items: Vec<SourceItemRef>,
    documents: BTreeMap<String, RawSourceDocument>,
}

impl StaticKnowledgeConnector {
    pub fn new(source_id: impl Into<String>, source_kind: KnowledgeSourceKind) -> Self {
        Self {
            source_id: source_id.into(),
            source_kind,
            items: Vec::new(),
            documents: BTreeMap::new(),
        }
    }

    pub fn with_document(
        mut self,
        external_id: impl Into<String>,
        body: impl Into<Vec<u8>>,
    ) -> Self {
        let external_id = external_id.into();
        let canonical_uri = format!("static://{}/{}", self.source_id, external_id);
        let document = RawSourceDocument {
            source_id: self.source_id.clone(),
            external_id: external_id.clone(),
            canonical_uri: canonical_uri.clone(),
            media_type: "text/plain".to_string(),
            payload: body.into(),
            fetched_at: Utc::now(),
            source_updated_at: None,
            content_hash: None,
            metadata: BTreeMap::new(),
        };

        let mut item = SourceItemRef::new(self.source_id.clone(), external_id.clone());
        item.canonical_uri = Some(canonical_uri);
        self.items.push(item);
        self.documents.insert(external_id, document);
        self
    }

    pub fn with_raw_document(mut self, document: RawSourceDocument) -> Self {
        let mut item = SourceItemRef::new(document.source_id.clone(), document.external_id.clone());
        item.canonical_uri = Some(document.canonical_uri.clone());
        self.items.push(item);
        self.documents
            .insert(document.external_id.clone(), document);
        self
    }
}

#[async_trait]
impl KnowledgeSourceConnector for StaticKnowledgeConnector {
    fn source_id(&self) -> &str {
        &self.source_id
    }

    fn source_kind(&self) -> KnowledgeSourceKind {
        self.source_kind.clone()
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::mirror_fetch()
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Ok(DiscoveryBatch {
            source_id: self.source_id.clone(),
            items: self.items.clone(),
            next_cursor: None,
            observed_at: Utc::now(),
        })
    }

    async fn fetch(&self, item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        self.documents
            .get(&item.external_id)
            .cloned()
            .ok_or_else(|| NexusError::NotFound {
                source_id: self.source_id.clone(),
                external_id: item.external_id,
            })
    }

    async fn delta(&self, since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        Ok(DeltaBatch {
            source_id: self.source_id.clone(),
            changes: Vec::new(),
            next_checkpoint: since,
            observed_at: Utc::now(),
        })
    }
}
