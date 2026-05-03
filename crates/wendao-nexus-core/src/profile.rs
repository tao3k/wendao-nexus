//! Source authority profiles and deterministic evidence judgement contracts.

use serde::{Deserialize, Serialize};

use crate::authority::AuthorityLevel;
use crate::query::EvidenceKind;
use crate::source::SourceDomain;

/// Expected identifier classes used by source authority profiles.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum IdentifierKind {
    Doi,
    Pmid,
    Jurisdiction,
    RevisionId,
    ClinicalTrialId,
    Statute,
    Article,
    CustomerDocId,
    Url,
    Other(String),
}

impl IdentifierKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Doi => "doi",
            Self::Pmid => "pmid",
            Self::Jurisdiction => "jurisdiction",
            Self::RevisionId => "revision_id",
            Self::ClinicalTrialId => "clinical_trial_id",
            Self::Statute => "statute",
            Self::Article => "article",
            Self::CustomerDocId => "customer_doc_id",
            Self::Url => "url",
            Self::Other(value) => value.as_str(),
        }
    }

    pub fn wire_label(&self) -> String {
        match self {
            Self::Other(value) if value.starts_with("other:") => value.clone(),
            Self::Other(value) => format!("other:{value}"),
            _ => self.as_str().to_string(),
        }
    }

    pub fn from_label(label: impl AsRef<str>) -> Self {
        let label = label.as_ref().trim();
        match label {
            "doi" => Self::Doi,
            "pmid" => Self::Pmid,
            "jurisdiction" => Self::Jurisdiction,
            "revision_id" => Self::RevisionId,
            "clinical_trial_id" => Self::ClinicalTrialId,
            "statute" => Self::Statute,
            "article" => Self::Article,
            "customer_doc_id" => Self::CustomerDocId,
            "url" => Self::Url,
            other if other.starts_with("other:") => {
                Self::Other(other.strip_prefix("other:").unwrap_or_default().to_string())
            }
            other => Self::Other(other.to_string()),
        }
    }
}

impl Serialize for IdentifierKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.wire_label())
    }
}

impl<'de> Deserialize<'de> for IdentifierKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(label))
    }
}

/// Deterministic warning emitted by the Rust authority judge.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EvidenceWarning {
    MissingCanonicalUri,
    MissingPublishedAt,
    MissingRequiredIdentifier(IdentifierKind),
    StaleEvidence,
    UnknownAuthority,
    UnknownLicense,
    MissingRevisionOrVersion,
    SourceKindMismatch,
    Other(String),
}

impl EvidenceWarning {
    pub fn wire_label(&self) -> String {
        match self {
            Self::MissingCanonicalUri => "missing_canonical_uri".to_string(),
            Self::MissingPublishedAt => "missing_published_at".to_string(),
            Self::MissingRequiredIdentifier(identifier) => {
                format!("missing_required_identifier:{}", identifier.wire_label())
            }
            Self::StaleEvidence => "stale_evidence".to_string(),
            Self::UnknownAuthority => "unknown_authority".to_string(),
            Self::UnknownLicense => "unknown_license".to_string(),
            Self::MissingRevisionOrVersion => "missing_revision_or_version".to_string(),
            Self::SourceKindMismatch => "source_kind_mismatch".to_string(),
            Self::Other(value) if value.starts_with("other:") => value.clone(),
            Self::Other(value) => format!("other:{value}"),
        }
    }

    pub fn from_label(label: impl AsRef<str>) -> Self {
        let label = label.as_ref().trim();
        match label {
            "missing_canonical_uri" => Self::MissingCanonicalUri,
            "missing_published_at" => Self::MissingPublishedAt,
            "stale_evidence" => Self::StaleEvidence,
            "unknown_authority" => Self::UnknownAuthority,
            "unknown_license" => Self::UnknownLicense,
            "missing_revision_or_version" => Self::MissingRevisionOrVersion,
            "source_kind_mismatch" => Self::SourceKindMismatch,
            other if other.starts_with("missing_required_identifier:") => {
                let identifier = other
                    .strip_prefix("missing_required_identifier:")
                    .unwrap_or_default();
                Self::MissingRequiredIdentifier(IdentifierKind::from_label(identifier))
            }
            other if other.starts_with("other:") => {
                Self::Other(other.strip_prefix("other:").unwrap_or_default().to_string())
            }
            other => Self::Other(other.to_string()),
        }
    }
}

impl Serialize for EvidenceWarning {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.wire_label())
    }
}

impl<'de> Deserialize<'de> for EvidenceWarning {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(label))
    }
}

/// Configurable source profile for basic authority judgement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceAuthorityProfile {
    pub source_id: String,
    pub domain: SourceDomain,
    pub authority_level: AuthorityLevel,
    #[serde(default)]
    pub expected_identifiers: Vec<IdentifierKind>,
    #[serde(default)]
    pub expected_evidence_kinds: Vec<EvidenceKind>,
    #[serde(default)]
    pub license_policy: Option<String>,
    #[serde(default)]
    pub max_staleness_days: Option<u32>,
    #[serde(default)]
    pub requires_published_at: bool,
    #[serde(default = "default_requires_canonical_uri")]
    pub requires_canonical_uri: bool,
    #[serde(default)]
    pub requires_revision_or_version: bool,
    #[serde(default)]
    pub requires_license: bool,
}

impl SourceAuthorityProfile {
    pub fn for_source_pack_source(
        source_id: impl Into<String>,
        domain: SourceDomain,
        authority_level: AuthorityLevel,
        license_policy: Option<String>,
    ) -> Self {
        let source_id = source_id.into();
        let requires_license = license_policy.is_some();
        let mut profile = Self {
            source_id,
            domain,
            authority_level,
            expected_identifiers: Vec::new(),
            expected_evidence_kinds: Vec::new(),
            license_policy,
            max_staleness_days: None,
            requires_published_at: false,
            requires_canonical_uri: true,
            requires_revision_or_version: false,
            requires_license,
        };
        profile.apply_domain_defaults();
        profile
    }

    fn apply_domain_defaults(&mut self) {
        match self.domain {
            SourceDomain::Medical => {
                self.expected_identifiers = vec![IdentifierKind::Pmid, IdentifierKind::Doi];
                self.expected_evidence_kinds = vec![
                    EvidenceKind::ReviewArticle,
                    EvidenceKind::TrialResult,
                    EvidenceKind::Guideline,
                ];
                self.max_staleness_days = Some(1825);
                self.requires_published_at = true;
            }
            SourceDomain::Legal => {
                self.expected_identifiers = vec![
                    IdentifierKind::Jurisdiction,
                    IdentifierKind::Statute,
                    IdentifierKind::Article,
                ];
                self.expected_evidence_kinds = vec![EvidenceKind::LawClause];
                self.requires_revision_or_version = true;
            }
            SourceDomain::Agriculture => {
                self.expected_evidence_kinds = vec![EvidenceKind::MarketSignal];
                self.max_staleness_days = Some(90);
            }
            SourceDomain::WikipediaSubset => {
                self.expected_identifiers = vec![IdentifierKind::RevisionId, IdentifierKind::Url];
                self.expected_evidence_kinds =
                    vec![EvidenceKind::Definition, EvidenceKind::Document];
                self.max_staleness_days = Some(365);
                self.requires_revision_or_version = true;
            }
            SourceDomain::CustomerPrivate => {
                self.expected_identifiers = vec![IdentifierKind::CustomerDocId];
                self.expected_evidence_kinds = vec![EvidenceKind::CustomerInternalNote];
                self.requires_revision_or_version = true;
            }
            SourceDomain::Finance | SourceDomain::Generic | SourceDomain::Other(_) => {}
        }
    }
}

/// Deterministic judgement emitted by the Rust basic authority judge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthorityJudgement {
    pub authority_score: f64,
    pub freshness_score: f64,
    pub provenance_score: f64,
    pub identifier_score: f64,
    pub license_score: f64,
    pub source_kind_fit_score: f64,
    pub final_trust_score: f64,
    pub warnings: Vec<EvidenceWarning>,
}

fn default_requires_canonical_uri() -> bool {
    true
}
