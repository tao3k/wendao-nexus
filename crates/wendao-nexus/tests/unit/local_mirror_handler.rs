use arrow_array::{BooleanArray, StringArray};
use wendao_nexus::LocalMirrorFlightHandler;
use wendao_nexus_connectors::StaticKnowledgeConnector;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceBoundary, EvidenceConflictMode, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest, KnowledgeSourceKind,
    SourceItemRef, TrustPolicy,
};
use wendao_nexus_flight::{
    NexusFlightBatchProvider, NexusFlightCommand, NexusFlightProviderError,
    NexusFlightStatusRequest, NexusFlightSyncRequest,
};
use wendao_nexus_runtime::{
    InMemoryKnowledgeStore, InMemoryNexusRegistry, NexusSyncRuntime, NormalizationContext,
    PlainTextNormalizer,
};

#[tokio::test]
async fn local_mirror_handler_serves_search_open_and_status_batches() {
    let (handler, content_hash) = seeded_handler().await;
    let provider = NexusFlightBatchProvider::new(handler);

    let mut search = ExternalKnowledgeSearchRequest::new("bounded evidence");
    search.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::Curated);
    let search_batch = provider
        .handle_command(NexusFlightCommand::Search(search))
        .await
        .unwrap();
    let titles = string_column(&search_batch, 2);
    assert_eq!(titles.value(0), "doc-1");

    let open_batch = provider
        .handle_command(NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
            source_id: "fixture".to_string(),
            external_id: "doc-1".to_string(),
            include_sections: true,
            include_provenance: true,
        }))
        .await
        .unwrap();
    let section_ids = string_column(&open_batch, 4);
    assert_eq!(section_ids.value(0), "body");
    assert!(!open_batch.column(8).is_null(0));

    let status_batch = provider
        .handle_command(NexusFlightCommand::Status(NexusFlightStatusRequest {
            sources: vec!["fixture".to_string()],
        }))
        .await
        .unwrap();
    let hashes = string_column(&status_batch, 4);
    assert_eq!(hashes.value(0), content_hash);
}

#[tokio::test]
async fn local_mirror_handler_compares_claims_against_local_evidence() {
    let (handler, _) = seeded_handler().await;
    let provider = NexusFlightBatchProvider::new(handler);

    let batch = provider
        .handle_command(NexusFlightCommand::Compare(
            ExternalKnowledgeCompareRequest {
                claim: "bounded evidence".to_string(),
                sources: vec!["fixture".to_string()],
                mode: EvidenceConflictMode::EvidenceConflictCheck,
                trust_policy: TrustPolicy::authority_at_least(AuthorityLevel::Curated),
            },
        ))
        .await
        .unwrap();

    let verdicts = string_column(&batch, 1);
    let insufficient_authority = batch
        .column(3)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();

    assert_eq!(verdicts.value(0), "evidence_available");
    assert!(!insufficient_authority.value(0));
    assert!(!batch.column(5).is_null(0));

    let provenance_json = string_column(&batch, 5);
    let boundary: EvidenceBoundary = serde_json::from_str(provenance_json.value(0)).unwrap();
    assert_eq!(boundary.minimum_authority, AuthorityLevel::Curated);
}

#[tokio::test]
async fn local_mirror_handler_leaves_sync_to_wendao_server_runtime() {
    let (handler, _) = seeded_handler().await;
    let provider = NexusFlightBatchProvider::new(handler);

    let error = provider
        .handle_command(NexusFlightCommand::Sync(NexusFlightSyncRequest {
            source_id: "fixture".to_string(),
            external_id: Some("doc-1".to_string()),
            force: false,
        }))
        .await
        .unwrap_err();

    assert!(matches!(error, NexusFlightProviderError::Handler(_)));
}

async fn seeded_handler() -> (
    LocalMirrorFlightHandler<InMemoryKnowledgeStore, InMemoryNexusRegistry>,
    String,
) {
    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "authority bounded evidence");
    let registry = InMemoryNexusRegistry::new();
    let store = InMemoryKnowledgeStore::new();
    let runtime = NexusSyncRuntime::new(registry.clone());

    let outcome = runtime
        .ingest_once(
            &connector,
            SourceItemRef::new("fixture", "doc-1"),
            &PlainTextNormalizer,
            &store,
            NormalizationContext::new(KnowledgeSourceKind::WebPage, AuthorityLevel::Curated),
        )
        .await
        .unwrap();

    (
        LocalMirrorFlightHandler::new(store, registry),
        outcome.document.content_hash,
    )
}

fn string_column(batch: &arrow_array::RecordBatch, index: usize) -> &StringArray {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap()
}
