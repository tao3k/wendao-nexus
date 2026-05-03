use chrono::Utc;
use wendao_nexus_connectors::StaticKnowledgeConnector;
use wendao_nexus_core::{
    AuthorityLevel, ExternalKnowledgeDocument, KnowledgeSection, KnowledgeSourceKind, NexusJobKind,
    NexusJobStatus, ProvenanceRecord, SourceItemRef, SourceMetadata,
};
use wendao_nexus_runtime::{
    ArtifactKind, ArtifactStore, CheckpointRegistry, InMemoryNexusRegistry, JobRegistry,
    LocalFileArtifactStore, NexusSyncRuntime, NormalizationContext, PlainTextNormalizer,
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
    assert!(
        runtime
            .registry()
            .get_checkpoint("fixture")
            .await
            .unwrap()
            .is_some()
    );
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
async fn runtime_ingests_fetched_document_as_normalized_handoff() {
    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "authority bounded evidence");
    let registry = InMemoryNexusRegistry::new();
    let runtime = NexusSyncRuntime::new(registry);

    let outcome = runtime
        .ingest_once(
            &connector,
            SourceItemRef::new("fixture", "doc-1"),
            &PlainTextNormalizer,
            NormalizationContext::new(KnowledgeSourceKind::WebPage, AuthorityLevel::Curated),
        )
        .await
        .unwrap();

    assert_eq!(outcome.fetch.job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.normalize_job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.document.title, "doc-1");
    assert_eq!(outcome.document.body, "authority bounded evidence");
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
async fn runtime_ingest_writes_raw_and_normalized_artifacts() {
    let artifact_root = temp_dir("runtime_artifacts");
    cleanup_dir(&artifact_root);

    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "authority bounded artifact evidence");
    let registry = InMemoryNexusRegistry::new();
    let artifact_store = LocalFileArtifactStore::open(&artifact_root).unwrap();
    let runtime = NexusSyncRuntime::new(registry);

    let outcome = runtime
        .ingest_once_with_artifact_store(
            &connector,
            SourceItemRef::new("fixture", "doc-1"),
            &PlainTextNormalizer,
            &artifact_store,
            NormalizationContext::new(KnowledgeSourceKind::WebPage, AuthorityLevel::Curated),
        )
        .await
        .unwrap();

    assert_eq!(
        outcome.ingest.normalize_job.status,
        NexusJobStatus::Succeeded
    );
    assert_eq!(outcome.raw_artifact.kind, ArtifactKind::RawSourcePayload);
    assert_eq!(
        outcome.normalized_artifact.kind,
        ArtifactKind::NormalizedDocument
    );

    let raw = artifact_store
        .get_artifact(
            "fixture",
            "doc-1",
            ArtifactKind::RawSourcePayload,
            &outcome.ingest.fetch.content_hash,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        raw.descriptor.content_hash,
        outcome.ingest.fetch.content_hash
    );
    assert!(artifact_root.join(&raw.descriptor.relative_path).exists());
    assert_eq!(raw.bytes, b"authority bounded artifact evidence");

    let normalized = artifact_store
        .get_artifact(
            "fixture",
            "doc-1",
            ArtifactKind::NormalizedDocument,
            &outcome.ingest.document.content_hash,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        normalized.descriptor.content_hash,
        outcome.ingest.document.content_hash
    );
    assert!(
        artifact_root
            .join(&normalized.descriptor.relative_path)
            .exists()
    );
    let normalized_json = String::from_utf8(normalized.bytes).unwrap();
    assert!(normalized_json.contains("\"title\": \"doc-1\""));
    let replayed_document: ExternalKnowledgeDocument =
        serde_json::from_str(&normalized_json).unwrap();
    assert_eq!(replayed_document, outcome.ingest.document);
    assert_eq!(
        normalized
            .descriptor
            .metadata
            .get("authority_level")
            .map(String::as_str),
        Some("Curated")
    );
    assert_eq!(
        runtime
            .registry()
            .get_checkpoint("fixture")
            .await
            .unwrap()
            .and_then(|checkpoint| checkpoint.last_content_hash)
            .as_deref(),
        Some(outcome.ingest.document.content_hash.as_str())
    );

    cleanup_dir(&artifact_root);
}

#[tokio::test]
async fn runtime_artifact_replay_keeps_deduped_sidecars_stable() {
    let artifact_root = temp_dir("runtime_artifact_dedup_replay");
    cleanup_dir(&artifact_root);

    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "stable replay artifact evidence");
    let registry = InMemoryNexusRegistry::new();
    let artifact_store = LocalFileArtifactStore::open(&artifact_root).unwrap();
    let runtime = NexusSyncRuntime::new(registry);

    let first = runtime
        .ingest_once_with_artifact_store(
            &connector,
            SourceItemRef::new("fixture", "doc-1"),
            &PlainTextNormalizer,
            &artifact_store,
            NormalizationContext::new(KnowledgeSourceKind::WebPage, AuthorityLevel::Curated),
        )
        .await
        .unwrap();
    let second = runtime
        .ingest_once_with_artifact_store(
            &connector,
            SourceItemRef::new("fixture", "doc-1"),
            &PlainTextNormalizer,
            &artifact_store,
            NormalizationContext::new(KnowledgeSourceKind::WebPage, AuthorityLevel::Curated),
        )
        .await
        .unwrap();

    assert!(second.ingest.fetch.dedup_hit);
    assert_eq!(second.raw_artifact, first.raw_artifact);
    assert_eq!(second.normalized_artifact, first.normalized_artifact);

    let replay = artifact_store
        .get_artifact(
            "fixture",
            "doc-1",
            ArtifactKind::NormalizedDocument,
            &second.ingest.document.content_hash,
        )
        .await
        .unwrap()
        .unwrap();
    let replayed_document: ExternalKnowledgeDocument =
        serde_json::from_slice(&replay.bytes).unwrap();
    assert_eq!(
        replayed_document.content_hash,
        second.ingest.document.content_hash
    );
    assert_eq!(
        replayed_document.provenance,
        second.ingest.document.provenance
    );

    cleanup_dir(&artifact_root);
}

#[tokio::test]
async fn runtime_accepts_wendao_normalized_document_handoff() {
    let registry = InMemoryNexusRegistry::new();
    let runtime = NexusSyncRuntime::new(registry);
    let document = normalized_document_fixture(
        "attachments",
        "protocol.docx",
        "Wendao parsed regulated protocol evidence.",
        "sha256:wendao-parsed-protocol",
    );

    let outcome = runtime
        .accept_normalized_document(document.clone())
        .await
        .unwrap();

    assert_eq!(outcome.job.job_kind, NexusJobKind::Ingest);
    assert_eq!(outcome.job.status, NexusJobStatus::Succeeded);
    assert!(!outcome.dedup_hit);
    assert_eq!(outcome.document.external_id, "protocol.docx");
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

    let duplicate = runtime.accept_normalized_document(document).await.unwrap();
    assert_eq!(duplicate.job.status, NexusJobStatus::Deduped);
    assert!(duplicate.dedup_hit);
}

#[tokio::test]
async fn runtime_rejects_inconsistent_normalized_document_handoff() {
    let registry = InMemoryNexusRegistry::new();
    let runtime = NexusSyncRuntime::new(registry);
    let mut document = normalized_document_fixture(
        "attachments",
        "protocol.docx",
        "Wendao parsed regulated protocol evidence.",
        "sha256:wendao-parsed-protocol",
    );
    document.provenance.content_hash = "sha256:other".to_string();

    let error = runtime
        .accept_normalized_document(document)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("content_hash"));

    let jobs = runtime
        .registry()
        .list_jobs(Some("attachments"))
        .await
        .unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_kind, NexusJobKind::Ingest);
    assert_eq!(jobs[0].status, NexusJobStatus::Failed);
    assert!(
        jobs[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("content_hash")
    );
}

fn temp_dir(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("wendao-nexus-{test_name}-{}", uuid::Uuid::new_v4()))
}

fn cleanup_dir(path: &std::path::PathBuf) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
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
