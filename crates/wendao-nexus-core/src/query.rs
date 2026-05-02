//! Agent-facing query contracts for external knowledge evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::authority::{AuthorityLevel, TrustPolicy};
use crate::trust::ProvenanceBundle;

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
    pub provenance: ProvenanceBundle,
}
