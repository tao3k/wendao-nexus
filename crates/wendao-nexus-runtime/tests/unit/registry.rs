use wendao_nexus_core::{
    AuthorityLevel, KnowledgeSourceKind, NexusJobKind, NexusJobRecord, NexusSourceRecord,
    SourceCapabilities, SourceCheckpoint,
};
use wendao_nexus_runtime::{
    CheckpointRegistry, ContentHashRegistry, InMemoryNexusRegistry, JobRegistry, SourceRegistry,
};

#[tokio::test]
async fn in_memory_registry_tracks_sources_jobs_checkpoints_and_hashes() {
    let registry = InMemoryNexusRegistry::new();
    let source = source_fixture("fixture", true);
    let disabled = source_fixture("disabled-fixture", false);
    registry.upsert_source(source.clone()).await.unwrap();
    registry.upsert_source(disabled.clone()).await.unwrap();

    assert_eq!(registry.get_source("fixture").await.unwrap(), Some(source));
    assert_eq!(registry.list_sources(false).await.unwrap().len(), 1);
    assert_eq!(registry.list_sources(true).await.unwrap().len(), 2);

    let job = NexusJobRecord::new("fixture", NexusJobKind::Discover);
    let job_id = job.job_id;

    registry.put_job(job).await.unwrap();
    assert!(registry.get_job(job_id).await.unwrap().is_some());

    registry
        .upsert_checkpoint(SourceCheckpoint::new("fixture"))
        .await
        .unwrap();
    assert!(registry.get_checkpoint("fixture").await.unwrap().is_some());

    assert!(
        registry
            .mark_content_hash("sha256:abc".to_string())
            .await
            .unwrap()
    );
    assert!(
        !registry
            .mark_content_hash("sha256:abc".to_string())
            .await
            .unwrap()
    );
    assert!(registry.contains_content_hash("sha256:abc").await.unwrap());
}

#[tokio::test]
async fn in_memory_registry_filters_jobs_by_source() {
    let registry = InMemoryNexusRegistry::new();
    let first = NexusJobRecord::new("fixture-a", NexusJobKind::Fetch);
    let second = NexusJobRecord::new("fixture-b", NexusJobKind::Fetch);
    registry.put_job(first.clone()).await.unwrap();
    registry.put_job(second.clone()).await.unwrap();

    let fixture_a_jobs = registry.list_jobs(Some("fixture-a")).await.unwrap();
    let all_jobs = registry.list_jobs(None).await.unwrap();

    assert_eq!(fixture_a_jobs, vec![first]);
    assert_eq!(all_jobs.len(), 2);
}

fn source_fixture(source_id: &str, enabled: bool) -> NexusSourceRecord {
    let mut source = NexusSourceRecord::new(source_id, KnowledgeSourceKind::ApiFeed);
    source.display_name = format!("Source {source_id}");
    source.base_uri = Some(format!("https://example.test/{source_id}"));
    source.auth_mode = Some("none".to_string());
    source.authority_level = AuthorityLevel::Curated;
    source.sync_policy = Some("mirror_and_federated".to_string());
    source.capabilities = SourceCapabilities::mirror_fetch();
    source.enabled = enabled;
    source
}
