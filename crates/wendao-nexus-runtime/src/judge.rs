//! Deterministic Rust authority judgement for fixture and probe evidence.

use chrono::{DateTime, Utc};
use wendao_nexus_core::{
    AuthorityJudgement, AuthorityLevel, EvidenceWarning, ExternalKnowledgeDocument, IdentifierKind,
    SOURCE_METADATA_ARTICLE_KEY, SOURCE_METADATA_CLINICAL_TRIAL_ID_KEY,
    SOURCE_METADATA_STATUTE_KEY, SourceAuthorityProfile,
};

/// Stable evidence judgement interface.
pub trait EvidenceJudge {
    fn judge(
        &self,
        document: &ExternalKnowledgeDocument,
        profile: &SourceAuthorityProfile,
    ) -> AuthorityJudgement;
}

/// Deterministic basic authority judge.
#[derive(Clone, Debug)]
pub struct BasicEvidenceJudge {
    now: DateTime<Utc>,
}

impl BasicEvidenceJudge {
    pub fn at(now: DateTime<Utc>) -> Self {
        Self { now }
    }
}

impl Default for BasicEvidenceJudge {
    fn default() -> Self {
        Self { now: Utc::now() }
    }
}

impl EvidenceJudge for BasicEvidenceJudge {
    fn judge(
        &self,
        document: &ExternalKnowledgeDocument,
        profile: &SourceAuthorityProfile,
    ) -> AuthorityJudgement {
        let mut warnings = Vec::new();
        let authority_score = authority_score(document.provenance.authority_level);
        let provenance_score = provenance_score(document);
        let (identifier_score, missing_identifiers) = identifier_score(document, profile);
        let (freshness_score, stale) = freshness_score(document, profile, self.now);
        let license_score = license_score(document, profile);
        let source_kind_fit_score = evidence_kind_fit_score(document, profile);

        if profile.requires_canonical_uri && document.canonical_uri.trim().is_empty() {
            warnings.push(EvidenceWarning::MissingCanonicalUri);
        }
        if profile.requires_published_at && document.metadata.published_at.is_none() {
            warnings.push(EvidenceWarning::MissingPublishedAt);
        }
        for identifier in missing_identifiers {
            warnings.push(EvidenceWarning::MissingRequiredIdentifier(identifier));
        }
        if stale {
            warnings.push(EvidenceWarning::StaleEvidence);
        }
        if document.provenance.authority_level == AuthorityLevel::Unknown {
            warnings.push(EvidenceWarning::UnknownAuthority);
        }
        if profile.requires_license && license_score <= f64::EPSILON {
            warnings.push(EvidenceWarning::UnknownLicense);
        }
        if profile.requires_revision_or_version
            && document.provenance.revision_id.is_none()
            && document.provenance.version.is_none()
            && document.metadata.revision_id_metadata().is_none()
            && document.metadata.version_metadata().is_none()
        {
            warnings.push(EvidenceWarning::MissingRevisionOrVersion);
        }
        if source_kind_fit_score <= f64::EPSILON {
            warnings.push(EvidenceWarning::SourceKindMismatch);
        }

        let weighted = 0.30 * authority_score
            + 0.20 * provenance_score
            + 0.20 * identifier_score
            + 0.15 * freshness_score
            + 0.10 * license_score
            + 0.05 * source_kind_fit_score;

        let penalty = warning_penalty(&warnings);
        let final_trust_score = clamp_score(weighted - penalty);

        AuthorityJudgement {
            authority_score,
            freshness_score,
            provenance_score,
            identifier_score,
            license_score,
            source_kind_fit_score,
            final_trust_score,
            warnings,
        }
    }
}

fn authority_score(authority: AuthorityLevel) -> f64 {
    match authority {
        AuthorityLevel::Official => 1.0,
        AuthorityLevel::PeerReviewed => 0.9,
        AuthorityLevel::CustomerInternal => 0.8,
        AuthorityLevel::Curated => 0.65,
        AuthorityLevel::Community => 0.4,
        AuthorityLevel::Unknown => 0.0,
    }
}

fn provenance_score(document: &ExternalKnowledgeDocument) -> f64 {
    let checks = [
        !document.source_id.trim().is_empty(),
        !document.external_id.trim().is_empty(),
        !document.provenance.canonical_uri.trim().is_empty(),
        !document.provenance.content_hash.trim().is_empty(),
        document.provenance.published_at.is_some()
            || document.provenance.revision_id.is_some()
            || document.provenance.version.is_some()
            || document.provenance.doi.is_some()
            || document.provenance.pmid.is_some()
            || document.provenance.jurisdiction.is_some(),
    ];
    checks.iter().filter(|present| **present).count() as f64 / checks.len() as f64
}

fn identifier_score(
    document: &ExternalKnowledgeDocument,
    profile: &SourceAuthorityProfile,
) -> (f64, Vec<IdentifierKind>) {
    if profile.expected_identifiers.is_empty() {
        return (1.0, Vec::new());
    }

    let mut missing = Vec::new();
    for identifier in &profile.expected_identifiers {
        if !identifier_present(document, identifier) {
            missing.push(identifier.clone());
        }
    }

    let present = profile.expected_identifiers.len() - missing.len();
    (
        present as f64 / profile.expected_identifiers.len() as f64,
        missing,
    )
}

fn identifier_present(document: &ExternalKnowledgeDocument, identifier: &IdentifierKind) -> bool {
    match identifier {
        IdentifierKind::Doi => document.metadata.doi.is_some() || document.provenance.doi.is_some(),
        IdentifierKind::Pmid => {
            document.metadata.pmid.is_some() || document.provenance.pmid.is_some()
        }
        IdentifierKind::Jurisdiction => {
            document.metadata.jurisdiction.is_some() || document.provenance.jurisdiction.is_some()
        }
        IdentifierKind::RevisionId => {
            document.provenance.revision_id.is_some()
                || document.metadata.revision_id_metadata().is_some()
        }
        IdentifierKind::ClinicalTrialId => document
            .metadata
            .extra
            .get(SOURCE_METADATA_CLINICAL_TRIAL_ID_KEY)
            .is_some_and(|value| !value.trim().is_empty()),
        IdentifierKind::Statute => document
            .metadata
            .extra
            .get(SOURCE_METADATA_STATUTE_KEY)
            .is_some_and(|value| !value.trim().is_empty()),
        IdentifierKind::Article => document
            .metadata
            .extra
            .get(SOURCE_METADATA_ARTICLE_KEY)
            .is_some_and(|value| !value.trim().is_empty()),
        IdentifierKind::CustomerDocId => !document.external_id.trim().is_empty(),
        IdentifierKind::Url => !document.canonical_uri.trim().is_empty(),
        IdentifierKind::Other(key) => document
            .metadata
            .extra
            .get(key)
            .is_some_and(|value| !value.trim().is_empty()),
    }
}

fn freshness_score(
    document: &ExternalKnowledgeDocument,
    profile: &SourceAuthorityProfile,
    now: DateTime<Utc>,
) -> (f64, bool) {
    let Some(max_days) = profile.max_staleness_days else {
        return (1.0, false);
    };
    let reference = document
        .source_updated_at
        .or(document.metadata.updated_at)
        .or(document.metadata.published_at)
        .unwrap_or(document.fetched_at);
    let age_days = now.signed_duration_since(reference).num_days().max(0) as u32;
    if age_days <= max_days {
        (1.0, false)
    } else {
        (0.0, true)
    }
}

fn license_score(document: &ExternalKnowledgeDocument, profile: &SourceAuthorityProfile) -> f64 {
    if !profile.requires_license || document.license.is_some() || profile.license_policy.is_some() {
        1.0
    } else {
        0.0
    }
}

fn evidence_kind_fit_score(
    document: &ExternalKnowledgeDocument,
    profile: &SourceAuthorityProfile,
) -> f64 {
    if profile.expected_evidence_kinds.is_empty()
        || profile
            .expected_evidence_kinds
            .contains(&document.metadata.evidence_kind())
    {
        1.0
    } else {
        0.0
    }
}

fn warning_penalty(warnings: &[EvidenceWarning]) -> f64 {
    warnings
        .iter()
        .map(|warning| match warning {
            EvidenceWarning::MissingCanonicalUri => 0.10,
            EvidenceWarning::MissingPublishedAt => 0.10,
            EvidenceWarning::MissingRequiredIdentifier(_) => 0.20,
            EvidenceWarning::StaleEvidence => 0.20,
            EvidenceWarning::UnknownAuthority => 0.20,
            EvidenceWarning::UnknownLicense => 0.05,
            EvidenceWarning::MissingRevisionOrVersion => 0.10,
            EvidenceWarning::SourceKindMismatch => 0.05,
            EvidenceWarning::Other(_) => 0.0,
        })
        .sum()
}

fn clamp_score(score: f64) -> f64 {
    score.clamp(0.0, 1.0)
}
