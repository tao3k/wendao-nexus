//! Authority levels and trust policies used to filter `Wendao Nexus` evidence.

use serde::{Deserialize, Serialize};

/// Coarse authority class used for filtering and ranking external evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum AuthorityLevel {
    Unknown,
    Community,
    Curated,
    CustomerInternal,
    PeerReviewed,
    Official,
}

/// A source-specific trust signal captured before final adjudication.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrustSignal {
    pub name: String,
    pub value: String,
}

impl TrustSignal {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Trust policy requested by a caller or configured for a source pack.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrustPolicy {
    pub minimum_authority: AuthorityLevel,
    pub allow_community_sources: bool,
    pub require_provenance: bool,
}

impl TrustPolicy {
    pub fn authority_at_least(minimum_authority: AuthorityLevel) -> Self {
        Self {
            minimum_authority,
            allow_community_sources: false,
            require_provenance: true,
        }
    }
}

impl Default for TrustPolicy {
    fn default() -> Self {
        Self {
            minimum_authority: AuthorityLevel::Unknown,
            allow_community_sources: true,
            require_provenance: true,
        }
    }
}
