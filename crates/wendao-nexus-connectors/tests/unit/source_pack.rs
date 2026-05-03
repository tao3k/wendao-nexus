use std::path::PathBuf;

use wendao_nexus_connectors::{
    SOURCE_PACK_MANIFEST_SCHEMA_VERSION, SourcePack, SourcePackManifest,
};
use wendao_nexus_core::{
    AuthorityLevel, KnowledgeSourceConnector, KnowledgeSourceKind, SOURCE_METADATA_PMID_KEY,
    SourceDomain, SourceItemRef,
};

#[tokio::test]
async fn source_pack_loads_manifest_and_local_corpus_connectors() {
    let pack = SourcePack::from_path(fixture_manifest()).unwrap();

    assert_eq!(pack.manifest().source_pack.id, "medical-demo-pack");
    assert_eq!(pack.manifest().source_pack.version, "0.1.0");
    assert_eq!(
        pack.manifest().source_pack.schema_version,
        SOURCE_PACK_MANIFEST_SCHEMA_VERSION
    );
    assert_eq!(
        pack.manifest().source_pack.producer.as_deref(),
        Some("wendao-nexus-fixtures")
    );
    assert_eq!(pack.manifest().source_pack.domain, SourceDomain::Medical);
    assert_eq!(
        pack.manifest().source_pack.authority_level,
        Some(AuthorityLevel::Curated)
    );
    assert_eq!(pack.connectors().len(), 2);

    let pubmed = pack.connector("demo-pubmed").unwrap();
    assert_eq!(pubmed.source_kind(), KnowledgeSourceKind::PubMed);
    let discovered = pubmed.discover(None).await.unwrap();
    assert_eq!(discovered.items.len(), 1);
    assert_eq!(discovered.items[0].external_id, "medical/pubmed-demo-1");

    let article = pubmed
        .fetch(SourceItemRef::new("demo-pubmed", "medical/pubmed-demo-1"))
        .await
        .unwrap();
    assert_eq!(
        article
            .metadata
            .get(SOURCE_METADATA_PMID_KEY)
            .map(String::as_str),
        Some("PMID:DEMO1")
    );
    assert_eq!(
        article.canonical_uri,
        "https://pubmed.ncbi.nlm.nih.gov/fixture-demo-1/"
    );

    let guideline_source = pack.source("demo-guideline").unwrap();
    assert_eq!(
        guideline_source.authority_level,
        Some(AuthorityLevel::Curated)
    );
    let guideline = pack
        .connector("demo-guideline")
        .unwrap()
        .fetch(SourceItemRef::new(
            "demo-guideline",
            "medical/guideline-demo",
        ))
        .await
        .unwrap();
    assert_eq!(
        guideline.canonical_uri,
        "local-corpus://demo-guideline/medical/guideline-demo"
    );
}

#[test]
fn source_pack_manifest_can_be_parsed_without_loading_connectors() {
    let manifest = SourcePackManifest::from_path(fixture_manifest()).unwrap();

    assert_eq!(manifest.sources.len(), 2);
    assert_eq!(manifest.source_pack.domain, SourceDomain::Medical);
    assert_eq!(
        manifest.source_pack.schema_version,
        SOURCE_PACK_MANIFEST_SCHEMA_VERSION
    );
    assert_eq!(manifest.sources[0].source_id, "demo-pubmed");
    assert_eq!(
        manifest.sources[1].fixture_path,
        "../corpus/medical/guideline.md"
    );
}

#[tokio::test]
async fn source_pack_json_manifest_loads_like_toml_manifest() {
    let pack = SourcePack::from_path(json_fixture_manifest()).unwrap();

    assert_eq!(pack.manifest().source_pack.id, "medical-demo-pack-json");
    assert_eq!(pack.manifest().source_pack.domain, SourceDomain::Medical);
    assert_eq!(
        pack.manifest().source_pack.producer.as_deref(),
        Some("wendao-nexus-fixtures")
    );
    assert_eq!(pack.connectors().len(), 2);

    let pubmed = pack.connector("demo-pubmed-json").unwrap();
    let discovered = pubmed.discover(None).await.unwrap();
    assert_eq!(discovered.items.len(), 1);

    let article = pubmed
        .fetch(SourceItemRef::new(
            "demo-pubmed-json",
            "medical/pubmed-demo-1",
        ))
        .await
        .unwrap();
    assert_eq!(
        article.canonical_uri,
        "https://pubmed.ncbi.nlm.nih.gov/fixture-demo-1/"
    );
}

#[test]
fn source_pack_emits_source_registry_records() {
    let pack = SourcePack::from_path(fixture_manifest()).unwrap();
    let records = pack.source_records();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].source_id, "demo-pubmed");
    assert_eq!(records[0].source_kind, KnowledgeSourceKind::PubMed);
    assert_eq!(records[0].authority_level, AuthorityLevel::PeerReviewed);
    assert!(records[0].capabilities.discover);
    assert!(records[0].capabilities.fetch);
    assert!(records[0].capabilities.delta);
    assert!(!records[0].capabilities.live_query);
    assert!(records[0].capabilities.local_mirror);
    assert_eq!(
        records[0].license_policy.as_deref(),
        Some("Fixture License")
    );
    assert_eq!(records[0].source_pack_id(), Some("medical-demo-pack"));
    assert_eq!(records[0].source_pack_version(), Some("0.1.0"));
    assert_eq!(records[0].source_pack_schema_version(), Some("1"));
    assert_eq!(
        records[0].source_pack_producer(),
        Some("wendao-nexus-fixtures")
    );
    assert_eq!(
        records[0].source_pack_display_name(),
        Some("Medical Demo Pack")
    );
    assert_eq!(records[0].source_pack_domain(), SourceDomain::Medical);
    assert_eq!(
        records[1].source_pack_fixture_path(),
        Some("../corpus/medical/guideline.md")
    );
}

#[test]
fn source_pack_keeps_disabled_sources_in_registry_records_only() {
    let pack = SourcePack::from_path(disabled_source_manifest()).unwrap();
    let records = pack.source_records();

    assert_eq!(pack.manifest().source_pack.domain, SourceDomain::Generic);
    assert_eq!(
        pack.manifest().source_pack.schema_version,
        SOURCE_PACK_MANIFEST_SCHEMA_VERSION
    );
    assert_eq!(pack.connectors().len(), 1);
    assert!(pack.connector("enabled-guideline").is_some());
    assert!(pack.connector("disabled-pubmed").is_none());
    assert_eq!(records.len(), 2);

    let disabled = records
        .iter()
        .find(|record| record.source_id == "disabled-pubmed")
        .unwrap();
    assert!(!disabled.enabled);
    assert_eq!(disabled.authority_level, AuthorityLevel::PeerReviewed);

    let enabled = records
        .iter()
        .find(|record| record.source_id == "enabled-guideline")
        .unwrap();
    assert!(enabled.enabled);
}

#[test]
fn source_pack_rejects_all_disabled_sources_for_connector_loading() {
    let error = SourcePack::from_path(all_disabled_source_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("has no enabled sources"));
}

#[test]
fn source_pack_rejects_unsupported_manifest_extension() {
    let error = SourcePackManifest::from_path(fixture_manifest().with_extension("txt"))
        .unwrap_err()
        .to_string();

    assert!(error.contains("must use .toml or .json"));
}

#[test]
fn source_pack_rejects_duplicate_source_ids() {
    let error = SourcePackManifest::from_path(duplicate_source_id_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("duplicate source_id `duplicate-source`"));
}

#[test]
fn source_pack_rejects_whitespace_padded_source_ids() {
    let error = SourcePackManifest::from_path(whitespace_source_id_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("source_id ` padded-source ` must not contain"));
}

#[test]
fn source_pack_rejects_whitespace_padded_fixture_paths() {
    let error = SourcePackManifest::from_path(whitespace_fixture_path_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("fixture_path must not contain"));
}

#[test]
fn source_pack_rejects_unsupported_schema_version() {
    let error = SourcePackManifest::from_path(unsupported_schema_version_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("schema_version 2 is unsupported"));
    assert!(error.contains("expected 1"));
}

#[test]
fn source_pack_rejects_whitespace_padded_producer() {
    let error = SourcePackManifest::from_path(whitespace_producer_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("producer ` fixture-builder ` must not contain"));
}

#[test]
fn source_pack_rejects_whitespace_padded_canonical_uri_prefix() {
    let error = SourcePackManifest::from_path(whitespace_canonical_uri_prefix_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("canonical_uri_prefix ` https://pubmed.ncbi.nlm.nih.gov/ `"));
    assert!(error.contains("must not contain leading or trailing whitespace"));
}

#[test]
fn source_pack_rejects_empty_display_name() {
    let error = SourcePackManifest::from_path(empty_display_name_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("display_name must not be empty"));
}

#[test]
fn source_pack_rejects_whitespace_padded_license() {
    let error = SourcePackManifest::from_path(whitespace_license_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("license ` Fixture License ` must not contain"));
}

fn fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/source_packs/medical_demo_pack.toml")
}

fn json_fixture_manifest() -> PathBuf {
    fixture_manifest().with_extension("json")
}

fn disabled_source_manifest() -> PathBuf {
    fixture_manifest().with_file_name("disabled_source.toml")
}

fn all_disabled_source_manifest() -> PathBuf {
    fixture_manifest().with_file_name("all_disabled.toml")
}

fn duplicate_source_id_manifest() -> PathBuf {
    fixture_manifest().with_file_name("duplicate_source_id.toml")
}

fn whitespace_source_id_manifest() -> PathBuf {
    fixture_manifest().with_file_name("whitespace_source_id.toml")
}

fn whitespace_fixture_path_manifest() -> PathBuf {
    fixture_manifest().with_file_name("whitespace_fixture_path.toml")
}

fn unsupported_schema_version_manifest() -> PathBuf {
    fixture_manifest().with_file_name("unsupported_schema_version.toml")
}

fn whitespace_producer_manifest() -> PathBuf {
    fixture_manifest().with_file_name("whitespace_producer.toml")
}

fn whitespace_canonical_uri_prefix_manifest() -> PathBuf {
    fixture_manifest().with_file_name("whitespace_canonical_uri_prefix.toml")
}

fn empty_display_name_manifest() -> PathBuf {
    fixture_manifest().with_file_name("empty_display_name.toml")
}

fn whitespace_license_manifest() -> PathBuf {
    fixture_manifest().with_file_name("whitespace_license.toml")
}
