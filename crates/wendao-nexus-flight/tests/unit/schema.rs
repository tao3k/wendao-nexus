use arrow_schema::SchemaRef;
use wendao_nexus_flight::{
    EXTERNAL_KNOWLEDGE_COMPARE_ROUTE, EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
    EXTERNAL_KNOWLEDGE_SYNC_ROUTE, KNOWLEDGE_EVIDENCE_JUDGE_ROUTE, NEXUS_FLIGHT_ROUTE_METADATA_KEY,
    NEXUS_FLIGHT_SCHEMA_VERSION, NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY, compare_result_schema,
    evidence_judge_input_schema, evidence_judge_result_schema, open_document_schema,
    search_result_schema, status_schema, sync_result_schema,
};

#[test]
fn search_schema_carries_provenance_boundary_columns() {
    let schema = search_result_schema();

    assert!(schema.field_with_name("source_id").is_ok());
    assert!(schema.field_with_name("authority_level").is_ok());
    assert!(schema.field_with_name("provenance_json").is_ok());
    assert!(schema.field_with_name("section_id").is_ok());
    assert!(schema.field_with_name("heading_path_json").is_ok());
    assert!(schema.field_with_name("source_kind").is_ok());
    assert!(schema.field_with_name("semantic_score").is_ok());
    assert!(schema.field_with_name("lexical_score").is_ok());
    assert!(schema.field_with_name("rerank_score").is_ok());
    assert!(schema.field_with_name("license_json").is_ok());
    assert!(schema.field_with_name("metadata_json").is_ok());
    assert!(schema.field_with_name("doi").is_ok());
    assert!(schema.field_with_name("pmid").is_ok());
    assert!(schema.field_with_name("jurisdiction").is_ok());
    assert!(schema.field_with_name("evidence_kind").is_ok());
    assert_eq!(
        schema
            .metadata()
            .get(NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY),
        Some(&NEXUS_FLIGHT_SCHEMA_VERSION.to_string())
    );
    assert_eq!(
        schema.metadata().get(NEXUS_FLIGHT_ROUTE_METADATA_KEY),
        Some(&EXTERNAL_KNOWLEDGE_SEARCH_ROUTE.to_string())
    );
}

#[test]
fn status_schema_carries_checkpoint_columns() {
    let schema = status_schema();

    assert!(schema.field_with_name("last_success_at").is_ok());
    assert!(schema.field_with_name("last_seen_revision").is_ok());
    assert!(schema.field_with_name("last_content_hash").is_ok());
}

#[test]
fn route_schemas_match_snapshot() {
    let snapshots = [
        (
            EXTERNAL_KNOWLEDGE_SEARCH_ROUTE,
            search_result_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/external/search
metadata:wendao_nexus.schema_version=1
field:source_id|Utf8|nullable=false
field:external_id|Utf8|nullable=false
field:title|Utf8|nullable=false
field:snippet|Utf8|nullable=true
field:score|Float64|nullable=true
field:authority_level|Utf8|nullable=false
field:canonical_uri|Utf8|nullable=false
field:fetched_at|Timestamp(Nanosecond, Some("UTC"))|nullable=true
field:content_hash|Utf8|nullable=false
field:provenance_json|Utf8|nullable=true
field:section_id|Utf8|nullable=true
field:heading_path_json|Utf8|nullable=true
field:source_kind|Utf8|nullable=true
field:published_at|Timestamp(Nanosecond, Some("UTC"))|nullable=true
field:source_updated_at|Timestamp(Nanosecond, Some("UTC"))|nullable=true
field:trust_score|Float64|nullable=true
field:freshness_score|Float64|nullable=true
field:semantic_score|Float64|nullable=true
field:lexical_score|Float64|nullable=true
field:rerank_score|Float64|nullable=true
field:license_json|Utf8|nullable=true
field:metadata_json|Utf8|nullable=true
field:doi|Utf8|nullable=true
field:pmid|Utf8|nullable=true
field:jurisdiction|Utf8|nullable=true
field:evidence_kind|Utf8|nullable=true"#,
        ),
        (
            EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
            open_document_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/external/open
metadata:wendao_nexus.schema_version=1
field:source_id|Utf8|nullable=false
field:external_id|Utf8|nullable=false
field:canonical_uri|Utf8|nullable=false
field:title|Utf8|nullable=false
field:section_id|Utf8|nullable=true
field:heading_path_json|Utf8|nullable=true
field:body|Utf8|nullable=true
field:metadata_json|Utf8|nullable=true
field:provenance_json|Utf8|nullable=true"#,
        ),
        (
            EXTERNAL_KNOWLEDGE_SYNC_ROUTE,
            sync_result_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/external/sync
metadata:wendao_nexus.schema_version=1
field:job_id|Utf8|nullable=false
field:source_id|Utf8|nullable=false
field:job_kind|Utf8|nullable=false
field:status|Utf8|nullable=false
field:cursor|Utf8|nullable=true
field:dedup_hit|Boolean|nullable=false
field:error|Utf8|nullable=true"#,
        ),
        (
            EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
            status_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/external/status
metadata:wendao_nexus.schema_version=1
field:source_id|Utf8|nullable=false
field:enabled|Boolean|nullable=false
field:last_success_at|Timestamp(Nanosecond, Some("UTC"))|nullable=true
field:last_seen_revision|Utf8|nullable=true
field:last_content_hash|Utf8|nullable=true
field:rate_limit_state|Utf8|nullable=true"#,
        ),
        (
            EXTERNAL_KNOWLEDGE_COMPARE_ROUTE,
            compare_result_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/external/compare
metadata:wendao_nexus.schema_version=1
field:claim|Utf8|nullable=false
field:verdict|Utf8|nullable=false
field:conflict_detected|Boolean|nullable=false
field:insufficient_authority|Boolean|nullable=false
field:stale_evidence|Boolean|nullable=false
field:provenance_json|Utf8|nullable=true"#,
        ),
        (
            KNOWLEDGE_EVIDENCE_JUDGE_ROUTE,
            evidence_judge_input_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/evidence/judge
metadata:wendao_nexus.schema_version=1
field:source_id|Utf8|nullable=false
field:external_id|Utf8|nullable=false
field:canonical_uri|Utf8|nullable=false
field:authority_level|Utf8|nullable=false
field:published_at|Timestamp(Nanosecond, Some("UTC"))|nullable=true
field:fetched_at|Timestamp(Nanosecond, Some("UTC"))|nullable=true
field:doi|Utf8|nullable=true
field:pmid|Utf8|nullable=true
field:jurisdiction|Utf8|nullable=true
field:evidence_kind|Utf8|nullable=true
field:snippet|Utf8|nullable=true
field:provenance_json|Utf8|nullable=true
field:metadata_json|Utf8|nullable=true
field:trust_score|Float64|nullable=true
field:freshness_score|Float64|nullable=true"#,
        ),
        (
            KNOWLEDGE_EVIDENCE_JUDGE_ROUTE,
            evidence_judge_result_schema(),
            r#"metadata:wendao_nexus.route=/knowledge/evidence/judge
metadata:wendao_nexus.schema_version=1
field:source_id|Utf8|nullable=false
field:external_id|Utf8|nullable=false
field:rust_trust_score|Float64|nullable=true
field:julia_evidence_score|Float64|nullable=true
field:corroboration_score|Float64|nullable=true
field:conflict_score|Float64|nullable=true
field:pollution_risk_score|Float64|nullable=true
field:freshness_adjusted_score|Float64|nullable=true
field:final_evidence_score|Float64|nullable=true
field:judgement_label|Utf8|nullable=false
field:explanation_json|Utf8|nullable=true"#,
        ),
    ];

    for (route, schema, expected) in snapshots {
        assert_eq!(schema_snapshot(&schema), expected, "{route} schema drifted");
    }
}

fn schema_snapshot(schema: &SchemaRef) -> String {
    let metadata = [
        NEXUS_FLIGHT_ROUTE_METADATA_KEY,
        NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY,
    ]
    .into_iter()
    .map(|key| {
        format!(
            "metadata:{key}={}",
            schema
                .metadata()
                .get(key)
                .unwrap_or_else(|| panic!("missing schema metadata key {key}"))
        )
    });

    let fields = schema.fields().iter().map(|field| {
        format!(
            "field:{}|{:?}|nullable={}",
            field.name(),
            field.data_type(),
            field.is_nullable()
        )
    });

    metadata.chain(fields).collect::<Vec<_>>().join("\n")
}
