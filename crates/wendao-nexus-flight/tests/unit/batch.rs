use arrow_array::{Array, BooleanArray, Float64Array, StringArray};
use chrono::{TimeZone, Utc};
use wendao_nexus_core::{
    AuthorityLevel, EvidenceRecord, ExternalKnowledgeDocument, ExternalKnowledgeSearchResponse,
    KnowledgeSection, KnowledgeSourceKind, NexusJobKind, NexusJobRecord, NexusJobStatus,
    ProvenanceBundle, ProvenanceRecord, SourceCheckpoint, SourceMetadata,
};
use wendao_nexus_flight::{
    compare_result_record_batch, open_document_record_batch, open_rows_from_document,
    search_result_record_batch, search_rows_from_response, status_record_batch,
    sync_result_record_batch, FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow,
    FlightStatusRow, FlightSyncResultRow,
};

#[test]
fn search_batch_preserves_evidence_columns() {
    let fetched_at = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    let batch = search_result_record_batch(&[FlightSearchResultRow {
        source_id: "pubmed".to_string(),
        external_id: "PMID:123".to_string(),
        title: "Trial".to_string(),
        snippet: Some("evidence".to_string()),
        score: Some(0.92),
        authority_level: "PeerReviewed".to_string(),
        canonical_uri: "https://pubmed.ncbi.nlm.nih.gov/123/".to_string(),
        fetched_at: Some(fetched_at),
        content_hash: "sha256:abc".to_string(),
        provenance_json: Some("{\"source\":\"pubmed\"}".to_string()),
    }])
    .unwrap();

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().field(0).name(), "source_id");

    let source_ids = batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let scores = batch
        .column(4)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert_eq!(source_ids.value(0), "pubmed");
    assert_eq!(scores.value(0), 0.92);
}

#[test]
fn open_batch_accepts_section_rows() {
    let batch = open_document_record_batch(&[FlightOpenDocumentRow {
        source_id: "wiki".to_string(),
        external_id: "page:Rust".to_string(),
        canonical_uri: "https://example.test/wiki/Rust".to_string(),
        title: "Rust".to_string(),
        section_id: Some("intro".to_string()),
        heading_path_json: Some("[\"Rust\"]".to_string()),
        body: Some("body".to_string()),
        metadata_json: None,
        provenance_json: Some("{\"authority\":\"Community\"}".to_string()),
    }])
    .unwrap();

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 9);
    assert!(batch.column(7).is_null(0));
}

#[test]
fn search_rows_can_be_built_from_core_evidence_response() {
    let fetched_at = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    let response = ExternalKnowledgeSearchResponse {
        query: "GLP-1".to_string(),
        records: vec![EvidenceRecord {
            source_id: "pubmed".to_string(),
            external_id: "PMID:123".to_string(),
            title: "Trial".to_string(),
            snippet: "evidence".to_string(),
            score: Some("0.91".to_string()),
            provenance: ProvenanceBundle {
                primary: provenance_fixture("pubmed", "PMID:123", fetched_at),
                corroborating: Vec::new(),
                conflicting: Vec::new(),
            },
        }],
        generated_at: fetched_at,
    };

    let rows = search_rows_from_response(&response).unwrap();
    let batch = search_result_record_batch(&rows).unwrap();
    let scores = batch
        .column(4)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert_eq!(rows[0].canonical_uri, "nexus://pubmed/PMID:123");
    assert_eq!(scores.value(0), 0.91);
}

#[test]
fn open_rows_can_be_built_from_normalized_document_sections() {
    let document = document_fixture(true);
    let rows = open_rows_from_document(&document, true, false).unwrap();
    let batch = open_document_record_batch(&rows).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].section_id.as_deref(), Some("intro"));
    assert!(rows[0].provenance_json.is_none());
    assert!(batch.column(8).is_null(0));
}

#[test]
fn sync_batch_can_be_built_from_job_record() {
    let job = NexusJobRecord::new("fixture", NexusJobKind::Fetch).finish(NexusJobStatus::Deduped);
    let row = FlightSyncResultRow::from(&job);
    let batch = sync_result_record_batch(&[row]).unwrap();

    let statuses = batch
        .column(3)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let dedup_hits = batch
        .column(5)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();

    assert_eq!(statuses.value(0), "Deduped");
    assert!(!dedup_hits.value(0));
}

#[test]
fn status_batch_can_be_built_from_checkpoint() {
    let mut checkpoint = SourceCheckpoint::new("pubmed");
    checkpoint.last_seen_revision = Some("rev-1".to_string());
    checkpoint.last_content_hash = Some("sha256:abc".to_string());

    let row = FlightStatusRow::from(&checkpoint);
    let batch = status_record_batch(&[row]).unwrap();

    let revisions = batch
        .column(3)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(revisions.value(0), "rev-1");
}

#[test]
fn compare_batch_carries_authority_flags() {
    let batch = compare_result_record_batch(&[FlightCompareResultRow {
        claim: "claim".to_string(),
        verdict: "insufficient_authority".to_string(),
        conflict_detected: false,
        insufficient_authority: true,
        stale_evidence: false,
        provenance_json: None,
    }])
    .unwrap();

    let insufficient_authority = batch
        .column(3)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();

    assert!(insufficient_authority.value(0));
    assert!(batch.column(5).is_null(0));
}

fn document_fixture(include_section: bool) -> ExternalKnowledgeDocument {
    let fetched_at = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    ExternalKnowledgeDocument {
        source_id: "pubmed".to_string(),
        external_id: "PMID:123".to_string(),
        canonical_uri: "nexus://pubmed/PMID:123".to_string(),
        title: "Trial".to_string(),
        body: "document body".to_string(),
        sections: if include_section {
            vec![KnowledgeSection {
                section_id: "intro".to_string(),
                heading_path: vec!["Trial".to_string(), "Intro".to_string()],
                text: "section body".to_string(),
                anchors: Vec::new(),
                citations: Vec::new(),
                tables: Vec::new(),
                figures: Vec::new(),
            }]
        } else {
            Vec::new()
        },
        metadata: SourceMetadata::default(),
        provenance: provenance_fixture("pubmed", "PMID:123", fetched_at),
        license: None,
        fetched_at,
        source_updated_at: None,
        content_hash: "sha256:PMID:123".to_string(),
    }
}

fn provenance_fixture(
    source_id: &str,
    external_id: &str,
    fetched_at: chrono::DateTime<Utc>,
) -> ProvenanceRecord {
    ProvenanceRecord {
        source_id: source_id.to_string(),
        source_kind: KnowledgeSourceKind::PubMed,
        authority_level: AuthorityLevel::PeerReviewed,
        canonical_uri: format!("nexus://{source_id}/{external_id}"),
        version: None,
        revision_id: None,
        doi: None,
        pmid: Some(external_id.to_string()),
        jurisdiction: None,
        published_at: None,
        fetched_at,
        content_hash: format!("sha256:{external_id}"),
        trust_signals: Vec::new(),
    }
}
