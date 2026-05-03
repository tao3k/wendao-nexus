use wendao_nexus_connectors::{PubMedConnector, WikipediaConnector};
use wendao_nexus_core::{KnowledgeSourceConnector, SourceItemRef};

#[test]
fn pubmed_stub_does_not_advertise_live_or_mirror_execution() {
    let connector = PubMedConnector::default();
    let capabilities = connector.capabilities();

    assert!(!capabilities.discover);
    assert!(!capabilities.fetch);
    assert!(!capabilities.delta);
    assert!(!capabilities.live_query);
    assert!(!capabilities.local_mirror);
    assert!(capabilities.structured_metadata);
    assert!(capabilities.license_metadata);
}

#[test]
fn wikipedia_stub_does_not_advertise_live_or_mirror_execution() {
    let connector = WikipediaConnector::default();
    let capabilities = connector.capabilities();

    assert!(!capabilities.discover);
    assert!(!capabilities.fetch);
    assert!(!capabilities.delta);
    assert!(!capabilities.live_query);
    assert!(!capabilities.local_mirror);
    assert!(capabilities.revisions);
    assert!(capabilities.structured_metadata);
    assert!(capabilities.license_metadata);
}

#[tokio::test]
async fn pubmed_and_wikipedia_live_fetch_remain_explicitly_unsupported() {
    let pubmed = PubMedConnector::default();
    let wikipedia = WikipediaConnector::default();

    let pubmed_error = pubmed
        .fetch(SourceItemRef::new("pubmed", "PMID:fixture"))
        .await
        .unwrap_err()
        .to_string();
    let wikipedia_error = wikipedia
        .fetch(SourceItemRef::new("wikipedia", "page:fixture"))
        .await
        .unwrap_err()
        .to_string();

    assert!(pubmed_error.contains("live pubmed fetch"));
    assert!(wikipedia_error.contains("live wikipedia fetch"));
}
