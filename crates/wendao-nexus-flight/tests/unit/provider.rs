use arrow_array::StringArray;
use async_trait::async_trait;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceConflictMode, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest, TrustPolicy,
};
use wendao_nexus_flight::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow,
};
use wendao_nexus_flight::{
    NexusFlightBatchProvider, NexusFlightCommand, NexusFlightCommandHandler,
    NexusFlightHandlerError, NexusFlightProviderError, NexusFlightStatusRequest,
    NexusFlightSyncRequest,
};

#[derive(Clone, Debug)]
struct StubHandler;

#[async_trait]
impl NexusFlightCommandHandler for StubHandler {
    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> Result<Vec<FlightSearchResultRow>, NexusFlightHandlerError> {
        Ok(vec![FlightSearchResultRow {
            source_id: request
                .sources
                .first()
                .cloned()
                .unwrap_or_else(|| "local".to_string()),
            external_id: "doc-1".to_string(),
            title: request.query,
            snippet: Some("evidence snippet".to_string()),
            score: Some(0.9),
            authority_level: "PeerReviewed".to_string(),
            canonical_uri: "https://example.test/doc-1".to_string(),
            fetched_at: None,
            content_hash: "sha256:doc-1".to_string(),
            provenance_json: None,
        }])
    }

    async fn open(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> Result<Vec<FlightOpenDocumentRow>, NexusFlightHandlerError> {
        Ok(vec![FlightOpenDocumentRow {
            source_id: request.source_id,
            external_id: request.external_id,
            canonical_uri: "https://example.test/doc-1".to_string(),
            title: "Document".to_string(),
            section_id: None,
            heading_path_json: None,
            body: Some("body".to_string()),
            metadata_json: None,
            provenance_json: None,
        }])
    }

    async fn sync(
        &self,
        request: NexusFlightSyncRequest,
    ) -> Result<Vec<FlightSyncResultRow>, NexusFlightHandlerError> {
        Ok(vec![FlightSyncResultRow {
            job_id: "job-1".to_string(),
            source_id: request.source_id,
            job_kind: "Refresh".to_string(),
            status: "Succeeded".to_string(),
            cursor: None,
            dedup_hit: false,
            error: None,
        }])
    }

    async fn status(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<FlightStatusRow>, NexusFlightHandlerError> {
        Ok(request
            .sources
            .into_iter()
            .map(|source_id| FlightStatusRow {
                source_id,
                enabled: true,
                last_success_at: None,
                last_seen_revision: None,
                last_content_hash: None,
                rate_limit_state: None,
            })
            .collect())
    }

    async fn compare(
        &self,
        request: ExternalKnowledgeCompareRequest,
    ) -> Result<Vec<FlightCompareResultRow>, NexusFlightHandlerError> {
        Ok(vec![FlightCompareResultRow {
            claim: request.claim,
            verdict: "needs_review".to_string(),
            conflict_detected: false,
            insufficient_authority: false,
            stale_evidence: false,
            provenance_json: None,
        }])
    }
}

#[derive(Clone, Debug)]
struct FailingHandler;

#[async_trait]
impl NexusFlightCommandHandler for FailingHandler {
    async fn search(
        &self,
        _request: ExternalKnowledgeSearchRequest,
    ) -> Result<Vec<FlightSearchResultRow>, NexusFlightHandlerError> {
        Err(NexusFlightHandlerError::message("search unavailable"))
    }

    async fn open(
        &self,
        _request: ExternalKnowledgeOpenRequest,
    ) -> Result<Vec<FlightOpenDocumentRow>, NexusFlightHandlerError> {
        Err("open unavailable".into())
    }

    async fn sync(
        &self,
        _request: NexusFlightSyncRequest,
    ) -> Result<Vec<FlightSyncResultRow>, NexusFlightHandlerError> {
        Err("sync unavailable".into())
    }

    async fn status(
        &self,
        _request: NexusFlightStatusRequest,
    ) -> Result<Vec<FlightStatusRow>, NexusFlightHandlerError> {
        Err("status unavailable".into())
    }

    async fn compare(
        &self,
        _request: ExternalKnowledgeCompareRequest,
    ) -> Result<Vec<FlightCompareResultRow>, NexusFlightHandlerError> {
        Err("compare unavailable".into())
    }
}

#[tokio::test]
async fn provider_dispatches_descriptor_to_search_batch() {
    let mut request = ExternalKnowledgeSearchRequest::new("GLP-1 evidence");
    request.sources = vec!["pubmed".to_string()];
    request.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::PeerReviewed);

    let command = NexusFlightCommand::Search(request);
    let descriptor = command.to_descriptor().unwrap();
    let provider = NexusFlightBatchProvider::new(StubHandler);
    let batch = provider.handle_descriptor(&descriptor).await.unwrap();

    let source_ids = batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let titles = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(source_ids.value(0), "pubmed");
    assert_eq!(titles.value(0), "GLP-1 evidence");
}

#[tokio::test]
async fn provider_preserves_route_specific_status_shape() {
    let command = NexusFlightCommand::Status(NexusFlightStatusRequest {
        sources: vec!["wikipedia".to_string(), "pubmed".to_string()],
    });
    let provider = NexusFlightBatchProvider::new(StubHandler);
    let batch = provider.handle_command(command).await.unwrap();

    let source_ids = batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().field(1).name(), "enabled");
    assert_eq!(source_ids.value(1), "pubmed");
}

#[tokio::test]
async fn provider_surfaces_handler_errors_without_server_policy() {
    let command = NexusFlightCommand::Compare(ExternalKnowledgeCompareRequest {
        claim: "claim".to_string(),
        sources: Vec::new(),
        mode: EvidenceConflictMode::EvidenceConflictCheck,
        trust_policy: TrustPolicy::default(),
    });
    let provider = NexusFlightBatchProvider::new(FailingHandler);
    let error = provider.handle_command(command).await.unwrap_err();

    assert!(matches!(error, NexusFlightProviderError::Handler(_)));
}
