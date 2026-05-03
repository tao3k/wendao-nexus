//! Opt-in external database live probe connector.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use wendao_nexus_core::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, NexusError, NexusResult,
    RawSourceDocument, SOURCE_METADATA_TITLE_KEY, SourceCapabilities, SourceCheckpoint,
    SourceCursor, SourceItemRef,
};

use crate::external_database::{ExternalDatabaseConfig, ExternalDatabaseConnector};

/// Feature-gated probe config for bounded live validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalDatabaseProbeConfig {
    pub database: ExternalDatabaseConfig,
    pub timeout_millis: u64,
    pub max_bytes: usize,
    pub user_agent: String,
}

impl ExternalDatabaseProbeConfig {
    pub fn new(database: ExternalDatabaseConfig) -> Self {
        Self {
            database,
            timeout_millis: 5_000,
            max_bytes: 1_048_576,
            user_agent: "wendao-nexus-live-probe/0.1".to_string(),
        }
    }
}

/// Minimal GET-only live probe.
#[derive(Clone, Debug)]
pub struct ExternalDatabaseProbeConnector {
    config: ExternalDatabaseProbeConfig,
    client: reqwest::Client,
}

impl ExternalDatabaseProbeConnector {
    pub fn try_new(config: ExternalDatabaseProbeConfig) -> NexusResult<Self> {
        ExternalDatabaseConnector::try_new(config.database.clone())?;
        if config.timeout_millis == 0 {
            return Err(NexusError::InvalidSource(format!(
                "external database probe `{}` timeout_millis must be greater than zero",
                config.database.source_id
            )));
        }
        if config.max_bytes == 0 {
            return Err(NexusError::InvalidSource(format!(
                "external database probe `{}` max_bytes must be greater than zero",
                config.database.source_id
            )));
        }
        if config.user_agent.trim().is_empty() {
            return Err(NexusError::InvalidSource(format!(
                "external database probe `{}` user_agent must not be empty",
                config.database.source_id
            )));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_millis))
            .user_agent(config.user_agent.clone())
            .build()
            .map_err(|error| {
                NexusError::InvalidSource(format!(
                    "external database probe `{}` client configuration is invalid: {error}",
                    config.database.source_id
                ))
            })?;
        Ok(Self { config, client })
    }

    pub fn config(&self) -> &ExternalDatabaseProbeConfig {
        &self.config
    }

    async fn fetch_endpoint(&self, item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        if item.source_id != self.config.database.source_id {
            return Err(NexusError::NotFound {
                source_id: self.config.database.source_id.clone(),
                external_id: item.external_id,
            });
        }

        let response = self
            .client
            .get(&self.config.database.endpoint_uri)
            .send()
            .await
            .map_err(|error| {
                NexusError::Sync(format!(
                    "external database probe `{}` request failed: {error}",
                    self.config.database.source_id
                ))
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(NexusError::Sync(format!(
                "external database probe `{}` returned HTTP {status}",
                self.config.database.source_id
            )));
        }

        let media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.config.database.default_media_type.clone());
        if !media_type.contains("json") {
            return Err(NexusError::Sync(format!(
                "external database probe `{}` expected JSON media type but received `{media_type}`",
                self.config.database.source_id
            )));
        }
        let bytes = response.bytes().await.map_err(|error| {
            NexusError::Sync(format!(
                "external database probe `{}` response read failed: {error}",
                self.config.database.source_id
            ))
        })?;
        if bytes.len() > self.config.max_bytes {
            return Err(NexusError::Sync(format!(
                "external database probe `{}` exceeded max_bytes {}",
                self.config.database.source_id, self.config.max_bytes
            )));
        }

        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("live_probe".to_string(), "true".to_string());
        metadata.insert("http_status".to_string(), status.as_u16().to_string());
        metadata.insert(
            SOURCE_METADATA_TITLE_KEY.to_string(),
            item.external_id.clone(),
        );

        Ok(RawSourceDocument {
            source_id: self.config.database.source_id.clone(),
            external_id: item.external_id,
            canonical_uri: self.config.database.endpoint_uri.clone(),
            media_type,
            payload: bytes.to_vec(),
            fetched_at: Utc::now(),
            source_updated_at: None,
            content_hash: None,
            metadata,
        })
    }
}

#[async_trait]
impl KnowledgeSourceConnector for ExternalDatabaseProbeConnector {
    fn source_id(&self) -> &str {
        &self.config.database.source_id
    }

    fn source_kind(&self) -> wendao_nexus_core::KnowledgeSourceKind {
        self.config.database.source_kind.clone()
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            discover: false,
            fetch: true,
            delta: false,
            live_query: true,
            local_mirror: false,
            revisions: self.config.database.supports_revisions,
            structured_metadata: self.config.database.structured_metadata,
            license_metadata: self.config.database.license_metadata,
            access_control: self.config.database.access_control,
        }
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "external database probe discovery",
        })
    }

    async fn fetch(&self, item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        self.fetch_endpoint(item).await
    }

    async fn delta(&self, _since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "external database probe delta",
        })
    }
}
