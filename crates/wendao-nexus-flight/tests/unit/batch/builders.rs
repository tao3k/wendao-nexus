use arrow_array::{Array, BooleanArray, Float64Array, StringArray, TimestampNanosecondArray};
use chrono::{TimeZone, Utc};
use wendao_nexus_core::{NexusJobKind, NexusJobRecord, NexusJobStatus, SourceCheckpoint};
use wendao_nexus_flight::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, compare_result_record_batch, open_document_record_batch,
    search_result_record_batch, status_record_batch, sync_result_record_batch,
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
        section_id: Some("abstract".to_string()),
        heading_path_json: Some("[\"Trial\",\"Abstract\"]".to_string()),
        source_kind: Some("PubMed".to_string()),
        published_at: Some(fetched_at),
        source_updated_at: Some(fetched_at),
        trust_score: Some(0.8),
        freshness_score: Some(0.7),
        semantic_score: Some(0.92),
        lexical_score: Some(0.81),
        rerank_score: Some(0.95),
        license_json: Some("{\"name\":\"Public Domain\"}".to_string()),
        metadata_json: Some("{\"pmid\":\"123\"}".to_string()),
        doi: Some("10.1000/example".to_string()),
        pmid: Some("123".to_string()),
        jurisdiction: None,
        evidence_kind: Some("section".to_string()),
    }])
    .unwrap();

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().field(0).name(), "source_id");
    assert_eq!(batch.schema().field(10).name(), "section_id");

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
    let semantic_scores = batch
        .column(17)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    let published_at_values = batch
        .column(13)
        .as_any()
        .downcast_ref::<TimestampNanosecondArray>()
        .unwrap();
    let dois = batch
        .column(22)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let evidence_kinds = batch
        .column(25)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();

    assert_eq!(source_ids.value(0), "pubmed");
    assert_eq!(scores.value(0), 0.92);
    assert_eq!(
        published_at_values.value(0),
        fetched_at.timestamp_nanos_opt().unwrap()
    );
    assert_eq!(semantic_scores.value(0), 0.92);
    assert_eq!(dois.value(0), "10.1000/example");
    assert_eq!(evidence_kinds.value(0), "section");
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
