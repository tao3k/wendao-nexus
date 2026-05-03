use std::path::PathBuf;

use wendao_nexus_connectors::SourcePack;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceKind, KnowledgeSourceKind, NexusJobStatus, SourceItemRef,
};
use wendao_nexus_runtime::{
    InMemoryNexusRegistry, NexusSyncRuntime, NormalizationContext, PlainTextNormalizer,
};

#[tokio::test]
async fn customer_private_sop_ingests_as_customer_internal_evidence() {
    let pack = SourcePack::from_path(customer_private_pack_manifest()).unwrap();
    let connector = pack.connector("customer-sop-demo").unwrap();
    let runtime = NexusSyncRuntime::new(InMemoryNexusRegistry::new());

    let outcome = runtime
        .ingest_once(
            connector,
            SourceItemRef::new("customer-sop-demo", "customer/sop/clinical-trial-intake"),
            &PlainTextNormalizer,
            NormalizationContext::new(
                KnowledgeSourceKind::CustomerPrivateCorpus,
                AuthorityLevel::CustomerInternal,
            ),
        )
        .await
        .unwrap();

    assert_eq!(outcome.normalize_job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.document.title, "Clinical Trial Intake SOP");
    assert_eq!(
        outcome.document.metadata.tenant_id.as_deref(),
        Some("acme-bio")
    );
    assert!(
        outcome
            .document
            .metadata
            .acl_tags
            .iter()
            .any(|tag| tag == "role:qa-reviewer")
    );
    assert_eq!(
        outcome.document.provenance.authority_level,
        AuthorityLevel::CustomerInternal
    );
    assert_eq!(
        outcome.document.provenance.source_kind,
        KnowledgeSourceKind::CustomerPrivateCorpus
    );
    assert_eq!(
        outcome.document.provenance.version.as_deref(),
        Some("3.2.0")
    );
    assert_eq!(
        outcome
            .document
            .license
            .as_ref()
            .map(|license| license.name.as_str()),
        Some("Customer Confidential")
    );
    assert_eq!(
        outcome
            .document
            .license
            .as_ref()
            .and_then(|license| license.usage_policy.as_deref()),
        Some("internal_only")
    );
    assert!(
        outcome
            .document
            .body
            .contains("QA reviewer approval before activation")
    );
}

#[tokio::test]
async fn legal_law_clause_ingests_with_official_provenance() {
    let pack = SourcePack::from_path(legal_compliance_pack_manifest()).unwrap();
    let connector = pack.connector("legal-compliance-demo").unwrap();
    let runtime = NexusSyncRuntime::new(InMemoryNexusRegistry::new());

    let outcome = runtime
        .ingest_once(
            connector,
            SourceItemRef::new(
                "legal-compliance-demo",
                "legal/privacy/data-retention-clause",
            ),
            &PlainTextNormalizer,
            NormalizationContext::new(KnowledgeSourceKind::LegalCorpus, AuthorityLevel::Official),
        )
        .await
        .unwrap();

    assert_eq!(outcome.normalize_job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.document.title, "Example Privacy Code Article 12");
    assert_eq!(
        outcome.document.provenance.authority_level,
        AuthorityLevel::Official
    );
    assert_eq!(
        outcome.document.provenance.source_kind,
        KnowledgeSourceKind::LegalCorpus
    );
    assert_eq!(
        outcome.document.metadata.jurisdiction.as_deref(),
        Some("US-EXAMPLE")
    );
    assert_eq!(
        outcome.document.metadata.article_metadata(),
        Some("Article 12")
    );
    assert_eq!(
        outcome.document.metadata.effective_at_metadata(),
        Some("2026-04-01T00:00:00Z")
    );
    assert_eq!(
        outcome.document.metadata.evidence_kind(),
        EvidenceKind::LawClause
    );
}

#[tokio::test]
async fn agriculture_market_signal_ingests_with_official_provenance() {
    let pack = SourcePack::from_path(agriculture_market_pack_manifest()).unwrap();
    let connector = pack.connector("agriculture-market-demo").unwrap();
    let runtime = NexusSyncRuntime::new(InMemoryNexusRegistry::new());

    let outcome = runtime
        .ingest_once(
            connector,
            SourceItemRef::new(
                "agriculture-market-demo",
                "agriculture/market/corn-midwest-weekly",
            ),
            &PlainTextNormalizer,
            NormalizationContext::new(
                KnowledgeSourceKind::GovernmentDatabase,
                AuthorityLevel::Official,
            ),
        )
        .await
        .unwrap();

    assert_eq!(outcome.normalize_job.status, NexusJobStatus::Succeeded);
    assert_eq!(outcome.document.title, "Midwest Corn Weekly Market Signal");
    assert_eq!(
        outcome.document.provenance.authority_level,
        AuthorityLevel::Official
    );
    assert_eq!(
        outcome.document.provenance.source_kind,
        KnowledgeSourceKind::GovernmentDatabase
    );
    assert_eq!(
        outcome.document.metadata.region_metadata(),
        Some("US-Midwest")
    );
    assert_eq!(outcome.document.metadata.crop_metadata(), Some("corn"));
    assert_eq!(
        outcome.document.metadata.price_date_metadata(),
        Some("2026-04-21")
    );
    assert_eq!(
        outcome.document.metadata.evidence_kind(),
        EvidenceKind::MarketSignal
    );
}

fn customer_private_pack_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/customer_private_sop/source_pack.toml",
    )
}

fn legal_compliance_pack_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/legal_compliance/source_pack.toml",
    )
}

fn agriculture_market_pack_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../wendao-nexus-connectors/tests/fixtures/source_packs/agriculture_market/source_pack.toml")
}
