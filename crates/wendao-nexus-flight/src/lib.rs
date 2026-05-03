//! Arrow Flight protocol contracts for the independent `Wendao Nexus` service.
//!
//! This crate is the public wire boundary that `xiuxian` may consume. It does
//! not depend on `xiuxian-*` crates.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);

/// Arrow `RecordBatch` builders for Nexus Flight payloads.
pub mod batch;
/// Typed command encoding for `FlightDescriptor::cmd`.
pub mod command;
/// Flight metadata header names.
pub mod headers;
/// Thin handler-to-batch adapter for Wendao-side Flight routing.
pub mod provider;
/// Flight route constants and route parsing.
pub mod routes;
/// Arrow schemas for external knowledge batches.
pub mod schema;

pub use batch::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, compare_result_record_batch, open_document_record_batch,
    open_rows_from_document, search_result_record_batch, search_rows_from_response,
    status_record_batch, sync_result_record_batch,
};
pub use command::{
    NEXUS_FLIGHT_COMMAND_SCHEMA_VERSION, NexusFlightCommand, NexusFlightCommandError,
    NexusFlightStatusRequest, NexusFlightSyncRequest, command_descriptor_from_json,
};
pub use headers::{
    NEXUS_AUTHORITY_MIN_HEADER, NEXUS_PROVENANCE_SUMMARY_HEADER, NEXUS_ROUTE_HEADER,
    NEXUS_SOURCE_ID_HEADER,
};
pub use provider::{
    NexusFlightBatchProvider, NexusFlightCommandHandler, NexusFlightHandlerError,
    NexusFlightProviderError,
};
pub use routes::{
    EXTERNAL_KNOWLEDGE_COMPARE_ROUTE, EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
    EXTERNAL_KNOWLEDGE_SYNC_ROUTE, NexusFlightRoute,
};
pub use schema::{
    NEXUS_FLIGHT_ROUTE_METADATA_KEY, NEXUS_FLIGHT_SCHEMA_VERSION,
    NEXUS_FLIGHT_SCHEMA_VERSION_METADATA_KEY, compare_result_schema, open_document_schema,
    search_result_schema, status_schema, sync_result_schema,
};
