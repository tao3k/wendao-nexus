use arrow_array::{Array, BooleanArray, Float64Array, StringArray, TimestampNanosecondArray};
use arrow_schema::DataType;
use chrono::{TimeZone, Utc};
use wendao_nexus_core::{
    AuthorityLevel, ExternalKnowledgeDocument, KnowledgeSection, KnowledgeSourceKind,
    ProvenanceRecord, SourceMetadata,
};

pub(super) fn document_fixture(include_section: bool) -> ExternalKnowledgeDocument {
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

pub(super) fn provenance_fixture(
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

pub(super) fn compact_batch_snapshot(batch: &arrow_array::RecordBatch) -> String {
    batch
        .schema()
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| format!("{}={}", field.name(), compact_value(batch, index)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compact_value(batch: &arrow_array::RecordBatch, column_index: usize) -> String {
    let column = batch.column(column_index);
    if column.is_null(0) {
        return "<null>".to_string();
    }

    match batch.schema().field(column_index).data_type() {
        DataType::Utf8 => column
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .value(0)
            .to_string(),
        DataType::Float64 => column
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap()
            .value(0)
            .to_string(),
        DataType::Boolean => column
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap()
            .value(0)
            .to_string(),
        DataType::Timestamp(_, _) => column
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .unwrap()
            .value(0)
            .to_string(),
        unsupported => panic!("unsupported compact snapshot type: {unsupported:?}"),
    }
}
