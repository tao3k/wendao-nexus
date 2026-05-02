//! Canonical Flight routes for external knowledge operations.

use std::fmt;

use arrow_flight::Ticket;
use serde::{Deserialize, Serialize};

/// Search external knowledge evidence.
pub const EXTERNAL_KNOWLEDGE_SEARCH_ROUTE: &str = "/knowledge/external/search";

/// Open one external knowledge item.
pub const EXTERNAL_KNOWLEDGE_OPEN_ROUTE: &str = "/knowledge/external/open";

/// Sync or refresh external knowledge sources.
pub const EXTERNAL_KNOWLEDGE_SYNC_ROUTE: &str = "/knowledge/external/sync";

/// Read external source or sync job status.
pub const EXTERNAL_KNOWLEDGE_STATUS_ROUTE: &str = "/knowledge/external/status";

/// Compare a claim against governed external evidence.
pub const EXTERNAL_KNOWLEDGE_COMPARE_ROUTE: &str = "/knowledge/external/compare";

/// Supported `Wendao Nexus` Flight routes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum NexusFlightRoute {
    Search,
    Open,
    Sync,
    Status,
    Compare,
}

impl NexusFlightRoute {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Search => EXTERNAL_KNOWLEDGE_SEARCH_ROUTE,
            Self::Open => EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
            Self::Sync => EXTERNAL_KNOWLEDGE_SYNC_ROUTE,
            Self::Status => EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
            Self::Compare => EXTERNAL_KNOWLEDGE_COMPARE_ROUTE,
        }
    }

    pub fn all() -> [Self; 5] {
        [
            Self::Search,
            Self::Open,
            Self::Sync,
            Self::Status,
            Self::Compare,
        ]
    }

    pub fn ticket(self) -> Ticket {
        Ticket {
            ticket: self.as_str().as_bytes().to_vec().into(),
        }
    }
}

impl fmt::Display for NexusFlightRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for NexusFlightRoute {
    type Error = String;

    fn try_from(route: &str) -> Result<Self, Self::Error> {
        match route {
            EXTERNAL_KNOWLEDGE_SEARCH_ROUTE => Ok(Self::Search),
            EXTERNAL_KNOWLEDGE_OPEN_ROUTE => Ok(Self::Open),
            EXTERNAL_KNOWLEDGE_SYNC_ROUTE => Ok(Self::Sync),
            EXTERNAL_KNOWLEDGE_STATUS_ROUTE => Ok(Self::Status),
            EXTERNAL_KNOWLEDGE_COMPARE_ROUTE => Ok(Self::Compare),
            _ => Err(format!("unsupported Wendao Nexus Flight route `{route}`")),
        }
    }
}
