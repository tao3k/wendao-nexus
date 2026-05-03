use arrow_array::{Array, Float64Array, StringArray};
use chrono::{TimeZone, Utc};
use wendao_nexus_core::{
    EvidenceKind, EvidenceRecord, ExternalKnowledgeSearchResponse, ProvenanceBundle,
};
use wendao_nexus_flight::{
    open_document_record_batch, open_rows_from_document, search_result_record_batch,
    search_rows_from_response,
};

use super::fixtures::{document_fixture, provenance_fixture};

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
