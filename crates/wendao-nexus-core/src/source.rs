//! Source identity and sync cursor contracts for external knowledge sources.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// External source family.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSourceKind {
    Wikipedia,
    LegalCorpus,
    MedicalJournal,
    PubMed,
    ClinicalTrial,
    GovernmentDatabase,
    CustomerPrivateCorpus,
    WebPage,
    ApiFeed,
    ObjectStorage,
    Other(String),
}

/// Capabilities advertised by a connector before the runtime schedules work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceCapabilities {
    pub discover: bool,
    pub fetch: bool,
    pub delta: bool,
    pub live_query: bool,
    pub local_mirror: bool,
    pub revisions: bool,
    pub structured_metadata: bool,
    pub license_metadata: bool,
    pub access_control: bool,
}

impl SourceCapabilities {
    pub fn none() -> Self {
        Self {
            discover: false,
            fetch: false,
            delta: false,
            live_query: false,
            local_mirror: false,
            revisions: false,
            structured_metadata: false,
            license_metadata: false,
            access_control: false,
        }
    }

    pub fn mirror_fetch() -> Self {
        Self {
            discover: true,
            fetch: true,
            delta: false,
            live_query: false,
            local_mirror: true,
            revisions: false,
            structured_metadata: true,
            license_metadata: true,
            access_control: false,
        }
    }
}

impl Default for SourceCapabilities {
    fn default() -> Self {
        Self::none()
    }
}

/// Opaque connector cursor. The runtime stores it; connectors interpret it.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SourceCursor {
    pub value: String,
}

impl SourceCursor {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// Durable checkpoint for incremental sync.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceCheckpoint {
    pub source_id: String,
    pub cursor: Option<SourceCursor>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_seen_revision: Option<String>,
    pub last_content_hash: Option<String>,
    pub rate_limit_state: Option<String>,
}

impl SourceCheckpoint {
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            cursor: None,
            last_success_at: None,
            last_seen_revision: None,
            last_content_hash: None,
            rate_limit_state: None,
        }
    }
}

/// Stable reference to one item in an external source.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SourceItemRef {
    pub source_id: String,
    pub external_id: String,
    pub canonical_uri: Option<String>,
    pub revision_id: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

impl SourceItemRef {
    pub fn new(source_id: impl Into<String>, external_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            external_id: external_id.into(),
            canonical_uri: None,
            revision_id: None,
            metadata: BTreeMap::new(),
        }
    }
}

/// Discovery result used by scheduled mirror jobs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryBatch {
    pub source_id: String,
    pub items: Vec<SourceItemRef>,
    pub next_cursor: Option<SourceCursor>,
    pub observed_at: DateTime<Utc>,
}

impl DiscoveryBatch {
    pub fn empty(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            items: Vec::new(),
            next_cursor: None,
            observed_at: Utc::now(),
        }
    }
}

/// Incremental change emitted by a source connector.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SourceChange {
    Upsert(SourceItemRef),
    Delete(SourceItemRef),
}

/// Delta result used by incremental sync jobs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeltaBatch {
    pub source_id: String,
    pub changes: Vec<SourceChange>,
    pub next_checkpoint: SourceCheckpoint,
    pub observed_at: DateTime<Utc>,
}
