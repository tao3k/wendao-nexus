use std::path::PathBuf;

use wendao_nexus_connectors::{
    SOURCE_PACK_MANIFEST_SCHEMA_VERSION, SourcePack, validate_source_pack_export,
};
use wendao_nexus_core::{
    EVIDENCE_KIND_METADATA_KEY, KnowledgeSourceConnector, KnowledgeSourceKind, SourceDomain,
    SourceItemRef,
};

use crate::fixture_flight_support::{
    agriculture_pack_fixture_manifest, customer_private_pack_fixture_manifest,
    legal_pack_fixture_manifest, real_medical_pubmed_snapshot_fixture_manifest,
    real_wikipedia_science_subset_fixture_manifest, source_pack_fixture_manifest,
};

#[test]
fn directory_first_business_source_packs_are_conformant() {
    for case in source_pack_cases() {
        let pack = SourcePack::from_path(case.manifest()).unwrap();
        let pack_root = pack.manifest_path().parent().unwrap();

        for golden in [
            "source_pack.toml",
            "documents.jsonl",
            "expected_search.snap",
            "expected_open.snap",
            "expected_status.snap",
            "expected_compare.snap",
        ] {
            assert!(
                pack_root.join(golden).exists(),
                "{} is missing {golden}",
                case.id
            );
        }

        assert_eq!(pack.manifest().source_pack.id, case.id);
        assert_eq!(pack.manifest().source_pack.domain, case.domain);
        assert_eq!(
            pack.manifest().source_pack.schema_version,
            SOURCE_PACK_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(pack.manifest().sources.len(), case.source_count);
        assert_eq!(pack.connectors().len(), case.enabled_source_count);

        let records = pack.source_records();
        assert_eq!(records.len(), case.source_count);
        for record in records {
            assert_eq!(record.source_pack_id(), Some(case.id));
            assert_eq!(record.source_pack_schema_version(), Some("1"));
            assert_eq!(record.source_pack_domain(), case.domain);
            assert!(record.source_pack_fixture_path().is_some());
            assert_ne!(record.display_name.trim(), "");
            assert_ne!(record.license_policy.as_deref().unwrap_or("").trim(), "");
        }
    }
}

#[test]
fn source_pack_export_reports_are_conformant_for_business_packs() {
    for case in source_pack_cases() {
        let manifest = case.manifest();
        let pack_root = manifest.parent().unwrap();
        let report = validate_source_pack_export(pack_root).unwrap();

        assert_eq!(report.manifest_path, pack_root.join("source_pack.toml"));
        assert_eq!(report.pack_id, case.id);
        assert_eq!(report.domain, case.domain);
        assert_eq!(report.schema_version, SOURCE_PACK_MANIFEST_SCHEMA_VERSION);
        assert_eq!(report.source_count, case.source_count);
        assert_eq!(report.enabled_source_count, case.enabled_source_count);
        assert_eq!(report.fixture_paths.len(), case.source_count);
        assert!(
            report
                .fixture_paths
                .iter()
                .any(|fixture| fixture.source_id == case.source_id && fixture.enabled)
        );
        assert!(
            report
                .fixture_paths
                .iter()
                .all(|fixture| fixture.path.is_file())
        );
    }
}

#[test]
fn disabled_customer_source_remains_catalog_only() {
    let pack = SourcePack::from_path(customer_private_pack_fixture_manifest()).unwrap();

    let disabled = pack
        .source_records()
        .into_iter()
        .find(|record| record.source_id == "customer-crm-demo")
        .unwrap();
    assert!(!disabled.enabled);
    assert_eq!(disabled.source_kind, KnowledgeSourceKind::ApiFeed);
    assert!(pack.connector("customer-crm-demo").is_none());
}

#[tokio::test]
async fn local_corpus_lifecycle_preserves_vertical_evidence_metadata() {
    for case in source_pack_cases() {
        let pack = SourcePack::from_path(case.manifest()).unwrap();
        let connector = pack.connector(case.source_id).unwrap();
        let discovered = connector.discover(None).await.unwrap();
        assert!(
            discovered
                .items
                .iter()
                .any(|item| item.external_id == case.external_id),
            "{} did not discover {}",
            case.source_id,
            case.external_id
        );

        let document = connector
            .fetch(SourceItemRef::new(case.source_id, case.external_id))
            .await
            .unwrap();
        assert_eq!(
            document
                .metadata
                .get(EVIDENCE_KIND_METADATA_KEY)
                .map(String::as_str),
            Some(case.evidence_kind)
        );
        assert_eq!(document.source_id, case.source_id);
        assert_eq!(document.external_id, case.external_id);
        assert_ne!(document.canonical_uri.trim(), "");
    }
}

struct SourcePackCase {
    id: &'static str,
    domain: SourceDomain,
    source_count: usize,
    enabled_source_count: usize,
    manifest: fn() -> PathBuf,
    source_id: &'static str,
    external_id: &'static str,
    evidence_kind: &'static str,
}

impl SourcePackCase {
    fn manifest(&self) -> PathBuf {
        (self.manifest)()
    }
}

fn source_pack_cases() -> [SourcePackCase; 6] {
    [
        SourcePackCase {
            id: "medical-baseline-pack",
            domain: SourceDomain::Medical,
            source_count: 2,
            enabled_source_count: 2,
            manifest: source_pack_fixture_manifest,
            source_id: "demo-pubmed",
            external_id: "medical/pubmed-demo-1",
            evidence_kind: "trial_result",
        },
        SourcePackCase {
            id: "customer-private-sop-pack",
            domain: SourceDomain::CustomerPrivate,
            source_count: 2,
            enabled_source_count: 1,
            manifest: customer_private_pack_fixture_manifest,
            source_id: "customer-sop-demo",
            external_id: "customer/sop/clinical-trial-intake",
            evidence_kind: "customer_internal_note",
        },
        SourcePackCase {
            id: "legal-compliance-pack",
            domain: SourceDomain::Legal,
            source_count: 1,
            enabled_source_count: 1,
            manifest: legal_pack_fixture_manifest,
            source_id: "legal-compliance-demo",
            external_id: "legal/privacy/data-retention-clause",
            evidence_kind: "law_clause",
        },
        SourcePackCase {
            id: "agriculture-market-pack",
            domain: SourceDomain::Agriculture,
            source_count: 1,
            enabled_source_count: 1,
            manifest: agriculture_pack_fixture_manifest,
            source_id: "agriculture-market-demo",
            external_id: "agriculture/market/corn-midwest-weekly",
            evidence_kind: "market_signal",
        },
        SourcePackCase {
            id: "real-medical-pubmed-snapshot",
            domain: SourceDomain::Medical,
            source_count: 1,
            enabled_source_count: 1,
            manifest: real_medical_pubmed_snapshot_fixture_manifest,
            source_id: "real-pubmed-snapshot",
            external_id: "pubmed/PMID:37952131",
            evidence_kind: "trial_result",
        },
        SourcePackCase {
            id: "real-wikipedia-science-subset",
            domain: SourceDomain::WikipediaSubset,
            source_count: 1,
            enabled_source_count: 1,
            manifest: real_wikipedia_science_subset_fixture_manifest,
            source_id: "real-wikipedia-science",
            external_id: "wikipedia/CRISPR_gene_editing",
            evidence_kind: "definition",
        },
    ]
}
