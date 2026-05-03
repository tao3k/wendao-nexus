//! Arrow schemas for `Wendao Nexus` Flight payloads.

use std::collections::HashMap;
use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};

use crate::routes::{
    EXTERNAL_KNOWLEDGE_COMPARE_ROUTE, EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
    EXTERNAL_KNOWLEDGE_SYNC_ROUTE,
};

/// Current stable schema version for Nexus Flight batches.
pub const NEXUS_FLIGHT_SCHEMA_VERSION: &str = "1";

/// Arrow schema metadata key for the Nexus schema version.
pub const NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY: &str = "wendao_nexus.schema_version";

/// Arrow schema metadata key for the canonical Nexus route.
pub const NEXUS_FLIGHT_ROUTE_METADATA_KEY: &str = "wendao_nexus.route";

fn fetched_at_field() -> Field {
    Field::new(
        "fetched_at",
        DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
        true,
    )
}

fn provenance_json_field() -> Field {
    Field::new("provenance_json", DataType::Utf8, true)
}

fn utc_timestamp_field(name: &str) -> Field {
    Field::new(
        name,
        DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
        true,
    )
}

fn nullable_utf8_field(name: &str) -> Field {
    Field::new(name, DataType::Utf8, true)
}

fn nullable_score_field(name: &str) -> Field {
    Field::new(name, DataType::Float64, true)
}

fn schema_metadata(route: &str) -> HashMap<String, String> {
    HashMap::from([
        (
            NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY.to_string(),
            NEXUS_FLIGHT_SCHEMA_VERSION.to_string(),
        ),
        (
            NEXUS_FLIGHT_ROUTE_METADATA_KEY.to_string(),
            route.to_string(),
        ),
    ])
}

/// Schema for `/knowledge/external/search` result batches.
pub fn search_result_schema() -> SchemaRef {
    Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("source_id", DataType::Utf8, false),
            Field::new("external_id", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, false),
            Field::new("snippet", DataType::Utf8, true),
            Field::new("score", DataType::Float64, true),
            Field::new("authority_level", DataType::Utf8, false),
            Field::new("canonical_uri", DataType::Utf8, false),
            fetched_at_field(),
            Field::new("content_hash", DataType::Utf8, false),
            provenance_json_field(),
            nullable_utf8_field("section_id"),
            nullable_utf8_field("heading_path_json"),
            nullable_utf8_field("source_kind"),
            utc_timestamp_field("published_at"),
            utc_timestamp_field("source_updated_at"),
            nullable_score_field("trust_score"),
            nullable_score_field("freshness_score"),
            nullable_score_field("semantic_score"),
            nullable_score_field("lexical_score"),
            nullable_score_field("rerank_score"),
            nullable_utf8_field("license_json"),
            nullable_utf8_field("metadata_json"),
            nullable_utf8_field("doi"),
            nullable_utf8_field("pmid"),
            nullable_utf8_field("jurisdiction"),
            nullable_utf8_field("evidence_kind"),
        ],
        schema_metadata(EXTERNAL_KNOWLEDGE_SEARCH_ROUTE),
    ))
}

/// Schema for `/knowledge/external/open` document batches.
pub fn open_document_schema() -> SchemaRef {
    Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("source_id", DataType::Utf8, false),
            Field::new("external_id", DataType::Utf8, false),
            Field::new("canonical_uri", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, false),
            Field::new("section_id", DataType::Utf8, true),
            Field::new("heading_path_json", DataType::Utf8, true),
            Field::new("body", DataType::Utf8, true),
            Field::new("metadata_json", DataType::Utf8, true),
            provenance_json_field(),
        ],
        schema_metadata(EXTERNAL_KNOWLEDGE_OPEN_ROUTE),
    ))
}

/// Schema for `/knowledge/external/sync` result batches.
pub fn sync_result_schema() -> SchemaRef {
    Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("job_id", DataType::Utf8, false),
            Field::new("source_id", DataType::Utf8, false),
            Field::new("job_kind", DataType::Utf8, false),
            Field::new("status", DataType::Utf8, false),
            Field::new("cursor", DataType::Utf8, true),
            Field::new("dedup_hit", DataType::Boolean, false),
            Field::new("error", DataType::Utf8, true),
        ],
        schema_metadata(EXTERNAL_KNOWLEDGE_SYNC_ROUTE),
    ))
}

/// Schema for `/knowledge/external/status` batches.
pub fn status_schema() -> SchemaRef {
    Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("source_id", DataType::Utf8, false),
            Field::new("enabled", DataType::Boolean, false),
            utc_timestamp_field("last_success_at"),
            Field::new("last_seen_revision", DataType::Utf8, true),
            Field::new("last_content_hash", DataType::Utf8, true),
            Field::new("rate_limit_state", DataType::Utf8, true),
        ],
        schema_metadata(EXTERNAL_KNOWLEDGE_STATUS_ROUTE),
    ))
}

/// Schema for `/knowledge/external/compare` evidence conflict batches.
pub fn compare_result_schema() -> SchemaRef {
    Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("claim", DataType::Utf8, false),
            Field::new("verdict", DataType::Utf8, false),
            Field::new("conflict_detected", DataType::Boolean, false),
            Field::new("insufficient_authority", DataType::Boolean, false),
            Field::new("stale_evidence", DataType::Boolean, false),
            provenance_json_field(),
        ],
        schema_metadata(EXTERNAL_KNOWLEDGE_COMPARE_ROUTE),
    ))
}
