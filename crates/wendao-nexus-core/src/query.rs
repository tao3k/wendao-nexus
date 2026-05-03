//! Agent-facing query contracts for external knowledge evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::authority::{AuthorityLevel, TrustPolicy};
use crate::trust::ProvenanceBundle;

/// Metadata key used by source packs to declare the agent-facing evidence kind.
pub const EVIDENCE_KIND_METADATA_KEY: &str = "evidence_kind";

/// Search request exposed to agents and LLM-facing gateways.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalKnowledgeSearchRequest {
    pub query: String,
    pub sources: Vec<String>,
    pub trust_policy: TrustPolicy,
    pub freshness_days: Option<u32>,
    pub limit: usize,
}

impl ExternalKnowledgeSearchRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            sources: Vec::new(),
            trust_policy: TrustPolicy::authority_at_least(AuthorityLevel::Unknown),
            freshness_days: None,
            limit: 20,
        }
    }
}

/// Open request for one external item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalKnowledgeOpenRequest {
    pub source_id: String,
    pub external_id: String,
    pub include_sections: bool,
    pub include_provenance: bool,
}

/// Claim comparison request for authority/conflict checks.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalKnowledgeCompareRequest {
    pub claim: String,
    pub sources: Vec<String>,
    pub mode: EvidenceConflictMode,
    pub trust_policy: TrustPolicy,
}

/// Refresh request for one source item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalKnowledgeRefreshRequest {
    pub source_id: String,
    pub external_id: String,
    pub force: bool,
}

/// Comparison strategy requested by an agent.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvidenceConflictMode {
    EvidenceConflictCheck,
    CorroborationCheck,
    FreshnessCheck,
}

/// Kind of evidence returned to an agent.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum EvidenceKind {
    #[default]
    Document,
    Definition,
    Claim,
    Statistic,
    Guideline,
    LawClause,
    TrialResult,
    ReviewArticle,
    CustomerInternalNote,
    NewsSignal,
    MarketSignal,
    Other(String),
}

impl EvidenceKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Document => "document",
            Self::Definition => "definition",
            Self::Claim => "claim",
            Self::Statistic => "statistic",
            Self::Guideline => "guideline",
            Self::LawClause => "law_clause",
            Self::TrialResult => "trial_result",
            Self::ReviewArticle => "review_article",
            Self::CustomerInternalNote => "customer_internal_note",
            Self::NewsSignal => "news_signal",
            Self::MarketSignal => "market_signal",
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
            "" | "document" => Self::Document,
            "definition" => Self::Definition,
            "claim" => Self::Claim,
            "statistic" => Self::Statistic,
            "guideline" => Self::Guideline,
            "law_clause" => Self::LawClause,
            "trial_result" => Self::TrialResult,
            "review_article" => Self::ReviewArticle,
            "customer_internal_note" => Self::CustomerInternalNote,
            "news_signal" => Self::NewsSignal,
            "market_signal" => Self::MarketSignal,
            other if other.starts_with("other:") => {
                Self::Other(other.strip_prefix("other:").unwrap_or_default().to_string())
            }
            other => Self::Other(other.to_string()),
        }
    }
}

impl Serialize for EvidenceKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.wire_label())
    }
}

impl<'de> Deserialize<'de> for EvidenceKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(label))
    }
}

/// Search response with evidence records rather than prose-only answers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalKnowledgeSearchResponse {
    pub query: String,
    pub records: Vec<EvidenceRecord>,
    pub generated_at: DateTime<Utc>,
}

/// Agent-consumable evidence record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub source_id: String,
    pub external_id: String,
    pub title: String,
    pub snippet: String,
    pub score: Option<String>,
    #[serde(default)]
    pub evidence_kind: EvidenceKind,
    pub provenance: ProvenanceBundle,
}
