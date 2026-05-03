//! Source connector implementations for Wendao Nexus.
//!
//! This crate owns source-specific adapter behavior. The first skeleton exposes
//! capability-accurate connectors and a deterministic static connector for
//! runtime tests.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);

/// Customer-owned private corpus connector boundary.
pub mod customer_corpus;
/// External database and API-feed connector boundary.
pub mod external_database;
/// File-backed deterministic corpus connector.
pub mod local_corpus;
/// PubMed connector boundary.
pub mod pubmed;
/// Deterministic source-pack manifest loader.
pub mod source_pack;
/// Deterministic connector for tests and local embedding.
pub mod static_connector;
/// Wikipedia or MediaWiki connector boundary.
pub mod wikipedia;

pub use customer_corpus::{CustomerCorpusConfig, CustomerCorpusConnector};
pub use external_database::{
    ExternalDatabaseAccessMode, ExternalDatabaseAuthMode, ExternalDatabaseConfig,
    ExternalDatabaseConnector,
};
pub use local_corpus::{LocalCorpusConfig, LocalCorpusConnector};
pub use pubmed::{PubMedConfig, PubMedConnector};
pub use source_pack::{SourcePack, SourcePackManifest, SourcePackMetadata, SourcePackSource};
pub use static_connector::StaticKnowledgeConnector;
pub use wikipedia::{WikipediaConfig, WikipediaConnector};
