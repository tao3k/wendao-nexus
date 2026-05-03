use std::collections::BTreeMap;

use chrono::Utc;
use wendao_nexus_core::{
    EVIDENCE_KIND_METADATA_KEY, EvidenceKind, ExtractedDocumentResource,
    ExtractedDocumentResourceSet, RawSourceDocument, SOURCE_METADATA_AMENDMENT_VERSION_KEY,
    SOURCE_METADATA_ARTICLE_KEY, SOURCE_METADATA_CROP_KEY, SOURCE_METADATA_DEMAND_SIGNAL_KEY,
    SOURCE_METADATA_DEPARTMENT_KEY, SOURCE_METADATA_DOCUMENT_KIND_KEY, SOURCE_METADATA_DOI_KEY,
    SOURCE_METADATA_EFFECTIVE_AT_KEY, SOURCE_METADATA_LICENSE_KEY,
    SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY, SOURCE_METADATA_OWNER_TEAM_KEY,
    SOURCE_METADATA_PRICE_DATE_KEY, SOURCE_METADATA_PUBLISHED_AT_KEY, SOURCE_METADATA_REGION_KEY,
    SOURCE_METADATA_REVISION_ID_KEY, SOURCE_METADATA_STATUTE_KEY,
    SOURCE_METADATA_SUPPLY_SIGNAL_KEY, SOURCE_METADATA_TITLE_KEY,
    SOURCE_METADATA_WEATHER_WINDOW_KEY, SourceMetadata,
};

#[test]
fn extracted_resource_contract_matches_wendao_document_extract_shape() {
    let payload = r#"{
        "sourcePath": "tenant/protocol.docx",
        "sourceFormat": "docx",
        "totalResources": 1,
        "totalPages": 3,
        "resources": [
            {
                "sourcePath": "tenant/protocol.docx",
                "resourceType": "document",
                "resourcePath": "pages/page-1.md",
                "pageIndex": 0,
                "caption": "Protocol",
                "content": "Parsed by Wendao attachment extraction.",
                "mimeType": "text/markdown",
                "status": "ok",
                "elementId": "body"
            }
        ]
    }"#;

    let resource_set: ExtractedDocumentResourceSet = serde_json::from_str(payload).unwrap();

    assert_eq!(resource_set.source_path, "tenant/protocol.docx");
    assert_eq!(resource_set.source_format, "docx");
    assert_eq!(resource_set.total_resources, 1);
    assert_eq!(resource_set.total_pages, 3);
    assert_eq!(resource_set.resources[0].element_id, "body");
}

#[test]
fn extracted_resource_row_allows_missing_optional_fields() {
    let payload = r#"{
        "resourceType": "text",
        "content": "Wendao parsed text"
    }"#;

    let resource: ExtractedDocumentResource = serde_json::from_str(payload).unwrap();

    assert_eq!(resource.source_path, None);
    assert_eq!(resource.resource_type, "text");
    assert_eq!(resource.content, "Wendao parsed text");
    assert_eq!(resource.page_index, 0);
    assert_eq!(resource.status, "");
}

#[test]
fn standard_source_metadata_keys_are_stable() {
    assert_eq!(SOURCE_METADATA_TITLE_KEY, "title");
    assert_eq!(SOURCE_METADATA_DOI_KEY, "doi");
    assert_eq!(SOURCE_METADATA_PUBLISHED_AT_KEY, "published_at");
    assert_eq!(SOURCE_METADATA_REVISION_ID_KEY, "revision_id");
    assert_eq!(SOURCE_METADATA_LICENSE_KEY, "license");
    assert_eq!(SOURCE_METADATA_EFFECTIVE_AT_KEY, "effective_at");
    assert_eq!(SOURCE_METADATA_STATUTE_KEY, "statute");
    assert_eq!(SOURCE_METADATA_ARTICLE_KEY, "article");
    assert_eq!(SOURCE_METADATA_AMENDMENT_VERSION_KEY, "amendment_version");
    assert_eq!(SOURCE_METADATA_REGION_KEY, "region");
    assert_eq!(SOURCE_METADATA_CROP_KEY, "crop");
    assert_eq!(SOURCE_METADATA_PRICE_DATE_KEY, "price_date");
    assert_eq!(SOURCE_METADATA_WEATHER_WINDOW_KEY, "weather_window");
    assert_eq!(SOURCE_METADATA_SUPPLY_SIGNAL_KEY, "supply_signal");
    assert_eq!(SOURCE_METADATA_DEMAND_SIGNAL_KEY, "demand_signal");
    assert_eq!(SOURCE_METADATA_DEPARTMENT_KEY, "department");
    assert_eq!(SOURCE_METADATA_DOCUMENT_KIND_KEY, "document_kind");
    assert_eq!(SOURCE_METADATA_OWNER_TEAM_KEY, "owner_team");
}

#[test]
fn raw_source_document_standard_metadata_helpers_read_values() {
    let raw = RawSourceDocument {
        source_id: "fixture-source".to_string(),
        external_id: "fixture-doc".to_string(),
        canonical_uri: "fixture://doc".to_string(),
        media_type: "text/plain".to_string(),
        payload: b"fixture".to_vec(),
        fetched_at: Utc::now(),
        source_updated_at: None,
        content_hash: None,
        metadata: BTreeMap::from([
            (
                SOURCE_METADATA_TITLE_KEY.to_string(),
                "Fixture Title".to_string(),
            ),
            (
                SOURCE_METADATA_DOI_KEY.to_string(),
                "10.1000/demo".to_string(),
            ),
            (
                SOURCE_METADATA_REVISION_ID_KEY.to_string(),
                "rev-1".to_string(),
            ),
            (
                SOURCE_METADATA_LICENSE_KEY.to_string(),
                "Fixture License".to_string(),
            ),
            (
                SOURCE_METADATA_ARTICLE_KEY.to_string(),
                "Article 12".to_string(),
            ),
            (
                SOURCE_METADATA_REGION_KEY.to_string(),
                "US-Midwest".to_string(),
            ),
            (
                SOURCE_METADATA_DEPARTMENT_KEY.to_string(),
                "clinical-operations".to_string(),
            ),
        ]),
    };

    assert_eq!(raw.title_metadata(), Some("Fixture Title"));
    assert_eq!(raw.doi_metadata(), Some("10.1000/demo"));
    assert_eq!(raw.revision_id_metadata(), Some("rev-1"));
    assert_eq!(raw.license_name_metadata(), Some("Fixture License"));
    assert_eq!(raw.article_metadata(), Some("Article 12"));
    assert_eq!(raw.region_metadata(), Some("US-Midwest"));
    assert_eq!(raw.department_metadata(), Some("clinical-operations"));
    assert_eq!(raw.pmid_metadata(), None);
}

#[test]
fn source_metadata_evidence_kind_defaults_to_document() {
    let metadata = SourceMetadata::default();

    assert_eq!(metadata.evidence_kind(), EvidenceKind::Document);
}

#[test]
fn source_metadata_evidence_kind_reads_source_pack_metadata() {
    let mut metadata = SourceMetadata::default();
    metadata.extra.insert(
        EVIDENCE_KIND_METADATA_KEY.to_string(),
        "law_clause".to_string(),
    );

    assert_eq!(metadata.evidence_kind(), EvidenceKind::LawClause);
}

#[test]
fn source_metadata_standard_extra_helpers_read_domain_values() {
    let mut metadata = SourceMetadata::default();
    metadata.extra.extend(BTreeMap::from([
        (
            SOURCE_METADATA_TITLE_KEY.to_string(),
            "Fixture Title".to_string(),
        ),
        (
            SOURCE_METADATA_ARTICLE_KEY.to_string(),
            "Article 12".to_string(),
        ),
        (
            SOURCE_METADATA_EFFECTIVE_AT_KEY.to_string(),
            "2026-04-01T00:00:00Z".to_string(),
        ),
        (
            SOURCE_METADATA_REGION_KEY.to_string(),
            "US-Midwest".to_string(),
        ),
        (SOURCE_METADATA_CROP_KEY.to_string(), "corn".to_string()),
        (
            SOURCE_METADATA_PRICE_DATE_KEY.to_string(),
            "2026-04-21".to_string(),
        ),
        (
            SOURCE_METADATA_SUPPLY_SIGNAL_KEY.to_string(),
            "tightening".to_string(),
        ),
        (
            SOURCE_METADATA_LICENSE_USAGE_POLICY_KEY.to_string(),
            "citation_allowed".to_string(),
        ),
    ]));

    assert_eq!(metadata.title_metadata(), Some("Fixture Title"));
    assert_eq!(metadata.article_metadata(), Some("Article 12"));
    assert_eq!(
        metadata.effective_at_metadata(),
        Some("2026-04-01T00:00:00Z")
    );
    assert_eq!(metadata.region_metadata(), Some("US-Midwest"));
    assert_eq!(metadata.crop_metadata(), Some("corn"));
    assert_eq!(metadata.price_date_metadata(), Some("2026-04-21"));
    assert_eq!(metadata.supply_signal_metadata(), Some("tightening"));
    assert_eq!(
        metadata.license_usage_policy_metadata(),
        Some("citation_allowed")
    );
    assert_eq!(metadata.demand_signal_metadata(), None);
}
