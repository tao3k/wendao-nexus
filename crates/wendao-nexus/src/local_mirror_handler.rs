//! Local mirror implementation of the Nexus Flight command handler.

use std::collections::BTreeSet;

use async_trait::async_trait;
use wendao_nexus_core::{
    EvidenceBoundary, ExternalKnowledgeCompareRequest, ExternalKnowledgeOpenRequest,
    ExternalKnowledgeSearchRequest,
};
use wendao_nexus_flight::{
    open_rows_from_document, search_rows_from_response, FlightCompareResultRow,
    FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow, FlightSyncResultRow,
    NexusFlightCommandHandler, NexusFlightHandlerError, NexusFlightStatusRequest,
    NexusFlightSyncRequest,
};
use wendao_nexus_runtime::{CheckpointRegistry, LocalKnowledgeStore};

/// Flight command handler backed by a local mirror store and checkpoint registry.
///
/// This is intended for Wendao-side mounting and tests. It does not schedule
/// live connector work; callers that need sync should wire their own handler
/// around `NexusSyncRuntime` and source-specific connector ownership.
#[derive(Clone, Debug)]
pub struct LocalMirrorFlightHandler<S, R> {
    store: S,
    checkpoint_registry: R,
}

impl<S, R> LocalMirrorFlightHandler<S, R> {
    pub fn new(store: S, checkpoint_registry: R) -> Self {
        Self {
            store,
            checkpoint_registry,
        }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn checkpoint_registry(&self) -> &R {
        &self.checkpoint_registry
    }
}

#[async_trait]
impl<S, R> NexusFlightCommandHandler for LocalMirrorFlightHandler<S, R>
where
    S: LocalKnowledgeStore,
    R: CheckpointRegistry,
{
    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> Result<Vec<FlightSearchResultRow>, NexusFlightHandlerError> {
        let response = self.store.search(request).await.map_err(handler_error)?;
        search_rows_from_response(&response).map_err(handler_error)
    }

    async fn open(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> Result<Vec<FlightOpenDocumentRow>, NexusFlightHandlerError> {
        let include_sections = request.include_sections;
        let include_provenance = request.include_provenance;
        let document = self
            .store
            .open_document(request)
            .await
            .map_err(handler_error)?;
        open_rows_from_document(&document, include_sections, include_provenance)
            .map_err(handler_error)
    }

    async fn sync(
        &self,
        request: NexusFlightSyncRequest,
    ) -> Result<Vec<FlightSyncResultRow>, NexusFlightHandlerError> {
        Err(NexusFlightHandlerError::message(format!(
            "source `{}` sync requires Wendao-side connector runtime ownership",
            request.source_id
        )))
    }

    async fn status(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<FlightStatusRow>, NexusFlightHandlerError> {
        let source_ids = self.status_source_ids(request).await?;
        let mut rows = Vec::with_capacity(source_ids.len());

        for source_id in source_ids {
            match self
                .checkpoint_registry
                .get_checkpoint(&source_id)
                .await
                .map_err(handler_error)?
            {
                Some(checkpoint) => rows.push(FlightStatusRow::from(&checkpoint)),
                None => rows.push(FlightStatusRow {
                    source_id,
                    enabled: true,
                    last_success_at: None,
                    last_seen_revision: None,
                    last_content_hash: None,
                    rate_limit_state: None,
                }),
            }
        }

        Ok(rows)
    }

    async fn compare(
        &self,
        request: ExternalKnowledgeCompareRequest,
    ) -> Result<Vec<FlightCompareResultRow>, NexusFlightHandlerError> {
        let mut search = ExternalKnowledgeSearchRequest::new(request.claim.clone());
        search.sources = request.sources;
        let minimum_authority = request.trust_policy.minimum_authority;
        search.trust_policy = request.trust_policy;
        search.limit = 20;

        let response = self.store.search(search).await.map_err(handler_error)?;
        let evidence_records = response
            .records
            .iter()
            .map(|record| record.provenance.primary.clone())
            .collect::<Vec<_>>();
        let insufficient_authority = response.records.is_empty();
        let boundary = EvidenceBoundary {
            records: evidence_records,
            minimum_authority,
            insufficient_authority,
            stale_evidence: false,
            conflict_detected: false,
        };

        Ok(vec![FlightCompareResultRow {
            claim: request.claim,
            verdict: if insufficient_authority {
                "insufficient_authority".to_string()
            } else {
                "evidence_available".to_string()
            },
            conflict_detected: boundary.conflict_detected,
            insufficient_authority: boundary.insufficient_authority,
            stale_evidence: boundary.stale_evidence,
            provenance_json: Some(serde_json::to_string(&boundary).map_err(handler_error)?),
        }])
    }
}

impl<S, R> LocalMirrorFlightHandler<S, R>
where
    S: LocalKnowledgeStore,
    R: CheckpointRegistry,
{
    async fn status_source_ids(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<String>, NexusFlightHandlerError> {
        if !request.sources.is_empty() {
            return Ok(request.sources);
        }

        let source_ids = self
            .store
            .list_documents(None)
            .await
            .map_err(handler_error)?
            .into_iter()
            .map(|document| document.source_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(source_ids)
    }
}

fn handler_error(error: impl std::fmt::Display) -> NexusFlightHandlerError {
    NexusFlightHandlerError::message(error.to_string())
}
