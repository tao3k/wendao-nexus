use chrono::{Duration, Utc};
use wendao_nexus_core::{
    AuthorityLevel, ExternalKnowledgeDocument, ExternalKnowledgeOpenRequest,
    ExternalKnowledgeSearchRequest, KnowledgeSection, KnowledgeSourceKind, ProvenanceRecord,
    SourceMetadata, TrustPolicy,
};
use wendao_nexus_runtime::{InMemoryKnowledgeStore, LocalKnowledgeStore};

#[tokio::test]
async fn local_store_search_filters_by_source_trust_and_freshness() {
    let store = InMemoryKnowledgeStore::new();
    store
        .upsert_document(document_fixture(DocumentFixture {
            source_id: "pubmed",
            external_id: "PMID:1",
            title: "GLP-1 cardiovascular trial",
            body: "Peer reviewed cardiovascular evidence.",
            authority: AuthorityLevel::PeerReviewed,
            fetched_days_ago: 2,
        }))
        .await
        .unwrap();
    store
        .upsert_document(document_fixture(DocumentFixture {
            source_id: "wiki",
            external_id: "page:glp1",
            title: "GLP-1 overview",
            body: "Community overview of cardiovascular effects.",
            authority: AuthorityLevel::Community,
            fetched_days_ago: 2,
        }))
        .await
        .unwrap();
    store
        .upsert_document(document_fixture(DocumentFixture {
            source_id: "pubmed",
            external_id: "PMID:old",
            title: "GLP-1 cardiovascular older trial",
            body: "Older peer reviewed cardiovascular evidence.",
            authority: AuthorityLevel::PeerReviewed,
            fetched_days_ago: 400,
        }))
        .await
        .unwrap();

    let mut request = ExternalKnowledgeSearchRequest::new("GLP-1 cardiovascular");
    request.sources = vec!["pubmed".to_string()];
    request.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::PeerReviewed);
    request.freshness_days = Some(365);
    request.limit = 10;

    let response = store.search(request).await.unwrap();

    assert_eq!(response.records.len(), 1);
    assert_eq!(response.records[0].source_id, "pubmed");
    assert_eq!(response.records[0].external_id, "PMID:1");
    assert_eq!(
        response.records[0].provenance.primary.authority_level,
        AuthorityLevel::PeerReviewed
    );
}

#[tokio::test]
async fn local_store_open_can_strip_sections_for_lightweight_callers() {
    let store = InMemoryKnowledgeStore::new();
    store
        .upsert_document(document_fixture(DocumentFixture {
            source_id: "customer",
            external_id: "sop-1",
            title: "Customer SOP",
            body: "Operational body.",
            authority: AuthorityLevel::CustomerInternal,
            fetched_days_ago: 0,
        }))
        .await
        .unwrap();

    let document = store
        .open_document(ExternalKnowledgeOpenRequest {
            source_id: "customer".to_string(),
            external_id: "sop-1".to_string(),
            include_sections: false,
            include_provenance: true,
        })
        .await
        .unwrap();

    assert_eq!(document.title, "Customer SOP");
    assert!(document.sections.is_empty());
    assert_eq!(
        document.provenance.authority_level,
        AuthorityLevel::CustomerInternal
    );
}

#[tokio::test]
async fn local_store_reports_not_found_for_open() {
    let store = InMemoryKnowledgeStore::new();
    let error = store
        .open_document(ExternalKnowledgeOpenRequest {
            source_id: "pubmed".to_string(),
            external_id: "missing".to_string(),
            include_sections: true,
            include_provenance: true,
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("missing"));
}

struct DocumentFixture {
    source_id: &'static str,
    external_id: &'static str,
    title: &'static str,
    body: &'static str,
    authority: AuthorityLevel,
    fetched_days_ago: i64,
}

fn document_fixture(fixture: DocumentFixture) -> ExternalKnowledgeDocument {
    let fetched_at = Utc::now() - Duration::days(fixture.fetched_days_ago);
    ExternalKnowledgeDocument {
        source_id: fixture.source_id.to_string(),
        external_id: fixture.external_id.to_string(),
        canonical_uri: format!("nexus://{}/{}", fixture.source_id, fixture.external_id),
        title: fixture.title.to_string(),
        body: fixture.body.to_string(),
        sections: vec![KnowledgeSection {
            section_id: "intro".to_string(),
            heading_path: vec![fixture.title.to_string()],
            text: fixture.body.to_string(),
            anchors: Vec::new(),
            citations: Vec::new(),
            tables: Vec::new(),
            figures: Vec::new(),
        }],
        metadata: SourceMetadata::default(),
        provenance: ProvenanceRecord {
            source_id: fixture.source_id.to_string(),
            source_kind: KnowledgeSourceKind::Other("fixture".to_string()),
            authority_level: fixture.authority,
            canonical_uri: format!("nexus://{}/{}", fixture.source_id, fixture.external_id),
            version: None,
            revision_id: None,
            doi: None,
            pmid: None,
            jurisdiction: None,
            published_at: None,
            fetched_at,
            content_hash: format!("sha256:{}", fixture.external_id),
            trust_signals: Vec::new(),
        },
        license: None,
        fetched_at,
        source_updated_at: None,
        content_hash: format!("sha256:{}", fixture.external_id),
    }
}
