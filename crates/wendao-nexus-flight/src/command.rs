//! Typed command encoding for `FlightDescriptor::cmd` payloads.

use arrow_flight::flight_descriptor::DescriptorType;
use arrow_flight::FlightDescriptor;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wendao_nexus_core::{
    ExternalKnowledgeCompareRequest, ExternalKnowledgeOpenRequest, ExternalKnowledgeRefreshRequest,
    ExternalKnowledgeSearchRequest,
};

use crate::routes::NexusFlightRoute;

/// Request payload for `/knowledge/external/status`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NexusFlightStatusRequest {
    pub sources: Vec<String>,
}

impl NexusFlightStatusRequest {
    pub fn all_sources() -> Self {
        Self {
            sources: Vec::new(),
        }
    }
}

/// Request payload for `/knowledge/external/sync`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NexusFlightSyncRequest {
    pub source_id: String,
    pub external_id: Option<String>,
    pub force: bool,
}

impl From<ExternalKnowledgeRefreshRequest> for NexusFlightSyncRequest {
    fn from(request: ExternalKnowledgeRefreshRequest) -> Self {
        Self {
            source_id: request.source_id,
            external_id: Some(request.external_id),
            force: request.force,
        }
    }
}

/// Typed command accepted by the Wendao-side Nexus Flight route adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NexusFlightCommand {
    Search(ExternalKnowledgeSearchRequest),
    Open(ExternalKnowledgeOpenRequest),
    Sync(NexusFlightSyncRequest),
    Status(NexusFlightStatusRequest),
    Compare(ExternalKnowledgeCompareRequest),
}

impl NexusFlightCommand {
    pub fn route(&self) -> NexusFlightRoute {
        match self {
            Self::Search(_) => NexusFlightRoute::Search,
            Self::Open(_) => NexusFlightRoute::Open,
            Self::Sync(_) => NexusFlightRoute::Sync,
            Self::Status(_) => NexusFlightRoute::Status,
            Self::Compare(_) => NexusFlightRoute::Compare,
        }
    }

    pub fn encode_json(&self) -> Result<Vec<u8>, NexusFlightCommandError> {
        let payload = match self {
            Self::Search(request) => serde_json::to_value(request)?,
            Self::Open(request) => serde_json::to_value(request)?,
            Self::Sync(request) => serde_json::to_value(request)?,
            Self::Status(request) => serde_json::to_value(request)?,
            Self::Compare(request) => serde_json::to_value(request)?,
        };
        let envelope = NexusFlightCommandEnvelope {
            route: self.route().as_str().to_string(),
            payload,
        };
        Ok(serde_json::to_vec(&envelope)?)
    }

    pub fn decode_json(bytes: &[u8]) -> Result<Self, NexusFlightCommandError> {
        let envelope: NexusFlightCommandEnvelope = serde_json::from_slice(bytes)?;
        let route = NexusFlightRoute::try_from(envelope.route.as_str())
            .map_err(NexusFlightCommandError::UnsupportedRoute)?;

        match route {
            NexusFlightRoute::Search => Ok(Self::Search(serde_json::from_value(envelope.payload)?)),
            NexusFlightRoute::Open => Ok(Self::Open(serde_json::from_value(envelope.payload)?)),
            NexusFlightRoute::Sync => Ok(Self::Sync(serde_json::from_value(envelope.payload)?)),
            NexusFlightRoute::Status => Ok(Self::Status(serde_json::from_value(envelope.payload)?)),
            NexusFlightRoute::Compare => {
                Ok(Self::Compare(serde_json::from_value(envelope.payload)?))
            }
        }
    }

    pub fn to_descriptor(&self) -> Result<FlightDescriptor, NexusFlightCommandError> {
        Ok(FlightDescriptor::new_cmd(self.encode_json()?))
    }

    pub fn from_descriptor(descriptor: &FlightDescriptor) -> Result<Self, NexusFlightCommandError> {
        if descriptor.r#type != DescriptorType::Cmd as i32 {
            return Err(NexusFlightCommandError::UnsupportedDescriptorType(
                descriptor.r#type,
            ));
        }

        Self::decode_json(&descriptor.cmd)
    }
}

/// Errors produced while encoding or decoding Nexus Flight commands.
#[derive(Debug, Error)]
pub enum NexusFlightCommandError {
    #[error("unsupported Nexus Flight route: {0}")]
    UnsupportedRoute(String),

    #[error("unsupported Flight descriptor type: {0}")]
    UnsupportedDescriptorType(i32),

    #[error("failed to encode or decode Nexus Flight command JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct NexusFlightCommandEnvelope {
    route: String,
    payload: serde_json::Value,
}
