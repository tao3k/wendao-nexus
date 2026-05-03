//! External database and API-feed connector boundary.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;
use wendao_nexus_core::{
    AuthorityLevel, DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, KnowledgeSourceKind,
    NexusError, NexusResult, NexusSourceRecord, RawSourceDocument, SourceCapabilities,
    SourceCheckpoint, SourceCursor, SourceItemRef,
};

/// High-level access pattern for an external database source.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExternalDatabaseAccessMode {
    MirrorOnly,
    FederatedLive,
    MirrorAndFederated,
}

impl ExternalDatabaseAccessMode {
    pub fn as_registry_label(self) -> &'static str {
        match self {
            Self::MirrorOnly => "mirror_only",
            Self::FederatedLive => "federated_live",
            Self::MirrorAndFederated => "mirror_and_federated",
        }
    }
}

/// Auth shape for an external database connector.
///
/// This intentionally stores auth mode, not secret material. Secret lookup and
/// runtime injection belong to the embedding Wendao-side server or deployment
/// environment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExternalDatabaseAuthMode {
    None,
    ApiKeyHeader { header_name: String },
    BearerToken,
    OAuth2ClientCredentials { token_url: String },
    Custom(String),
}

impl ExternalDatabaseAuthMode {
    pub fn as_registry_label(&self) -> String {
        match self {
            Self::None => "none".to_string(),
            Self::ApiKeyHeader { header_name } => format!("api_key_header:{header_name}"),
            Self::BearerToken => "bearer_token".to_string(),
            Self::OAuth2ClientCredentials { token_url } => {
                format!("oauth2_client_credentials:{token_url}")
            }
            Self::Custom(label) => format!("custom:{label}"),
        }
    }
}

/// Configuration for one external database or API-backed source.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalDatabaseConfig {
    pub source_id: String,
    pub source_kind: KnowledgeSourceKind,
    pub display_name: String,
    pub endpoint_uri: String,
    pub auth_mode: ExternalDatabaseAuthMode,
    pub access_mode: ExternalDatabaseAccessMode,
    pub supports_delta: bool,
    pub supports_revisions: bool,
    pub structured_metadata: bool,
    pub license_metadata: bool,
    pub access_control: bool,
    pub default_media_type: String,
}

impl ExternalDatabaseConfig {
    pub fn new(
        source_id: impl Into<String>,
        source_kind: KnowledgeSourceKind,
        endpoint_uri: impl Into<String>,
    ) -> Self {
        let source_id = source_id.into();
        Self {
            display_name: source_id.clone(),
            source_id,
            source_kind,
            endpoint_uri: endpoint_uri.into(),
            auth_mode: ExternalDatabaseAuthMode::None,
            access_mode: ExternalDatabaseAccessMode::MirrorOnly,
            supports_delta: true,
            supports_revisions: false,
            structured_metadata: true,
            license_metadata: false,
            access_control: false,
            default_media_type: "application/json".to_string(),
        }
    }

    pub fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            discover: false,
            fetch: false,
            delta: false,
            live_query: false,
            local_mirror: false,
            revisions: self.supports_revisions,
            structured_metadata: self.structured_metadata,
            license_metadata: self.license_metadata,
            access_control: self.access_control,
        }
    }

    pub fn source_record(&self, authority_level: AuthorityLevel) -> NexusSourceRecord {
        let mut source = NexusSourceRecord::new(self.source_id.clone(), self.source_kind.clone());
        source.display_name = self.display_name.clone();
        source.base_uri = Some(self.endpoint_uri.clone());
        source.auth_mode = Some(self.auth_mode.as_registry_label());
        source.authority_level = authority_level;
        source.sync_policy = Some(self.access_mode.as_registry_label().to_string());
        source.capabilities = self.capabilities();
        source.metadata.insert(
            "default_media_type".to_string(),
            self.default_media_type.clone(),
        );
        source
    }
}

/// Connector boundary for external databases and API feeds.
#[derive(Clone, Debug)]
pub struct ExternalDatabaseConnector {
    config: ExternalDatabaseConfig,
    endpoint: Url,
}

impl ExternalDatabaseConnector {
    pub fn try_new(config: ExternalDatabaseConfig) -> NexusResult<Self> {
        validate_config(&config)?;
        let endpoint = Url::parse(&config.endpoint_uri).map_err(|error| {
            NexusError::InvalidSource(format!(
                "external database `{}` endpoint_uri is invalid: {error}",
                config.source_id
            ))
        })?;
        Ok(Self { config, endpoint })
    }

    pub fn config(&self) -> &ExternalDatabaseConfig {
        &self.config
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

#[async_trait]
impl KnowledgeSourceConnector for ExternalDatabaseConnector {
    fn source_id(&self) -> &str {
        &self.config.source_id
    }

    fn source_kind(&self) -> KnowledgeSourceKind {
        self.config.source_kind.clone()
    }

    fn capabilities(&self) -> SourceCapabilities {
        self.config.capabilities()
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "external database discovery backend",
        })
    }

    async fn fetch(&self, _item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "external database fetch backend",
        })
    }

    async fn delta(&self, _since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "external database delta backend",
        })
    }
}

fn validate_config(config: &ExternalDatabaseConfig) -> NexusResult<()> {
    if config.source_id.trim().is_empty() {
        return Err(NexusError::InvalidSource(
            "external database source_id must not be empty".to_string(),
        ));
    }
    if config.endpoint_uri.trim().is_empty() {
        return Err(NexusError::InvalidSource(format!(
            "external database `{}` endpoint_uri must not be empty",
            config.source_id
        )));
    }
    if config.default_media_type.trim().is_empty() {
        return Err(NexusError::InvalidSource(format!(
            "external database `{}` default_media_type must not be empty",
            config.source_id
        )));
    }
    Ok(())
}
