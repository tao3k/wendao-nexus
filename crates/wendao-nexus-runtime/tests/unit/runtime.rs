use chrono::Utc;
use wendao_nexus_connectors::StaticKnowledgeConnector;
use wendao_nexus_core::{
    AuthorityLevel, ExternalKnowledgeDocument, ExternalKnowledgeSearchRequest, KnowledgeSection,
    KnowledgeSourceKind, NexusJobKind, NexusJobStatus, ProvenanceRecord, SourceItemRef,
    SourceMetadata, TrustPolicy,
};
use wendao_nexus_runtime::{
    CheckpointRegistry, InMemoryKnowledgeStore, InMemoryNexusRegistry, JobRegistry,
    LocalKnowledgeStore, NexusSyncRuntime, NormalizationContext, PlainTextNormalizer,
};

#[tokio::test]
async fn runtime_discovers_and_checkpoints_source() {
    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "hello");
    let registry = InMemoryNexusRegistry::new();
    let runtime = NexusSyncRuntime::new(registry);

    let outcome = runtime.discover_once(&connector).await.unwrap();

    assert_eq!(outcome.job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.batch.items.len(), 1);
    assert!(runtime
        .registry()
        .get_checkpoint("fixture")
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn runtime_marks_duplicate_fetches_as_deduped() {
    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "hello");
    let registry = InMemoryNexusRegistry::new();
    let runtime = NexusSyncRuntime::new(registry);

    let first = runtime
        .fetch_once(&connector, SourceItemRef::new("fixture", "doc-1"))
        .await
        .unwrap();
    let second = runtime
        .fetch_once(&connector, SourceItemRef::new("fixture", "doc-1"))
        .await
        .unwrap();

    assert_eq!(first.job.status, NexusJobStatus::Succeeded);
    assert_eq!(second.job.status, NexusJobStatus::Deduped);
    assert!(!first.dedup_hit);
    assert!(second.dedup_hit);
}

#[tokio::test]
async fn runtime_ingests_fetched_document_into_local_store() {
    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "authority bounded evidence");
    let registry = InMemoryNexusRegistry::new();
    let store = InMemoryKnowledgeStore::new();
    let runtime = NexusSyncRuntime::new(registry);

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

    assert_eq!(outcome.fetch.job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.normalize_job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.document.title, "doc-1");

    let mut search = ExternalKnowledgeSearchRequest::new("authority evidence");
    search.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::Curated);
    let response = store.search(search).await.unwrap();

    assert_eq!(response.records.len(), 1);
    assert_eq!(response.records[0].external_id, "doc-1");
    assert_eq!(
        runtime
            .registry()
            .get_checkpoint("fixture")
            .await
            .unwrap()
            .and_then(|checkpoint| checkpoint.last_content_hash)
            .as_deref(),
        Some(outcome.document.content_hash.as_str())
    );
}

#[tokio::test]
async fn runtime_upserts_wendao_normalized_document_into_local_store() {
    let registry = InMemoryNexusRegistry::new();
    let store = InMemoryKnowledgeStore::new();
    let runtime = NexusSyncRuntime::new(registry);
    let document = normalized_document_fixture(
        "attachments",
        "protocol.docx",
        "Wendao parsed regulated protocol evidence.",
        "sha256:wendao-parsed-protocol",
    );

    let outcome = runtime
        .upsert_normalized_document(document.clone(), &store)
        .await
        .unwrap();

    assert_eq!(outcome.job.job_kind, NexusJobKind::Ingest);
    assert_eq!(outcome.job.status, NexusJobStatus::Succeeded);
    assert!(!outcome.dedup_hit);
    assert_eq!(outcome.document.external_id, "protocol.docx");

    let response = store
        .search(ExternalKnowledgeSearchRequest::new("regulated protocol"))
        .await
        .unwrap();
    assert_eq!(response.records.len(), 1);
    assert_eq!(response.records[0].source_id, "attachments");
    assert_eq!(
        runtime
            .registry()
            .get_checkpoint("attachments")
            .await
            .unwrap()
            .and_then(|checkpoint| checkpoint.last_content_hash)
            .as_deref(),
        Some("sha256:wendao-parsed-protocol")
    );

    let duplicate = runtime
        .upsert_normalized_document(document, &store)
        .await
        .unwrap();
    assert_eq!(duplicate.job.status, NexusJobStatus::Deduped);
    assert!(duplicate.dedup_hit);
}

#[tokio::test]
async fn runtime_rejects_inconsistent_normalized_document_handoff() {
    let registry = InMemoryNexusRegistry::new();
    let store = InMemoryKnowledgeStore::new();
    let runtime = NexusSyncRuntime::new(registry);
    let mut document = normalized_document_fixture(
        "attachments",
        "protocol.docx",
        "Wendao parsed regulated protocol evidence.",
        "sha256:wendao-parsed-protocol",
    );
    document.provenance.content_hash = "sha256:other".to_string();

    let error = runtime
        .upsert_normalized_document(document, &store)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("content_hash"));
    assert!(store
        .get_document("attachments", "protocol.docx")
        .await
        .unwrap()
        .is_none());

    let jobs = runtime
        .registry()
        .list_jobs(Some("attachments"))
        .await
        .unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_kind, NexusJobKind::Ingest);
    assert_eq!(jobs[0].status, NexusJobStatus::Failed);
    assert!(jobs[0]
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("content_hash"));
}

fn normalized_document_fixture(
    source_id: &str,
    external_id: &str,
    body: &str,
    content_hash: &str,
) -> ExternalKnowledgeDocument {
    let fetched_at = Utc::now();
    ExternalKnowledgeDocument {
        source_id: source_id.to_string(),
        external_id: external_id.to_string(),
        canonical_uri: format!("nexus://{source_id}/{external_id}"),
        title: "Wendao Parsed Attachment".to_string(),
        body: body.to_string(),
        sections: vec![KnowledgeSection {
            section_id: "body".to_string(),
            heading_path: vec!["Wendao Parsed Attachment".to_string()],
            text: body.to_string(),
            anchors: Vec::new(),
            citations: Vec::new(),
            tables: Vec::new(),
            figures: Vec::new(),
        }],
        metadata: SourceMetadata::default(),
        provenance: ProvenanceRecord {
            source_id: source_id.to_string(),
            source_kind: KnowledgeSourceKind::ObjectStorage,
            authority_level: AuthorityLevel::CustomerInternal,
            canonical_uri: format!("nexus://{source_id}/{external_id}"),
            version: None,
            revision_id: Some("wendao-docling-revision-1".to_string()),
            doi: None,
            pmid: None,
            jurisdiction: None,
            published_at: None,
            fetched_at,
            content_hash: content_hash.to_string(),
            trust_signals: Vec::new(),
        },
        license: None,
        fetched_at,
        source_updated_at: None,
        content_hash: content_hash.to_string(),
    }
}
