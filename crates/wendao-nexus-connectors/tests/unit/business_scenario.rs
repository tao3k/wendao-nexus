use std::path::PathBuf;

use wendao_nexus_connectors::SourcePack;
use wendao_nexus_core::{
    AuthorityLevel, EVIDENCE_KIND_METADATA_KEY, KnowledgeSourceConnector, KnowledgeSourceKind,
    SOURCE_METADATA_ACL_TAGS_KEY, SOURCE_METADATA_DOCUMENT_KIND_KEY,
    SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY, SOURCE_METADATA_TENANT_ID_KEY, SourceDomain,
    SourceItemRef,
};

#[tokio::test]
async fn customer_private_knowledge_pack_preserves_business_metadata() {
    let pack = SourcePack::from_path(customer_private_pack_manifest()).unwrap();

    assert_eq!(
        pack.manifest().source_pack.id,
        "customer-private-knowledge-pack"
    );
    assert_eq!(
        pack.manifest().source_pack.authority_level,
        Some(AuthorityLevel::CustomerInternal)
    );
    assert_eq!(pack.connectors().len(), 1);
    assert!(pack.connector("customer-crm-demo").is_none());

    let records = pack.source_records();
    let sop_record = records
        .iter()
        .find(|record| record.source_id == "customer-sop-demo")
        .unwrap();
    assert_eq!(
        sop_record.source_kind,
        KnowledgeSourceKind::CustomerPrivateCorpus
    );
    assert_eq!(sop_record.authority_level, AuthorityLevel::CustomerInternal);
    assert_eq!(
        sop_record.license_policy.as_deref(),
        Some("Customer Confidential")
    );
    assert!(sop_record.enabled);
    assert!(sop_record.capabilities.access_control);
    assert!(!sop_record.capabilities.live_query);
    assert_eq!(
        sop_record.source_pack_id(),
        Some("customer-private-knowledge-pack")
    );
    assert_eq!(
        sop_record.source_pack_display_name(),
        Some("Customer Private Knowledge Pack")
    );
    assert_eq!(
        sop_record.source_pack_fixture_path(),
        Some("../corpus/customer/sop.jsonl")
    );

    let crm_record = records
        .iter()
        .find(|record| record.source_id == "customer-crm-demo")
        .unwrap();
    assert_eq!(crm_record.source_kind, KnowledgeSourceKind::ApiFeed);
    assert!(!crm_record.enabled);

    let connector = pack.connector("customer-sop-demo").unwrap();
    let discovered = connector.discover(None).await.unwrap();
    assert_eq!(discovered.items.len(), 2);
    assert_eq!(
        discovered.items[0]
            .metadata
            .get(SOURCE_METADATA_TENANT_ID_KEY)
            .map(String::as_str),
        Some("acme-bio")
    );
    assert_eq!(
        discovered.items[0]
            .metadata
            .get(SOURCE_METADATA_DOCUMENT_KIND_KEY)
            .map(String::as_str),
        Some("SOP")
    );

    let intake = connector
        .fetch(SourceItemRef::new(
            "customer-sop-demo",
            "customer/sop/clinical-trial-intake",
        ))
        .await
        .unwrap();
    assert_eq!(intake.title_metadata(), Some("Clinical Trial Intake SOP"));
    assert_eq!(intake.department_metadata(), Some("clinical-operations"));
    assert_eq!(intake.version_metadata(), Some("3.2.0"));
    assert_eq!(
        intake
            .metadata
            .get(SOURCE_METADATA_ACL_TAGS_KEY)
            .map(String::as_str),
        Some("tenant:acme-bio; department:clinical-operations; role:qa-reviewer")
    );
    assert_eq!(
        intake
            .metadata
            .get(SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY)
            .map(String::as_str),
        Some("internal_only")
    );
    assert!(intake.source_updated_at.is_some());
}

#[tokio::test]
async fn legal_compliance_pack_preserves_law_clause_metadata() {
    let pack = SourcePack::from_path(legal_compliance_pack_manifest()).unwrap();

    assert_eq!(pack.manifest().source_pack.id, "legal-compliance-pack");
    assert_eq!(pack.manifest().source_pack.domain, SourceDomain::Legal);
    assert_eq!(
        pack.manifest().source_pack.authority_level,
        Some(AuthorityLevel::Official)
    );

    let record = pack.source_records().pop().unwrap();
    assert_eq!(record.source_id, "legal-compliance-demo");
    assert_eq!(record.source_kind, KnowledgeSourceKind::LegalCorpus);
    assert_eq!(record.authority_level, AuthorityLevel::Official);
    assert_eq!(record.source_pack_domain(), SourceDomain::Legal);

    let connector = pack.connector("legal-compliance-demo").unwrap();
    let discovered = connector.discover(None).await.unwrap();
    assert_eq!(discovered.items.len(), 2);

    let clause = connector
        .fetch(SourceItemRef::new(
            "legal-compliance-demo",
            "legal/privacy/data-retention-clause",
        ))
        .await
        .unwrap();
    assert_eq!(clause.jurisdiction_metadata(), Some("US-EXAMPLE"));
    assert_eq!(clause.article_metadata(), Some("Article 12"));
    assert_eq!(clause.effective_at_metadata(), Some("2026-04-01T00:00:00Z"));
    assert_eq!(clause.amendment_version_metadata(), Some("2026-A"));
    assert_eq!(
        clause
            .metadata
            .get(EVIDENCE_KIND_METADATA_KEY)
            .map(String::as_str),
        Some("law_clause")
    );
}

#[tokio::test]
async fn agriculture_market_pack_preserves_market_signal_metadata() {
    let pack = SourcePack::from_path(agriculture_market_pack_manifest()).unwrap();

    assert_eq!(pack.manifest().source_pack.id, "agriculture-market-pack");
    assert_eq!(
        pack.manifest().source_pack.domain,
        SourceDomain::Agriculture
    );
    assert_eq!(
        pack.manifest().source_pack.authority_level,
        Some(AuthorityLevel::Official)
    );

    let record = pack.source_records().pop().unwrap();
    assert_eq!(record.source_id, "agriculture-market-demo");
    assert_eq!(record.source_kind, KnowledgeSourceKind::GovernmentDatabase);
    assert_eq!(record.authority_level, AuthorityLevel::Official);
    assert_eq!(record.source_pack_domain(), SourceDomain::Agriculture);

    let connector = pack.connector("agriculture-market-demo").unwrap();
    let discovered = connector.discover(None).await.unwrap();
    assert_eq!(discovered.items.len(), 2);

    let signal = connector
        .fetch(SourceItemRef::new(
            "agriculture-market-demo",
            "agriculture/market/corn-midwest-weekly",
        ))
        .await
        .unwrap();
    assert_eq!(signal.region_metadata(), Some("US-Midwest"));
    assert_eq!(signal.crop_metadata(), Some("corn"));
    assert_eq!(signal.price_date_metadata(), Some("2026-04-21"));
    assert_eq!(signal.weather_window_metadata(), Some("dry_7_day"));
    assert_eq!(signal.supply_signal_metadata(), Some("tightening"));
    assert_eq!(
        signal
            .metadata
            .get(EVIDENCE_KIND_METADATA_KEY)
            .map(String::as_str),
        Some("market_signal")
    );
}

fn customer_private_pack_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/source_packs/customer_private_knowledge_pack.toml")
}

fn legal_compliance_pack_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/source_packs/legal_compliance_pack.toml")
}

fn agriculture_market_pack_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/source_packs/agriculture_market_pack.toml")
}
