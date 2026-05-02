use wendao_nexus_connectors::StaticKnowledgeConnector;
use wendao_nexus_core::{KnowledgeSourceConnector, KnowledgeSourceKind, SourceItemRef};

#[tokio::test]
async fn static_connector_discovers_and_fetches_documents() {
    let connector = StaticKnowledgeConnector::new("fixture", KnowledgeSourceKind::WebPage)
        .with_document("doc-1", "hello");

    let batch = connector.discover(None).await.unwrap();
    assert_eq!(batch.items.len(), 1);

    let raw = connector
        .fetch(SourceItemRef::new("fixture", "doc-1"))
        .await
        .unwrap();
    assert_eq!(raw.payload, b"hello");
}
