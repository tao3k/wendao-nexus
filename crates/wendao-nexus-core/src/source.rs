//! Source identity and sync cursor contracts for external knowledge sources.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::authority::AuthorityLevel;

/// Source registry metadata key used to preserve the source-pack domain.
pub const SOURCE_PACK_DOMAIN_METADATA_KEY: &str = "source_pack_domain";
/// Source registry metadata key used to preserve the source-pack id.
pub const SOURCE_PACK_ID_METADATA_KEY: &str = "source_pack_id";
/// Source registry metadata key used to preserve the source-pack version.
pub const SOURCE_PACK_VERSION_METADATA_KEY: &str = "source_pack_version";
/// Source registry metadata key used to preserve the source-pack display name.
pub const SOURCE_PACK_DISPLAY_NAME_METADATA_KEY: &str = "source_pack_display_name";
/// Source registry metadata key used to preserve deterministic fixture paths.
pub const SOURCE_PACK_FIXTURE_PATH_METADATA_KEY: &str = "fixture_path";

/// Vertical source-pack domain.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum SourceDomain {
    #[default]
    Generic,
    Medical,
    Legal,
    Agriculture,
    Finance,
    WikipediaSubset,
    CustomerPrivate,
    Other(String),
}

impl SourceDomain {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Generic => "generic",
            Self::Medical => "medical",
            Self::Legal => "legal",
            Self::Agriculture => "agriculture",
            Self::Finance => "finance",
            Self::WikipediaSubset => "wikipedia_subset",
            Self::CustomerPrivate => "customer_private",
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
            "" | "generic" => Self::Generic,
            "medical" => Self::Medical,
            "legal" => Self::Legal,
            "agriculture" => Self::Agriculture,
            "finance" => Self::Finance,
            "wikipedia_subset" => Self::WikipediaSubset,
            "customer_private" => Self::CustomerPrivate,
            other if other.starts_with("other:") => {
                Self::Other(other.strip_prefix("other:").unwrap_or_default().to_string())
            }
            other => Self::Other(other.to_string()),
        }
    }
}

impl Serialize for SourceDomain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.wire_label())
    }
}

impl<'de> Deserialize<'de> for SourceDomain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(label))
    }
}

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

    pub fn local_corpus_fixture() -> Self {
        Self {
            discover: true,
            fetch: true,
            delta: true,
            live_query: false,
            local_mirror: true,
            revisions: true,
            structured_metadata: true,
            license_metadata: true,
            access_control: true,
        }
    }
}

impl Default for SourceCapabilities {
    fn default() -> Self {
        Self::none()
    }
}

/// Source catalog record shared by connectors and Wendao-side adapters.
///
/// This is source identity and policy metadata only. It must not contain secret
/// values; connector credentials belong to the embedding Wendao-side runtime or
/// deployment environment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NexusSourceRecord {
    pub source_id: String,
    pub source_kind: KnowledgeSourceKind,
    pub display_name: String,
    pub base_uri: Option<String>,
    pub auth_mode: Option<String>,
    pub license_policy: Option<String>,
    pub authority_level: AuthorityLevel,
    pub sync_policy: Option<String>,
    pub capabilities: SourceCapabilities,
    pub enabled: bool,
    pub metadata: BTreeMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NexusSourceRecord {
    pub fn new(source_id: impl Into<String>, source_kind: KnowledgeSourceKind) -> Self {
        let source_id = source_id.into();
        let now = Utc::now();
        Self {
            display_name: source_id.clone(),
            source_id,
            source_kind,
            base_uri: None,
            auth_mode: None,
            license_policy: None,
            authority_level: AuthorityLevel::Unknown,
            sync_policy: None,
            capabilities: SourceCapabilities::none(),
            enabled: true,
            metadata: BTreeMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn source_pack_domain(&self) -> SourceDomain {
        self.metadata
            .get(SOURCE_PACK_DOMAIN_METADATA_KEY)
            .map(SourceDomain::from_label)
            .unwrap_or_default()
    }

    pub fn source_pack_id(&self) -> Option<&str> {
        self.metadata_value(SOURCE_PACK_ID_METADATA_KEY)
    }

    pub fn source_pack_version(&self) -> Option<&str> {
        self.metadata_value(SOURCE_PACK_VERSION_METADATA_KEY)
    }

    pub fn source_pack_display_name(&self) -> Option<&str> {
        self.metadata_value(SOURCE_PACK_DISPLAY_NAME_METADATA_KEY)
    }

    pub fn source_pack_fixture_path(&self) -> Option<&str> {
        self.metadata_value(SOURCE_PACK_FIXTURE_PATH_METADATA_KEY)
    }

    fn metadata_value(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
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

/// Checkpoint payload for incremental sync.
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
