use wendao_nexus_core::{
    EvidenceKind, KnowledgeSourceKind, NexusSourceRecord, SOURCE_PACK_DISPLAY_NAME_METADATA_KEY,
    SOURCE_PACK_DOMAIN_METADATA_KEY, SOURCE_PACK_FIXTURE_PATH_METADATA_KEY,
    SOURCE_PACK_ID_METADATA_KEY, SOURCE_PACK_VERSION_METADATA_KEY, SourceCapabilities,
    SourceCheckpoint, SourceCursor, SourceDomain, SourceItemRef,
};

#[test]
fn evidence_kind_wire_labels_are_stable() {
    assert_eq!(EvidenceKind::default().as_str(), "document");
    assert_eq!(EvidenceKind::LawClause.as_str(), "law_clause");
    assert_eq!(EvidenceKind::MarketSignal.as_str(), "market_signal");
    assert_eq!(
        EvidenceKind::from_label("custom_signal"),
        EvidenceKind::Other("custom_signal".to_string())
    );
    assert_eq!(
        EvidenceKind::Other("custom_signal".to_string()).wire_label(),
        "other:custom_signal"
    );
    assert_eq!(
        serde_json::to_string(&EvidenceKind::CustomerInternalNote).unwrap(),
        r#""customer_internal_note""#
    );
    assert_eq!(
        serde_json::to_string(&EvidenceKind::Other("custom_signal".to_string())).unwrap(),
        r#""other:custom_signal""#
    );
    assert_eq!(
        serde_json::from_str::<EvidenceKind>(r#""trial_result""#).unwrap(),
        EvidenceKind::TrialResult
    );
}

#[test]
fn source_domain_wire_labels_and_default_are_stable() {
    assert_eq!(SourceDomain::default().as_str(), "generic");
    assert_eq!(SourceDomain::Agriculture.as_str(), "agriculture");
    assert_eq!(
        SourceDomain::from_label("custom_domain"),
        SourceDomain::Other("custom_domain".to_string())
    );
    assert_eq!(
        SourceDomain::Other("custom_domain".to_string()).wire_label(),
        "other:custom_domain"
    );
    assert_eq!(
        serde_json::to_string(&SourceDomain::WikipediaSubset).unwrap(),
        r#""wikipedia_subset""#
    );
    assert_eq!(
        serde_json::from_str::<SourceDomain>(r#""customer_private""#).unwrap(),
        SourceDomain::CustomerPrivate
    );
}

#[test]
fn source_record_source_pack_domain_reads_metadata_with_generic_default() {
    let mut record = NexusSourceRecord::new("legal-demo", KnowledgeSourceKind::LegalCorpus);

    assert_eq!(record.source_pack_domain(), SourceDomain::Generic);

    record.metadata.insert(
        SOURCE_PACK_DOMAIN_METADATA_KEY.to_string(),
        "legal".to_string(),
    );

    assert_eq!(record.source_pack_domain(), SourceDomain::Legal);
}

#[test]
fn source_record_source_pack_metadata_helpers_read_string_values() {
    let mut record = NexusSourceRecord::new("medical-demo", KnowledgeSourceKind::PubMed);

    assert_eq!(record.source_pack_id(), None);
    assert_eq!(record.source_pack_version(), None);
    assert_eq!(record.source_pack_display_name(), None);
    assert_eq!(record.source_pack_fixture_path(), None);

    record.metadata.insert(
        SOURCE_PACK_ID_METADATA_KEY.to_string(),
        "medical-demo-pack".to_string(),
    );
    record.metadata.insert(
        SOURCE_PACK_VERSION_METADATA_KEY.to_string(),
        "0.1.0".to_string(),
    );
    record.metadata.insert(
        SOURCE_PACK_DISPLAY_NAME_METADATA_KEY.to_string(),
        "Medical Demo Pack".to_string(),
    );
    record.metadata.insert(
        SOURCE_PACK_FIXTURE_PATH_METADATA_KEY.to_string(),
        "../corpus/medical/articles.jsonl".to_string(),
    );

    assert_eq!(record.source_pack_id(), Some("medical-demo-pack"));
    assert_eq!(record.source_pack_version(), Some("0.1.0"));
    assert_eq!(record.source_pack_display_name(), Some("Medical Demo Pack"));
    assert_eq!(
        record.source_pack_fixture_path(),
        Some("../corpus/medical/articles.jsonl")
    );
}

#[test]
fn mirror_fetch_capability_preserves_structured_metadata_boundary() {
    let capabilities = SourceCapabilities::mirror_fetch();

    assert!(capabilities.discover);
    assert!(capabilities.fetch);
    assert!(!capabilities.delta);
    assert!(capabilities.local_mirror);
    assert!(capabilities.structured_metadata);
    assert!(capabilities.license_metadata);
    assert!(!capabilities.access_control);
}

#[test]
fn local_corpus_fixture_capability_matches_file_backed_connector_contract() {
    let capabilities = SourceCapabilities::local_corpus_fixture();

    assert!(capabilities.discover);
    assert!(capabilities.fetch);
    assert!(capabilities.delta);
    assert!(!capabilities.live_query);
    assert!(capabilities.local_mirror);
    assert!(capabilities.revisions);
    assert!(capabilities.structured_metadata);
    assert!(capabilities.license_metadata);
    assert!(capabilities.access_control);
}

#[test]
fn checkpoint_and_item_refs_keep_connector_specific_state_opaque() {
    let mut checkpoint = SourceCheckpoint::new("pubmed");
    checkpoint.cursor = Some(SourceCursor::new("retstart=100"));

    let item = SourceItemRef::new("pubmed", "PMID:123");

    assert_eq!(checkpoint.source_id, "pubmed");
    assert_eq!(checkpoint.cursor.unwrap().value, "retstart=100");
    assert_eq!(item.external_id, "PMID:123");
}
