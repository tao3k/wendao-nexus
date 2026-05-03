use arrow_array::{Array, BooleanArray, Float64Array, StringArray, TimestampNanosecondArray};
use chrono::{TimeZone, Utc};
use wendao_nexus_core::{
    AuthorityLevel, EvidenceKind, EvidenceRecord, ExternalKnowledgeDocument,
    ExternalKnowledgeSearchResponse, KnowledgeSection, KnowledgeSourceKind, NexusJobKind,
    NexusJobRecord, NexusJobStatus, ProvenanceBundle, ProvenanceRecord, SourceCheckpoint,
    SourceMetadata,
};
use wendao_nexus_flight::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, compare_result_record_batch, open_document_record_batch,
    open_rows_from_document, search_result_record_batch, search_rows_from_response,
    status_record_batch, sync_result_record_batch,
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
            evidence_kind: EvidenceKind::TrialResult,
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
    let source_kinds = batch
        .column(12)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let pmids = batch
        .column(23)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let evidence_kinds = batch
        .column(25)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();

    assert_eq!(rows[0].canonical_uri, "nexus://pubmed/PMID:123");
    assert_eq!(rows[0].source_kind.as_deref(), Some("PubMed"));
    assert_eq!(rows[0].pmid.as_deref(), Some("PMID:123"));
    assert_eq!(rows[0].evidence_kind.as_deref(), Some("trial_result"));
    assert_eq!(scores.value(0), 0.91);
    assert_eq!(source_kinds.value(0), "PubMed");
    assert_eq!(pmids.value(0), "PMID:123");
    assert_eq!(evidence_kinds.value(0), "trial_result");
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

    let heading_paths = batch
        .column(5)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let bodies = batch
        .column(6)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();

    assert_eq!(heading_paths.value(0), r#"["Trial","Intro"]"#);
    assert_eq!(bodies.value(0), "section body");
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
