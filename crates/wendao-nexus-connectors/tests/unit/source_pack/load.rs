use wendao_nexus_connectors::{
    SOURCE_PACK_MANIFEST_SCHEMA_VERSION, SourcePack, SourcePackManifest,
};
use wendao_nexus_core::{
    AuthorityLevel, IdentifierKind, KnowledgeSourceConnector, KnowledgeSourceKind,
    SOURCE_METADATA_PMID_KEY, SourceDomain, SourceItemRef,
};

use super::fixtures::{fixture_manifest, json_fixture_manifest};

#[tokio::test]
async fn source_pack_loads_manifest_and_local_corpus_connectors() {
    let pack = SourcePack::from_path(fixture_manifest()).unwrap();

    assert_eq!(pack.manifest().source_pack.id, "medical-baseline-pack");
    assert_eq!(pack.manifest().source_pack.version, "2026.04-fixture");
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
    assert_eq!(pack.manifest().source_profiles.len(), 1);

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
fn source_pack_resolves_explicit_and_default_authority_profiles() {
    let pack = SourcePack::from_path(fixture_manifest()).unwrap();

    let pubmed = pack.source_authority_profile("demo-pubmed").unwrap();
    assert_eq!(pubmed.source_id, "demo-pubmed");
    assert_eq!(pubmed.domain, SourceDomain::Medical);
    assert_eq!(pubmed.authority_level, AuthorityLevel::PeerReviewed);
    assert_eq!(
        pubmed.license_policy.as_deref(),
        Some("PubMed Fixture License")
    );
    assert!(pubmed.expected_identifiers.contains(&IdentifierKind::Pmid));
    assert!(pubmed.expected_identifiers.contains(&IdentifierKind::Doi));

    let guideline = pack.source_authority_profile("demo-guideline").unwrap();
    assert_eq!(guideline.source_id, "demo-guideline");
    assert_eq!(guideline.domain, SourceDomain::Medical);
    assert_eq!(guideline.authority_level, AuthorityLevel::Curated);
    assert_eq!(guideline.license_policy.as_deref(), Some("Fixture License"));
    assert!(
        guideline
            .expected_identifiers
            .contains(&IdentifierKind::Pmid)
    );

    assert_eq!(pack.source_authority_profiles().len(), 2);
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
    assert_eq!(manifest.sources[1].fixture_path, "guideline.jsonl");
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
    assert_eq!(
        pack.source("demo-pubmed-json")
            .and_then(|source| source.display_name.as_deref()),
        Some("Demo PubMed Fixture JSON")
    );
    assert_eq!(
        pack.source("demo-pubmed-json")
            .and_then(|source| source.license.as_deref()),
        Some("PubMed Fixture License JSON")
    );

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
