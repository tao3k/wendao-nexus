//! Wikipedia or MediaWiki connector boundary and capability declaration.

use async_trait::async_trait;
use wendao_nexus_core::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, KnowledgeSourceKind, NexusError,
    NexusResult, RawSourceDocument, SourceCapabilities, SourceCheckpoint, SourceCursor,
    SourceItemRef,
};

/// Configuration for a Wikipedia or MediaWiki source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WikipediaConfig {
    pub source_id: String,
    pub api_endpoint: String,
}

impl Default for WikipediaConfig {
    fn default() -> Self {
        Self {
            source_id: "wikipedia".to_string(),
            api_endpoint: "https://en.wikipedia.org/w/api.php".to_string(),
        }
    }
}

/// Capability declaration for Wikipedia/MediaWiki ingestion.
#[derive(Clone, Debug)]
pub struct WikipediaConnector {
    config: WikipediaConfig,
}

impl WikipediaConnector {
    pub fn new(config: WikipediaConfig) -> Self {
        Self { config }
    }
}

impl Default for WikipediaConnector {
    fn default() -> Self {
        Self::new(WikipediaConfig::default())
    }
}

#[async_trait]
impl KnowledgeSourceConnector for WikipediaConnector {
    fn source_id(&self) -> &str {
        &self.config.source_id
    }

    fn source_kind(&self) -> KnowledgeSourceKind {
        KnowledgeSourceKind::Wikipedia
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            discover: false,
            fetch: false,
            delta: false,
            live_query: false,
            local_mirror: false,
            revisions: true,
            structured_metadata: true,
            license_metadata: true,
            access_control: false,
        }
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "live wikipedia discovery",
        })
    }

    async fn fetch(&self, _item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "live wikipedia fetch",
        })
    }

    async fn delta(&self, _since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "live wikipedia delta",
        })
    }
}
