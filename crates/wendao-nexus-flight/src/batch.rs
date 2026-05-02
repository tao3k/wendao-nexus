//! Arrow `RecordBatch` builders for `Wendao Nexus` Flight routes.

use std::sync::Arc;

use arrow_array::builder::{
    BooleanBuilder, Float64Builder, StringBuilder, TimestampNanosecondBuilder,
};
use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::{ArrowError, DataType, TimeUnit};
use chrono::{DateTime, Utc};
use wendao_nexus_core::{
    EvidenceRecord, ExternalKnowledgeDocument, ExternalKnowledgeSearchResponse, NexusJobRecord,
    SourceCheckpoint,
};

use crate::schema::{
    compare_result_schema, open_document_schema, search_result_schema, status_schema,
    sync_result_schema,
};

/// Search result row for `/knowledge/external/search`.
#[derive(Clone, Debug, PartialEq)]
pub struct FlightSearchResultRow {
    pub source_id: String,
    pub external_id: String,
    pub title: String,
    pub snippet: Option<String>,
    pub score: Option<f64>,
    pub authority_level: String,
    pub canonical_uri: String,
    pub fetched_at: Option<DateTime<Utc>>,
    pub content_hash: String,
    pub provenance_json: Option<String>,
}

impl TryFrom<&EvidenceRecord> for FlightSearchResultRow {
    type Error = serde_json::Error;

    fn try_from(record: &EvidenceRecord) -> Result<Self, Self::Error> {
        let primary = &record.provenance.primary;
        Ok(Self {
            source_id: record.source_id.clone(),
            external_id: record.external_id.clone(),
            title: record.title.clone(),
            snippet: Some(record.snippet.clone()),
            score: record
                .score
                .as_deref()
                .and_then(|score| score.parse::<f64>().ok()),
            authority_level: format!("{:?}", primary.authority_level),
            canonical_uri: primary.canonical_uri.clone(),
            fetched_at: Some(primary.fetched_at),
            content_hash: primary.content_hash.clone(),
            provenance_json: Some(serde_json::to_string(&record.provenance)?),
        })
    }
}

/// Open-document row for `/knowledge/external/open`.
#[derive(Clone, Debug, PartialEq)]
pub struct FlightOpenDocumentRow {
    pub source_id: String,
    pub external_id: String,
    pub canonical_uri: String,
    pub title: String,
    pub section_id: Option<String>,
    pub heading_path_json: Option<String>,
    pub body: Option<String>,
    pub metadata_json: Option<String>,
    pub provenance_json: Option<String>,
}

/// Convert a core search response into Flight search rows.
pub fn search_rows_from_response(
    response: &ExternalKnowledgeSearchResponse,
) -> Result<Vec<FlightSearchResultRow>, serde_json::Error> {
    response.records.iter().map(TryFrom::try_from).collect()
}

/// Convert a normalized document into Flight open rows.
pub fn open_rows_from_document(
    document: &ExternalKnowledgeDocument,
    include_sections: bool,
    include_provenance: bool,
) -> Result<Vec<FlightOpenDocumentRow>, serde_json::Error> {
    if include_sections && !document.sections.is_empty() {
        document
            .sections
            .iter()
            .map(|section| {
                Ok(FlightOpenDocumentRow {
                    source_id: document.source_id.clone(),
                    external_id: document.external_id.clone(),
                    canonical_uri: document.canonical_uri.clone(),
                    title: document.title.clone(),
                    section_id: Some(section.section_id.clone()),
                    heading_path_json: Some(serde_json::to_string(&section.heading_path)?),
                    body: Some(section.text.clone()),
                    metadata_json: Some(serde_json::to_string(&document.metadata)?),
                    provenance_json: optional_provenance_json(document, include_provenance)?,
                })
            })
            .collect()
    } else {
        Ok(vec![FlightOpenDocumentRow {
            source_id: document.source_id.clone(),
            external_id: document.external_id.clone(),
            canonical_uri: document.canonical_uri.clone(),
            title: document.title.clone(),
            section_id: None,
            heading_path_json: None,
            body: Some(document.body.clone()),
            metadata_json: Some(serde_json::to_string(&document.metadata)?),
            provenance_json: optional_provenance_json(document, include_provenance)?,
        }])
    }
}

/// Sync job result row for `/knowledge/external/sync`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightSyncResultRow {
    pub job_id: String,
    pub source_id: String,
    pub job_kind: String,
    pub status: String,
    pub cursor: Option<String>,
    pub dedup_hit: bool,
    pub error: Option<String>,
}

impl From<&NexusJobRecord> for FlightSyncResultRow {
    fn from(job: &NexusJobRecord) -> Self {
        Self {
            job_id: job.job_id.to_string(),
            source_id: job.source_id.clone(),
            job_kind: format!("{:?}", job.job_kind),
            status: format!("{:?}", job.status),
            cursor: job.cursor.as_ref().map(|cursor| cursor.value.clone()),
            dedup_hit: job.dedup_hit,
            error: job.error.clone(),
        }
    }
}

/// Source status row for `/knowledge/external/status`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightStatusRow {
    pub source_id: String,
    pub enabled: bool,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_seen_revision: Option<String>,
    pub last_content_hash: Option<String>,
    pub rate_limit_state: Option<String>,
}

impl From<&SourceCheckpoint> for FlightStatusRow {
    fn from(checkpoint: &SourceCheckpoint) -> Self {
        Self {
            source_id: checkpoint.source_id.clone(),
            enabled: true,
            last_success_at: checkpoint.last_success_at,
            last_seen_revision: checkpoint.last_seen_revision.clone(),
            last_content_hash: checkpoint.last_content_hash.clone(),
            rate_limit_state: checkpoint.rate_limit_state.clone(),
        }
    }
}

/// Claim comparison result row for `/knowledge/external/compare`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightCompareResultRow {
    pub claim: String,
    pub verdict: String,
    pub conflict_detected: bool,
    pub insufficient_authority: bool,
    pub stale_evidence: bool,
    pub provenance_json: Option<String>,
}

/// Build a search result `RecordBatch`.
pub fn search_result_record_batch(
    rows: &[FlightSearchResultRow],
) -> Result<RecordBatch, ArrowError> {
    let mut source_ids = StringBuilder::new();
    let mut external_ids = StringBuilder::new();
    let mut titles = StringBuilder::new();
    let mut snippets = StringBuilder::new();
    let mut scores = Float64Builder::new();
    let mut authority_levels = StringBuilder::new();
    let mut canonical_uris = StringBuilder::new();
    let mut fetched_at = utc_timestamp_builder();
    let mut content_hashes = StringBuilder::new();
    let mut provenance_json = StringBuilder::new();

    for row in rows {
        source_ids.append_value(&row.source_id);
        external_ids.append_value(&row.external_id);
        titles.append_value(&row.title);
        append_optional_string(&mut snippets, row.snippet.as_deref());
        append_optional_f64(&mut scores, row.score);
        authority_levels.append_value(&row.authority_level);
        canonical_uris.append_value(&row.canonical_uri);
        append_optional_timestamp(&mut fetched_at, row.fetched_at);
        content_hashes.append_value(&row.content_hash);
        append_optional_string(&mut provenance_json, row.provenance_json.as_deref());
    }

    RecordBatch::try_new(
        search_result_schema(),
        vec![
            Arc::new(source_ids.finish()) as ArrayRef,
            Arc::new(external_ids.finish()) as ArrayRef,
            Arc::new(titles.finish()) as ArrayRef,
            Arc::new(snippets.finish()) as ArrayRef,
            Arc::new(scores.finish()) as ArrayRef,
            Arc::new(authority_levels.finish()) as ArrayRef,
            Arc::new(canonical_uris.finish()) as ArrayRef,
            Arc::new(fetched_at.finish()) as ArrayRef,
            Arc::new(content_hashes.finish()) as ArrayRef,
            Arc::new(provenance_json.finish()) as ArrayRef,
        ],
    )
}

/// Build an open-document `RecordBatch`.
pub fn open_document_record_batch(
    rows: &[FlightOpenDocumentRow],
) -> Result<RecordBatch, ArrowError> {
    let mut source_ids = StringBuilder::new();
    let mut external_ids = StringBuilder::new();
    let mut canonical_uris = StringBuilder::new();
    let mut titles = StringBuilder::new();
    let mut section_ids = StringBuilder::new();
    let mut heading_paths = StringBuilder::new();
    let mut bodies = StringBuilder::new();
    let mut metadata_json = StringBuilder::new();
    let mut provenance_json = StringBuilder::new();

    for row in rows {
        source_ids.append_value(&row.source_id);
        external_ids.append_value(&row.external_id);
        canonical_uris.append_value(&row.canonical_uri);
        titles.append_value(&row.title);
        append_optional_string(&mut section_ids, row.section_id.as_deref());
        append_optional_string(&mut heading_paths, row.heading_path_json.as_deref());
        append_optional_string(&mut bodies, row.body.as_deref());
        append_optional_string(&mut metadata_json, row.metadata_json.as_deref());
        append_optional_string(&mut provenance_json, row.provenance_json.as_deref());
    }

    RecordBatch::try_new(
        open_document_schema(),
        vec![
            Arc::new(source_ids.finish()) as ArrayRef,
            Arc::new(external_ids.finish()) as ArrayRef,
            Arc::new(canonical_uris.finish()) as ArrayRef,
            Arc::new(titles.finish()) as ArrayRef,
            Arc::new(section_ids.finish()) as ArrayRef,
            Arc::new(heading_paths.finish()) as ArrayRef,
            Arc::new(bodies.finish()) as ArrayRef,
            Arc::new(metadata_json.finish()) as ArrayRef,
            Arc::new(provenance_json.finish()) as ArrayRef,
        ],
    )
}

/// Build a sync result `RecordBatch`.
pub fn sync_result_record_batch(rows: &[FlightSyncResultRow]) -> Result<RecordBatch, ArrowError> {
    let mut job_ids = StringBuilder::new();
    let mut source_ids = StringBuilder::new();
    let mut job_kinds = StringBuilder::new();
    let mut statuses = StringBuilder::new();
    let mut cursors = StringBuilder::new();
    let mut dedup_hits = BooleanBuilder::new();
    let mut errors = StringBuilder::new();

    for row in rows {
        job_ids.append_value(&row.job_id);
        source_ids.append_value(&row.source_id);
        job_kinds.append_value(&row.job_kind);
        statuses.append_value(&row.status);
        append_optional_string(&mut cursors, row.cursor.as_deref());
        dedup_hits.append_value(row.dedup_hit);
        append_optional_string(&mut errors, row.error.as_deref());
    }

    RecordBatch::try_new(
        sync_result_schema(),
        vec![
            Arc::new(job_ids.finish()) as ArrayRef,
            Arc::new(source_ids.finish()) as ArrayRef,
            Arc::new(job_kinds.finish()) as ArrayRef,
            Arc::new(statuses.finish()) as ArrayRef,
            Arc::new(cursors.finish()) as ArrayRef,
            Arc::new(dedup_hits.finish()) as ArrayRef,
            Arc::new(errors.finish()) as ArrayRef,
        ],
    )
}

/// Build a status `RecordBatch`.
pub fn status_record_batch(rows: &[FlightStatusRow]) -> Result<RecordBatch, ArrowError> {
    let mut source_ids = StringBuilder::new();
    let mut enabled = BooleanBuilder::new();
    let mut last_success_at = utc_timestamp_builder();
    let mut last_seen_revisions = StringBuilder::new();
    let mut last_content_hashes = StringBuilder::new();
    let mut rate_limit_states = StringBuilder::new();

    for row in rows {
        source_ids.append_value(&row.source_id);
        enabled.append_value(row.enabled);
        append_optional_timestamp(&mut last_success_at, row.last_success_at);
        append_optional_string(&mut last_seen_revisions, row.last_seen_revision.as_deref());
        append_optional_string(&mut last_content_hashes, row.last_content_hash.as_deref());
        append_optional_string(&mut rate_limit_states, row.rate_limit_state.as_deref());
    }

    RecordBatch::try_new(
        status_schema(),
        vec![
            Arc::new(source_ids.finish()) as ArrayRef,
            Arc::new(enabled.finish()) as ArrayRef,
            Arc::new(last_success_at.finish()) as ArrayRef,
            Arc::new(last_seen_revisions.finish()) as ArrayRef,
            Arc::new(last_content_hashes.finish()) as ArrayRef,
            Arc::new(rate_limit_states.finish()) as ArrayRef,
        ],
    )
}

/// Build a compare result `RecordBatch`.
pub fn compare_result_record_batch(
    rows: &[FlightCompareResultRow],
) -> Result<RecordBatch, ArrowError> {
    let mut claims = StringBuilder::new();
    let mut verdicts = StringBuilder::new();
    let mut conflict_detected = BooleanBuilder::new();
    let mut insufficient_authority = BooleanBuilder::new();
    let mut stale_evidence = BooleanBuilder::new();
    let mut provenance_json = StringBuilder::new();

    for row in rows {
        claims.append_value(&row.claim);
        verdicts.append_value(&row.verdict);
        conflict_detected.append_value(row.conflict_detected);
        insufficient_authority.append_value(row.insufficient_authority);
        stale_evidence.append_value(row.stale_evidence);
        append_optional_string(&mut provenance_json, row.provenance_json.as_deref());
    }

    RecordBatch::try_new(
        compare_result_schema(),
        vec![
            Arc::new(claims.finish()) as ArrayRef,
            Arc::new(verdicts.finish()) as ArrayRef,
            Arc::new(conflict_detected.finish()) as ArrayRef,
            Arc::new(insufficient_authority.finish()) as ArrayRef,
            Arc::new(stale_evidence.finish()) as ArrayRef,
            Arc::new(provenance_json.finish()) as ArrayRef,
        ],
    )
}

fn append_optional_string(builder: &mut StringBuilder, value: Option<&str>) {
    match value {
        Some(value) => builder.append_value(value),
        None => builder.append_null(),
    }
}

fn append_optional_f64(builder: &mut Float64Builder, value: Option<f64>) {
    match value {
        Some(value) => builder.append_value(value),
        None => builder.append_null(),
    }
}

fn append_optional_timestamp(
    builder: &mut TimestampNanosecondBuilder,
    value: Option<DateTime<Utc>>,
) {
    match value.and_then(|value| value.timestamp_nanos_opt()) {
        Some(value) => builder.append_value(value),
        None => builder.append_null(),
    }
}

fn utc_timestamp_builder() -> TimestampNanosecondBuilder {
    TimestampNanosecondBuilder::new().with_data_type(DataType::Timestamp(
        TimeUnit::Nanosecond,
        Some("UTC".into()),
    ))
}

fn optional_provenance_json(
    document: &ExternalKnowledgeDocument,
    include_provenance: bool,
) -> Result<Option<String>, serde_json::Error> {
    if include_provenance {
        Ok(Some(serde_json::to_string(&document.provenance)?))
    } else {
        Ok(None)
    }
}
