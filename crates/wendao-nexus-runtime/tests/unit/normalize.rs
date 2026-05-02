use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use wendao_nexus_core::{AuthorityLevel, KnowledgeSourceKind, RawSourceDocument};
use wendao_nexus_runtime::{
    KnowledgeDocumentNormalizer, NormalizationContext, PlainTextNormalizer,
};

#[tokio::test]
async fn plain_text_normalizer_preserves_metadata_and_provenance() {
    let fetched_at = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    let raw = RawSourceDocument {
        source_id: "pubmed".to_string(),
        external_id: "PMID:123".to_string(),
        canonical_uri: "https://pubmed.ncbi.nlm.nih.gov/123/".to_string(),
        media_type: "text/plain".to_string(),
        payload: b"GLP-1 cardiovascular evidence".to_vec(),
        fetched_at,
        source_updated_at: None,
        content_hash: Some("sha256:known".to_string()),
        metadata: BTreeMap::from([
            ("title".to_string(), "Trial Abstract".to_string()),
            (
                "authors".to_string(),
                "Ada Lovelace; Grace Hopper".to_string(),
            ),
            ("doi".to_string(), "10.1000/example".to_string()),
            ("pmid".to_string(), "123".to_string()),
            (
                "mesh_terms".to_string(),
                "Cardiology, Endocrinology".to_string(),
            ),
            ("license".to_string(), "Public Domain".to_string()),
        ]),
    };

    let document = PlainTextNormalizer
        .normalize(
            raw,
            NormalizationContext::new(KnowledgeSourceKind::PubMed, AuthorityLevel::PeerReviewed),
        )
        .await
        .unwrap();

    assert_eq!(document.title, "Trial Abstract");
    assert_eq!(document.body, "GLP-1 cardiovascular evidence");
    assert_eq!(document.sections[0].heading_path, vec!["Trial Abstract"]);
    assert_eq!(document.metadata.authors.len(), 2);
    assert_eq!(document.metadata.doi.as_deref(), Some("10.1000/example"));
    assert_eq!(
        document.provenance.authority_level,
        AuthorityLevel::PeerReviewed
    );
    assert_eq!(document.provenance.content_hash, "sha256:known");
    assert_eq!(document.license.unwrap().name, "Public Domain");
}

#[tokio::test]
async fn plain_text_normalizer_rejects_binary_payloads() {
    let raw = RawSourceDocument {
        source_id: "object-store".to_string(),
        external_id: "blob-1".to_string(),
        canonical_uri: "s3://bucket/blob-1".to_string(),
        media_type: "application/pdf".to_string(),
        payload: vec![0, 1, 2],
        fetched_at: Utc::now(),
        source_updated_at: None,
        content_hash: None,
        metadata: BTreeMap::new(),
    };

    let error = PlainTextNormalizer
        .normalize(
            raw,
            NormalizationContext::new(KnowledgeSourceKind::ObjectStorage, AuthorityLevel::Unknown),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("plain text normalization"));
}
