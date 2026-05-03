use chrono::{TimeZone, Utc};
use wendao_nexus_core::{
    AuthorityLevel, EvidenceConflictMode, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest, TrustPolicy,
};
use wendao_nexus_flight::{
    EXTERNAL_KNOWLEDGE_COMPARE_ROUTE, EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
    EXTERNAL_KNOWLEDGE_SYNC_ROUTE, FlightCompareResultRow, FlightOpenDocumentRow,
    FlightSearchResultRow, FlightStatusRow, FlightSyncResultRow,
    NEXUS_FLIGHT_COMMAND_SCHEMA_VERSION, NexusFlightCommand, NexusFlightCommandError,
    NexusFlightRoute, NexusFlightStatusRequest, compare_result_record_batch, compare_result_schema,
    open_document_record_batch, open_document_schema, search_result_record_batch,
    search_result_schema, status_record_batch, status_schema, sync_result_record_batch,
    sync_result_schema,
};

use super::support::compact_batch_snapshot;

macro_rules! assert_schema_route {
    ($schema:expr, $route:expr) => {{
        assert_eq!(
            $schema
                .metadata()
                .get(wendao_nexus_flight::NEXUS_FLIGHT_ROUTE_METADATA_KEY)
                .map(String::as_str),
            Some($route)
        );
        assert_eq!(
            $schema
                .metadata()
                .get(wendao_nexus_flight::NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY)
                .map(String::as_str),
            Some(wendao_nexus_flight::NEXUS_FLIGHT_SCHEMA_VERSION)
        );
    }};
}

macro_rules! schema_signature {
    ($schema:expr) => {{
        $schema
            .fields()
            .iter()
            .map(|field| {
                format!(
                    "{}|{:?}|nullable={}",
                    field.name(),
                    field.data_type(),
                    field.is_nullable()
                )
            })
            .collect::<Vec<_>>()
    }};
}

macro_rules! assert_batch_route {
    ($batch:expr, $route:expr) => {{
        assert_eq!(
            $batch
                .schema()
                .metadata()
                .get(wendao_nexus_flight::NEXUS_FLIGHT_ROUTE_METADATA_KEY)
                .map(String::as_str),
            Some($route)
        );
        assert_eq!(
            $batch
                .schema()
                .metadata()
                .get(wendao_nexus_flight::NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY)
                .map(String::as_str),
            Some(wendao_nexus_flight::NEXUS_FLIGHT_SCHEMA_VERSION)
        );
    }};
}

#[test]
fn route_constants_and_command_envelope_are_conformant() {
    assert_eq!(
        NexusFlightRoute::all()
            .into_iter()
            .map(NexusFlightRoute::as_str)
            .collect::<Vec<_>>(),
        vec![
            EXTERNAL_KNOWLEDGE_SEARCH_ROUTE,
            EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
            EXTERNAL_KNOWLEDGE_SYNC_ROUTE,
            EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
            EXTERNAL_KNOWLEDGE_COMPARE_ROUTE,
        ]
    );
    assert_eq!(NEXUS_FLIGHT_COMMAND_SCHEMA_VERSION, 1);

    let status = NexusFlightCommand::Status(NexusFlightStatusRequest::all_sources());
    assert_eq!(
        String::from_utf8(status.encode_json().unwrap()).unwrap(),
        r#"{"schema_version":1,"route":"/knowledge/external/status","payload":{"sources":[]}}"#
    );
    assert_eq!(
        NexusFlightCommand::from_descriptor(&status.to_descriptor().unwrap()).unwrap(),
        status
    );

    let search = NexusFlightCommand::Search(ExternalKnowledgeSearchRequest::new(
        "authority bounded evidence",
    ));
    assert_eq!(
        NexusFlightCommand::from_descriptor(&search.to_descriptor().unwrap()).unwrap(),
        search
    );

    let open = NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
        source_id: "source".to_string(),
        external_id: "doc".to_string(),
        include_sections: true,
        include_provenance: true,
    });
    assert_eq!(
        NexusFlightCommand::decode_json(&open.encode_json().unwrap()).unwrap(),
        open
    );

    let compare = NexusFlightCommand::Compare(ExternalKnowledgeCompareRequest {
        claim: "claim".to_string(),
        sources: vec!["source".to_string()],
        mode: EvidenceConflictMode::EvidenceConflictCheck,
        trust_policy: TrustPolicy::authority_at_least(AuthorityLevel::Official),
    });
    assert_eq!(
        NexusFlightCommand::from_descriptor(&compare.to_descriptor().unwrap()).unwrap(),
        compare
    );
}

#[test]
fn malformed_or_unsupported_commands_are_rejected_before_handlers() {
    assert!(matches!(
        NexusFlightCommand::decode_json(
            br#"{"schema_version":2,"route":"/knowledge/external/status","payload":{"sources":[]}}"#,
        )
        .unwrap_err(),
        NexusFlightCommandError::UnsupportedSchemaVersion(2)
    ));
    assert!(matches!(
        NexusFlightCommand::decode_json(
            br#"{"schema_version":1,"route":"/knowledge/external/missing","payload":{}}"#,
        )
        .unwrap_err(),
        NexusFlightCommandError::UnsupportedRoute(_)
    ));
    assert!(matches!(
        NexusFlightCommand::decode_json(
            br#"{"schema_version":1,"route":"/knowledge/external/open","payload":{"source_id":"source"}}"#,
        )
        .unwrap_err(),
        NexusFlightCommandError::Json(_)
    ));
    assert!(matches!(
        NexusFlightCommand::decode_json(br#"{"route":"#).unwrap_err(),
        NexusFlightCommandError::Json(_)
    ));
}

#[test]
fn arrow_route_schemas_are_conformant() {
    let route_schemas = [
        (
            EXTERNAL_KNOWLEDGE_SEARCH_ROUTE,
            search_result_schema(),
            vec![
                "source_id|Utf8|nullable=false",
                "external_id|Utf8|nullable=false",
                "title|Utf8|nullable=false",
                "snippet|Utf8|nullable=true",
                "score|Float64|nullable=true",
                "authority_level|Utf8|nullable=false",
                "canonical_uri|Utf8|nullable=false",
                "fetched_at|Timestamp(Nanosecond, Some(\"UTC\"))|nullable=true",
                "content_hash|Utf8|nullable=false",
                "provenance_json|Utf8|nullable=true",
                "section_id|Utf8|nullable=true",
                "heading_path_json|Utf8|nullable=true",
                "source_kind|Utf8|nullable=true",
                "published_at|Timestamp(Nanosecond, Some(\"UTC\"))|nullable=true",
                "source_updated_at|Timestamp(Nanosecond, Some(\"UTC\"))|nullable=true",
                "trust_score|Float64|nullable=true",
                "freshness_score|Float64|nullable=true",
                "semantic_score|Float64|nullable=true",
                "lexical_score|Float64|nullable=true",
                "rerank_score|Float64|nullable=true",
                "license_json|Utf8|nullable=true",
                "metadata_json|Utf8|nullable=true",
                "doi|Utf8|nullable=true",
                "pmid|Utf8|nullable=true",
                "jurisdiction|Utf8|nullable=true",
                "evidence_kind|Utf8|nullable=true",
            ],
        ),
        (
            EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
            open_document_schema(),
            vec![
                "source_id|Utf8|nullable=false",
                "external_id|Utf8|nullable=false",
                "canonical_uri|Utf8|nullable=false",
                "title|Utf8|nullable=false",
                "section_id|Utf8|nullable=true",
                "heading_path_json|Utf8|nullable=true",
                "body|Utf8|nullable=true",
                "metadata_json|Utf8|nullable=true",
                "provenance_json|Utf8|nullable=true",
            ],
        ),
        (
            EXTERNAL_KNOWLEDGE_SYNC_ROUTE,
            sync_result_schema(),
            vec![
                "job_id|Utf8|nullable=false",
                "source_id|Utf8|nullable=false",
                "job_kind|Utf8|nullable=false",
                "status|Utf8|nullable=false",
                "cursor|Utf8|nullable=true",
                "dedup_hit|Boolean|nullable=false",
                "error|Utf8|nullable=true",
            ],
        ),
        (
            EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
            status_schema(),
            vec![
                "source_id|Utf8|nullable=false",
                "enabled|Boolean|nullable=false",
                "last_success_at|Timestamp(Nanosecond, Some(\"UTC\"))|nullable=true",
                "last_seen_revision|Utf8|nullable=true",
                "last_content_hash|Utf8|nullable=true",
                "rate_limit_state|Utf8|nullable=true",
            ],
        ),
        (
            EXTERNAL_KNOWLEDGE_COMPARE_ROUTE,
            compare_result_schema(),
            vec![
                "claim|Utf8|nullable=false",
                "verdict|Utf8|nullable=false",
                "conflict_detected|Boolean|nullable=false",
                "insufficient_authority|Boolean|nullable=false",
                "stale_evidence|Boolean|nullable=false",
                "provenance_json|Utf8|nullable=true",
            ],
        ),
    ];

    for (route, schema, expected) in route_schemas {
        assert_schema_route!(schema, route);
        assert_eq!(
            schema_signature!(schema),
            expected,
            "{route} schema changed"
        );
    }
}

#[test]
fn route_batch_builders_emit_conformant_rows() {
    let timestamp = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    let search = search_result_record_batch(&[FlightSearchResultRow {
        source_id: "legal-compliance-demo".to_string(),
        external_id: "legal/privacy/data-retention-clause".to_string(),
        title: "Example Privacy Code Article 12".to_string(),
        snippet: Some("retain audit evidence".to_string()),
        score: Some(1.0),
        authority_level: "Official".to_string(),
        canonical_uri: "https://law.example.test/privacy-code/article-12".to_string(),
        fetched_at: Some(timestamp),
        content_hash: "sha256:legal".to_string(),
        provenance_json: Some(r#"{"primary":{"source_id":"legal-compliance-demo"}}"#.to_string()),
        section_id: Some("article-12".to_string()),
        heading_path_json: Some(r#"["Privacy Code","Article 12"]"#.to_string()),
        source_kind: Some("LegalCorpus".to_string()),
        published_at: Some(timestamp),
        source_updated_at: Some(timestamp),
        trust_score: Some(1.0),
        freshness_score: Some(0.9),
        semantic_score: None,
        lexical_score: Some(1.0),
        rerank_score: Some(1.0),
        license_json: Some(r#"{"name":"Official Example License"}"#.to_string()),
        metadata_json: Some(r#"{"jurisdiction":"US-EXAMPLE"}"#.to_string()),
        doi: None,
        pmid: None,
        jurisdiction: Some("US-EXAMPLE".to_string()),
        evidence_kind: Some("law_clause".to_string()),
    }])
    .unwrap();
    assert_batch_route!(search, EXTERNAL_KNOWLEDGE_SEARCH_ROUTE);
    assert_eq!(
        compact_batch_snapshot(&search),
        r#"source_id=legal-compliance-demo
external_id=legal/privacy/data-retention-clause
title=Example Privacy Code Article 12
snippet=retain audit evidence
score=1
authority_level=Official
canonical_uri=https://law.example.test/privacy-code/article-12
fetched_at=1777723200000000000
content_hash=sha256:legal
provenance_json={"primary":{"source_id":"legal-compliance-demo"}}
section_id=article-12
heading_path_json=["Privacy Code","Article 12"]
source_kind=LegalCorpus
published_at=1777723200000000000
source_updated_at=1777723200000000000
trust_score=1
freshness_score=0.9
semantic_score=<null>
lexical_score=1
rerank_score=1
license_json={"name":"Official Example License"}
metadata_json={"jurisdiction":"US-EXAMPLE"}
doi=<null>
pmid=<null>
jurisdiction=US-EXAMPLE
evidence_kind=law_clause"#
    );

    let open = open_document_record_batch(&[FlightOpenDocumentRow {
        source_id: "legal-compliance-demo".to_string(),
        external_id: "legal/privacy/data-retention-clause".to_string(),
        canonical_uri: "https://law.example.test/privacy-code/article-12".to_string(),
        title: "Example Privacy Code Article 12".to_string(),
        section_id: Some("article-12".to_string()),
        heading_path_json: Some(r#"["Privacy Code","Article 12"]"#.to_string()),
        body: Some("Agencies retain audit evidence.".to_string()),
        metadata_json: Some(r#"{"article":"Article 12"}"#.to_string()),
        provenance_json: Some(r#"{"source_id":"legal-compliance-demo"}"#.to_string()),
    }])
    .unwrap();
    let sync = sync_result_record_batch(&[FlightSyncResultRow {
        job_id: "job-1".to_string(),
        source_id: "legal-compliance-demo".to_string(),
        job_kind: "Normalize".to_string(),
        status: "Succeeded".to_string(),
        cursor: Some("cursor-1".to_string()),
        dedup_hit: false,
        error: None,
    }])
    .unwrap();
    let status = status_record_batch(&[FlightStatusRow {
        source_id: "legal-compliance-demo".to_string(),
        enabled: true,
        last_success_at: Some(timestamp),
        last_seen_revision: Some("rev-1".to_string()),
        last_content_hash: Some("sha256:legal".to_string()),
        rate_limit_state: None,
    }])
    .unwrap();
    let compare = compare_result_record_batch(&[FlightCompareResultRow {
        claim: "retain audit evidence".to_string(),
        verdict: "evidence_available".to_string(),
        conflict_detected: false,
        insufficient_authority: false,
        stale_evidence: false,
        provenance_json: Some(r#"{"records":[{"source_id":"legal-compliance-demo"}]}"#.to_string()),
    }])
    .unwrap();

    assert_batch_route!(open, EXTERNAL_KNOWLEDGE_OPEN_ROUTE);
    assert_batch_route!(sync, EXTERNAL_KNOWLEDGE_SYNC_ROUTE);
    assert_batch_route!(status, EXTERNAL_KNOWLEDGE_STATUS_ROUTE);
    assert_batch_route!(compare, EXTERNAL_KNOWLEDGE_COMPARE_ROUTE);
    assert!(compact_batch_snapshot(&open).contains("metadata_json={\"article\":\"Article 12\"}"));
    assert!(compact_batch_snapshot(&sync).contains("dedup_hit=false"));
    assert!(compact_batch_snapshot(&status).contains("enabled=true"));
    assert!(compact_batch_snapshot(&compare).contains("verdict=evidence_available"));
}
