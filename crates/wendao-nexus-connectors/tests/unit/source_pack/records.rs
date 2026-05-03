use wendao_nexus_connectors::{SOURCE_PACK_MANIFEST_SCHEMA_VERSION, SourcePack};
use wendao_nexus_core::{AuthorityLevel, KnowledgeSourceKind, SourceDomain};

use super::fixtures::{disabled_source_manifest, fixture_manifest};

#[test]
fn source_pack_emits_source_registry_records() {
    let pack = SourcePack::from_path(fixture_manifest()).unwrap();
    let records = pack.source_records();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].source_id, "demo-pubmed");
    assert_eq!(records[0].display_name, "Demo PubMed Fixture");
    assert_eq!(records[0].source_kind, KnowledgeSourceKind::PubMed);
    assert_eq!(records[0].authority_level, AuthorityLevel::PeerReviewed);
    assert!(records[0].capabilities.discover);
    assert!(records[0].capabilities.fetch);
    assert!(records[0].capabilities.delta);
    assert!(!records[0].capabilities.live_query);
    assert!(records[0].capabilities.local_mirror);
    assert_eq!(
        records[0].license_policy.as_deref(),
        Some("PubMed Fixture License")
    );
    assert_eq!(
        records[1].license_policy.as_deref(),
        Some("Fixture License")
    );
    assert_eq!(records[0].source_pack_id(), Some("medical-baseline-pack"));
    assert_eq!(records[0].source_pack_version(), Some("2026.04-fixture"));
    assert_eq!(records[0].source_pack_schema_version(), Some("1"));
    assert_eq!(
        records[0].source_pack_producer(),
        Some("wendao-nexus-fixtures")
    );
    assert_eq!(
        records[0].source_pack_display_name(),
        Some("Medical Baseline Pack")
    );
    assert_eq!(records[0].source_pack_domain(), SourceDomain::Medical);
    assert_eq!(
        records[1].source_pack_fixture_path(),
        Some("guideline.jsonl")
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
