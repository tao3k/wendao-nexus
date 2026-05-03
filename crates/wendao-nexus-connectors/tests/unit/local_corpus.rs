use std::path::PathBuf;

use wendao_nexus_connectors::{LocalCorpusConfig, LocalCorpusConnector};
use wendao_nexus_core::{
    KnowledgeSourceConnector, KnowledgeSourceKind, SOURCE_METADATA_DOI_KEY,
    SOURCE_METADATA_PMID_KEY, SOURCE_METADATA_REVISION_ID_KEY, SOURCE_METADATA_TITLE_KEY,
    SourceCheckpoint, SourceItemRef,
};

#[tokio::test]
async fn local_corpus_connector_loads_jsonl_and_markdown_fixtures() {
    let connector = fixture_connector();

    assert_eq!(connector.len(), 2);
    assert_eq!(connector.source_kind(), KnowledgeSourceKind::MedicalJournal);
    assert!(connector.capabilities().structured_metadata);
    assert!(connector.capabilities().license_metadata);

    let discovered = connector.discover(None).await.unwrap();
    assert_eq!(discovered.items.len(), 2);
    assert_eq!(discovered.items[0].external_id, "medical/guideline-demo");
    assert_eq!(discovered.items[1].external_id, "medical/pubmed-demo-1");

    let article = connector
        .fetch(SourceItemRef::new(
            "demo-medical-pack",
            "medical/pubmed-demo-1",
        ))
        .await
        .unwrap();
    assert_eq!(article.media_type, "text/plain");
    assert_eq!(
        article
            .metadata
            .get(SOURCE_METADATA_PMID_KEY)
            .map(String::as_str),
        Some("PMID:DEMO1")
    );
    assert_eq!(
        article
            .metadata
            .get(SOURCE_METADATA_DOI_KEY)
            .map(String::as_str),
        Some("10.1000/demo1")
    );
    assert!(article.source_updated_at.is_some());

    let guideline = connector
        .fetch(SourceItemRef::new(
            "demo-medical-pack",
            "medical/guideline-demo",
        ))
        .await
        .unwrap();
    assert_eq!(guideline.media_type, "text/markdown");
    assert_eq!(
        guideline
            .metadata
            .get(SOURCE_METADATA_TITLE_KEY)
            .map(String::as_str),
        Some("Demo Clinical Guideline")
    );
    assert_eq!(
        guideline
            .metadata
            .get(SOURCE_METADATA_REVISION_ID_KEY)
            .map(String::as_str),
        Some("guideline-rev-1")
    );
    assert!(
        String::from_utf8(guideline.payload)
            .unwrap()
            .contains("Deterministic clinical guidance fixture")
    );
}

#[tokio::test]
async fn local_corpus_delta_reports_fixture_upserts() {
    let connector = fixture_connector();
    let checkpoint = SourceCheckpoint::new("demo-medical-pack");

    let delta = connector.delta(checkpoint).await.unwrap();

    assert_eq!(delta.changes.len(), 2);
    assert_eq!(delta.source_id, "demo-medical-pack");
    assert_eq!(delta.next_checkpoint.source_id, "demo-medical-pack");
    assert!(delta.next_checkpoint.last_success_at.is_some());
}

#[tokio::test]
async fn local_corpus_assigns_connector_source_for_source_agnostic_fixtures() {
    let config = LocalCorpusConfig::new("expected-source", KnowledgeSourceKind::MedicalJournal);
    let path = corpus_root().join("medical/articles.jsonl");

    let connector = LocalCorpusConnector::from_path(config, path).unwrap();

    assert_eq!(connector.source_id(), "expected-source");
}

fn fixture_connector() -> LocalCorpusConnector {
    LocalCorpusConnector::from_path(
        LocalCorpusConfig::new("demo-medical-pack", KnowledgeSourceKind::MedicalJournal),
        corpus_root().join("medical"),
    )
    .unwrap()
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/corpus")
}
