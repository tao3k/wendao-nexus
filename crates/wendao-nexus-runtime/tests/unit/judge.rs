use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use wendao_nexus_core::{
    AuthorityLevel, EvidenceKind, EvidenceWarning, ExternalKnowledgeDocument, IdentifierKind,
    KnowledgeSection, KnowledgeSourceKind, LicenseInfo, ProvenanceRecord,
    SOURCE_METADATA_ARTICLE_KEY, SOURCE_METADATA_CLINICAL_TRIAL_ID_KEY,
    SOURCE_METADATA_LICENSE_KEY, SOURCE_METADATA_REVISION_ID_KEY, SOURCE_METADATA_STATUTE_KEY,
    SourceAuthorityProfile, SourceDomain, SourceMetadata,
};
use wendao_nexus_runtime::{BasicEvidenceJudge, EvidenceJudge};

#[test]
fn basic_judge_scores_complete_medical_evidence_highly() {
    let document = fixture_document(
        "pubmed-probe",
        "PMID:37952131",
        KnowledgeSourceKind::PubMed,
        AuthorityLevel::PeerReviewed,
        SourceDomain::Medical,
    );
    let profile = SourceAuthorityProfile::for_source_pack_source(
        "pubmed-probe",
        SourceDomain::Medical,
        AuthorityLevel::PeerReviewed,
        Some("PubMed metadata".to_string()),
    );
    let judgement = BasicEvidenceJudge::at(ts("2026-05-01T00:00:00Z")).judge(&document, &profile);

    assert!(judgement.final_trust_score > 0.85);
    assert_eq!(judgement.freshness_score, 1.0);
    assert_eq!(judgement.identifier_score, 1.0);
    assert_eq!(judgement.warnings, Vec::new());
}

#[test]
fn basic_judge_warns_for_missing_medical_identifiers_and_license() {
    let mut document = fixture_document(
        "pubmed-probe",
        "PMID:missing",
        KnowledgeSourceKind::PubMed,
        AuthorityLevel::PeerReviewed,
        SourceDomain::Medical,
    );
    document.metadata.doi = None;
    document.metadata.pmid = None;
    document.provenance.doi = None;
    document.provenance.pmid = None;
    document.license = None;
    let mut profile = SourceAuthorityProfile::for_source_pack_source(
        "pubmed-probe",
        SourceDomain::Medical,
        AuthorityLevel::PeerReviewed,
        None,
    );
    profile.license_policy = None;

    let judgement = BasicEvidenceJudge::at(ts("2026-05-01T00:00:00Z")).judge(&document, &profile);

    assert!(
        judgement
            .warnings
            .contains(&EvidenceWarning::MissingRequiredIdentifier(
                IdentifierKind::Pmid
            ))
    );
    assert!(
        judgement
            .warnings
            .contains(&EvidenceWarning::MissingRequiredIdentifier(
                IdentifierKind::Doi
            ))
    );
    assert!(
        judgement
            .warnings
            .contains(&EvidenceWarning::UnknownLicense)
    );
    assert!(judgement.final_trust_score < 0.55);
}

#[test]
fn basic_judge_handles_legal_customer_and_wikipedia_profiles() {
    let mut legal = fixture_document(
        "legal-probe",
        "legal/privacy/article-12",
        KnowledgeSourceKind::LegalCorpus,
        AuthorityLevel::Official,
        SourceDomain::Legal,
    );
    legal.metadata.doi = None;
    legal.metadata.pmid = None;
    legal.provenance.doi = None;
    legal.provenance.pmid = None;
    legal.metadata.extra.insert(
        SOURCE_METADATA_STATUTE_KEY.to_string(),
        "Example Privacy Code".to_string(),
    );
    legal.metadata.extra.insert(
        SOURCE_METADATA_ARTICLE_KEY.to_string(),
        "Article 12".to_string(),
    );
    legal.provenance.jurisdiction = Some("US-EXAMPLE".to_string());
    legal.metadata.jurisdiction = Some("US-EXAMPLE".to_string());
    legal.metadata.extra.insert(
        "evidence_kind".to_string(),
        EvidenceKind::LawClause.wire_label(),
    );
    legal.provenance.revision_id = None;
    legal.provenance.version = None;
    legal.metadata.extra.remove(SOURCE_METADATA_REVISION_ID_KEY);
    let legal_profile = SourceAuthorityProfile::for_source_pack_source(
        "legal-probe",
        SourceDomain::Legal,
        AuthorityLevel::Official,
        Some("Public domain".to_string()),
    );

    let judgement =
        BasicEvidenceJudge::at(ts("2026-05-01T00:00:00Z")).judge(&legal, &legal_profile);
    assert!(
        judgement
            .warnings
            .contains(&EvidenceWarning::MissingRevisionOrVersion)
    );

    let customer = fixture_document(
        "customer-sop",
        "customer/sop/clinical-trial-intake",
        KnowledgeSourceKind::CustomerPrivateCorpus,
        AuthorityLevel::CustomerInternal,
        SourceDomain::CustomerPrivate,
    );
    let customer_profile = SourceAuthorityProfile::for_source_pack_source(
        "customer-sop",
        SourceDomain::CustomerPrivate,
        AuthorityLevel::CustomerInternal,
        Some("Customer Confidential".to_string()),
    );
    let judgement =
        BasicEvidenceJudge::at(ts("2026-05-01T00:00:00Z")).judge(&customer, &customer_profile);
    assert!(
        !judgement
            .warnings
            .contains(&EvidenceWarning::UnknownAuthority)
    );

    let mut wikipedia = fixture_document(
        "wiki-science",
        "wikipedia/crispr-gene-editing",
        KnowledgeSourceKind::Wikipedia,
        AuthorityLevel::Community,
        SourceDomain::WikipediaSubset,
    );
    wikipedia.source_updated_at = Some(ts("2024-01-01T00:00:00Z"));
    wikipedia.provenance.revision_id = Some("1234567890".to_string());
    let wiki_profile = SourceAuthorityProfile::for_source_pack_source(
        "wiki-science",
        SourceDomain::WikipediaSubset,
        AuthorityLevel::Community,
        Some("CC BY-SA 4.0".to_string()),
    );
    let judgement =
        BasicEvidenceJudge::at(ts("2026-05-01T00:00:00Z")).judge(&wikipedia, &wiki_profile);
    assert!(judgement.warnings.contains(&EvidenceWarning::StaleEvidence));
}

fn fixture_document(
    source_id: &str,
    external_id: &str,
    source_kind: KnowledgeSourceKind,
    authority_level: AuthorityLevel,
    domain: SourceDomain,
) -> ExternalKnowledgeDocument {
    let published_at = ts("2024-11-11T00:00:00Z");
    let mut extra = BTreeMap::new();
    extra.insert(
        "evidence_kind".to_string(),
        match domain {
            SourceDomain::Legal => EvidenceKind::LawClause,
            SourceDomain::Agriculture => EvidenceKind::MarketSignal,
            SourceDomain::CustomerPrivate => EvidenceKind::CustomerInternalNote,
            SourceDomain::WikipediaSubset => EvidenceKind::Definition,
            _ => EvidenceKind::TrialResult,
        }
        .wire_label(),
    );
    extra.insert(
        SOURCE_METADATA_REVISION_ID_KEY.to_string(),
        "fixture-revision".to_string(),
    );
    extra.insert(
        SOURCE_METADATA_LICENSE_KEY.to_string(),
        "Fixture License".to_string(),
    );
    extra.insert(
        SOURCE_METADATA_CLINICAL_TRIAL_ID_KEY.to_string(),
        "NCT00000000".to_string(),
    );

    let metadata = SourceMetadata {
        authors: vec!["Fixture Author".to_string()],
        published_at: Some(published_at),
        updated_at: Some(published_at),
        doi: Some("10.1056/NEJMoa2307563".to_string()),
        pmid: Some("37952131".to_string()),
        mesh_terms: Vec::new(),
        jurisdiction: None,
        tenant_id: None,
        acl_tags: Vec::new(),
        extra,
    };
    let provenance = ProvenanceRecord {
        source_id: source_id.to_string(),
        source_kind,
        authority_level,
        canonical_uri: format!("https://example.test/{external_id}"),
        version: Some("v1".to_string()),
        revision_id: Some("fixture-revision".to_string()),
        doi: metadata.doi.clone(),
        pmid: metadata.pmid.clone(),
        jurisdiction: metadata.jurisdiction.clone(),
        published_at: metadata.published_at,
        fetched_at: ts("2026-04-01T00:00:00Z"),
        content_hash: "fixture-content-hash".to_string(),
        trust_signals: Vec::new(),
    };

    ExternalKnowledgeDocument {
        source_id: source_id.to_string(),
        external_id: external_id.to_string(),
        canonical_uri: provenance.canonical_uri.clone(),
        title: "Fixture evidence".to_string(),
        body: "Recorded evidence probe fixture body.".to_string(),
        sections: vec![KnowledgeSection {
            section_id: "body".to_string(),
            heading_path: vec!["Fixture evidence".to_string()],
            text: "Recorded evidence probe fixture body.".to_string(),
            anchors: Vec::new(),
            citations: Vec::new(),
            tables: Vec::new(),
            figures: Vec::new(),
        }],
        metadata,
        provenance,
        license: Some(LicenseInfo {
            name: "Fixture License".to_string(),
            url: None,
            usage_policy: Some("fixture".to_string()),
        }),
        fetched_at: ts("2026-04-01T00:00:00Z"),
        source_updated_at: Some(published_at),
        content_hash: "fixture-content-hash".to_string(),
    }
}

fn ts(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .unwrap()
        .with_timezone(&Utc)
}
