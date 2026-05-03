#[path = "unit/business_scenario.rs"]
mod business_scenario;

#[path = "unit/external_database.rs"]
mod external_database;
#[cfg(feature = "live-probe")]
#[path = "unit/external_database_probe.rs"]
mod external_database_probe;

#[path = "unit/local_corpus.rs"]
mod local_corpus;
#[path = "unit/pubmed_wikipedia.rs"]
mod pubmed_wikipedia;
#[path = "unit/source_pack/mod.rs"]
mod source_pack;
#[path = "unit/static_connector.rs"]
mod static_connector;
