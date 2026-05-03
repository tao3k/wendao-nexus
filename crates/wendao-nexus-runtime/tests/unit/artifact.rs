use std::collections::BTreeMap;

use wendao_nexus_runtime::{ArtifactKind, ArtifactStore, ArtifactWrite, LocalFileArtifactStore};

#[tokio::test]
async fn local_file_artifact_store_persists_payloads_across_reopen() {
    let root = artifact_root("local_file_artifact_store_persists");
    cleanup_dir(&root);

    let store = LocalFileArtifactStore::open(&root).unwrap();
    let descriptor = store
        .put_artifact(
            ArtifactWrite::new(
                "customer/source",
                "docs/protocol.md",
                "sha256:abc123",
                ArtifactKind::RawSourcePayload,
                "text/markdown",
                b"# Protocol\nEvidence boundary.".to_vec(),
            )
            .with_metadata(BTreeMap::from([(
                "canonical_uri".to_string(),
                "file:///docs/protocol.md".to_string(),
            )])),
        )
        .await
        .unwrap();

    assert_eq!(descriptor.kind, ArtifactKind::RawSourcePayload);
    assert_eq!(descriptor.byte_len, 29);
    assert!(
        descriptor
            .relative_path
            .contains("customer_source/docs_protocol.md/raw-source-payload/sha256_abc123.payload")
    );
    drop(store);

    let reopened = LocalFileArtifactStore::open(&root).unwrap();
    let artifact = reopened
        .get_artifact(
            "customer/source",
            "docs/protocol.md",
            ArtifactKind::RawSourcePayload,
            "sha256:abc123",
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(artifact.descriptor.media_type, "text/markdown");
    assert_eq!(artifact.bytes, b"# Protocol\nEvidence boundary.");
    assert_eq!(
        artifact
            .descriptor
            .metadata
            .get("canonical_uri")
            .map(String::as_str),
        Some("file:///docs/protocol.md")
    );

    cleanup_dir(&root);
}

#[tokio::test]
async fn local_file_artifact_store_lists_artifacts_for_one_item() {
    let root = artifact_root("local_file_artifact_store_lists");
    cleanup_dir(&root);
    let store = LocalFileArtifactStore::open(&root).unwrap();

    store
        .put_artifact(ArtifactWrite::new(
            "fixture",
            "doc-1",
            "sha256:raw",
            ArtifactKind::RawSourcePayload,
            "text/plain",
            b"raw".to_vec(),
        ))
        .await
        .unwrap();
    store
        .put_artifact(ArtifactWrite::new(
            "fixture",
            "doc-1",
            "sha256:normalized",
            ArtifactKind::NormalizedDocument,
            "application/json",
            br#"{"title":"Doc"}"#.to_vec(),
        ))
        .await
        .unwrap();

    let descriptors = store.list_artifacts("fixture", "doc-1").await.unwrap();

    assert_eq!(descriptors.len(), 2);
    assert_eq!(descriptors[0].kind, ArtifactKind::RawSourcePayload);
    assert_eq!(descriptors[1].kind, ArtifactKind::NormalizedDocument);
    assert!(
        store
            .list_artifacts("fixture", "missing")
            .await
            .unwrap()
            .is_empty()
    );

    cleanup_dir(&root);
}

fn artifact_root(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("wendao-nexus-{test_name}-{}", uuid::Uuid::new_v4()))
}

fn cleanup_dir(path: &std::path::PathBuf) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}
