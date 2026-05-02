//! Arrow schemas for `Wendao Nexus` Flight payloads.

use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};

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

/// Schema for `/knowledge/external/search` result batches.
pub fn search_result_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
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
    ]))
}

/// Schema for `/knowledge/external/open` document batches.
pub fn open_document_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("source_id", DataType::Utf8, false),
        Field::new("external_id", DataType::Utf8, false),
        Field::new("canonical_uri", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("section_id", DataType::Utf8, true),
        Field::new("heading_path_json", DataType::Utf8, true),
        Field::new("body", DataType::Utf8, true),
        Field::new("metadata_json", DataType::Utf8, true),
        provenance_json_field(),
    ]))
}

/// Schema for `/knowledge/external/sync` result batches.
pub fn sync_result_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("job_id", DataType::Utf8, false),
        Field::new("source_id", DataType::Utf8, false),
        Field::new("job_kind", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("cursor", DataType::Utf8, true),
        Field::new("dedup_hit", DataType::Boolean, false),
        Field::new("error", DataType::Utf8, true),
    ]))
}

/// Schema for `/knowledge/external/status` batches.
pub fn status_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("source_id", DataType::Utf8, false),
        Field::new("enabled", DataType::Boolean, false),
        Field::new(
            "last_success_at",
            DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
            true,
        ),
        Field::new("last_seen_revision", DataType::Utf8, true),
        Field::new("last_content_hash", DataType::Utf8, true),
        Field::new("rate_limit_state", DataType::Utf8, true),
    ]))
}

/// Schema for `/knowledge/external/compare` evidence conflict batches.
pub fn compare_result_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("claim", DataType::Utf8, false),
        Field::new("verdict", DataType::Utf8, false),
        Field::new("conflict_detected", DataType::Boolean, false),
        Field::new("insufficient_authority", DataType::Boolean, false),
        Field::new("stale_evidence", DataType::Boolean, false),
        provenance_json_field(),
    ]))
}
