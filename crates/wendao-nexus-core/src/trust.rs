//! Provenance bundles and evidence boundary contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::authority::{AuthorityLevel, TrustSignal};
use crate::source::KnowledgeSourceKind;

/// Provenance attached to a normalized document, section, or evidence item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub source_id: String,
    pub source_kind: KnowledgeSourceKind,
    pub authority_level: AuthorityLevel,
    pub canonical_uri: String,
    pub version: Option<String>,
    pub revision_id: Option<String>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub jurisdiction: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
    pub content_hash: String,
    pub trust_signals: Vec<TrustSignal>,
}

/// Compact bundle returned to callers that need auditable evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceBundle {
    pub primary: ProvenanceRecord,
    pub corroborating: Vec<ProvenanceRecord>,
    pub conflicting: Vec<ProvenanceRecord>,
}

/// Boundary statement for answer generation and adjudication layers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceBoundary {
    pub records: Vec<ProvenanceRecord>,
    pub minimum_authority: AuthorityLevel,
    pub insufficient_authority: bool,
    pub stale_evidence: bool,
    pub conflict_detected: bool,
}
