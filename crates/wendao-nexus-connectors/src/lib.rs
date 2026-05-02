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
/// PubMed connector boundary.
pub mod pubmed;
/// Deterministic connector for tests and local embedding.
pub mod static_connector;
/// Wikipedia or MediaWiki connector boundary.
pub mod wikipedia;

pub use customer_corpus::{CustomerCorpusConfig, CustomerCorpusConnector};
pub use pubmed::{PubMedConfig, PubMedConnector};
pub use static_connector::StaticKnowledgeConnector;
pub use wikipedia::{WikipediaConfig, WikipediaConnector};
