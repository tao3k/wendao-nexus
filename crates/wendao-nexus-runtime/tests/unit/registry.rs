use wendao_nexus_core::{NexusJobKind, NexusJobRecord, SourceCheckpoint};
use wendao_nexus_runtime::{
    CheckpointRegistry, ContentHashRegistry, InMemoryNexusRegistry, JobRegistry,
};

#[tokio::test]
async fn in_memory_registry_tracks_jobs_checkpoints_and_hashes() {
    let registry = InMemoryNexusRegistry::new();
    let job = NexusJobRecord::new("fixture", NexusJobKind::Discover);
    let job_id = job.job_id;

    registry.put_job(job).await.unwrap();
    assert!(registry.get_job(job_id).await.unwrap().is_some());

    registry
        .upsert_checkpoint(SourceCheckpoint::new("fixture"))
        .await
        .unwrap();
    assert!(registry.get_checkpoint("fixture").await.unwrap().is_some());

    assert!(registry
        .mark_content_hash("sha256:abc".to_string())
        .await
        .unwrap());
    assert!(!registry
        .mark_content_hash("sha256:abc".to_string())
        .await
        .unwrap());
    assert!(registry.contains_content_hash("sha256:abc").await.unwrap());
}
