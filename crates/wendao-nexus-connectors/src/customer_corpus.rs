//! Customer private corpus connector boundary.

use async_trait::async_trait;
use wendao_nexus_core::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, KnowledgeSourceKind, NexusError,
    NexusResult, RawSourceDocument, SourceCapabilities, SourceCheckpoint, SourceCursor,
    SourceItemRef,
};

/// Configuration for a customer-owned private corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomerCorpusConfig {
    pub source_id: String,
    pub tenant_id: String,
    pub display_name: String,
}

/// Connector boundary for customer private corpus ingestion.
#[derive(Clone, Debug)]
pub struct CustomerCorpusConnector {
    config: CustomerCorpusConfig,
}

impl CustomerCorpusConnector {
    pub fn new(config: CustomerCorpusConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl KnowledgeSourceConnector for CustomerCorpusConnector {
    fn source_id(&self) -> &str {
        &self.config.source_id
    }

    fn source_kind(&self) -> KnowledgeSourceKind {
        KnowledgeSourceKind::CustomerPrivateCorpus
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            discover: true,
            fetch: true,
            delta: true,
            live_query: false,
            local_mirror: true,
            revisions: true,
            structured_metadata: true,
            license_metadata: false,
            access_control: true,
        }
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "customer corpus discovery backend",
        })
    }

    async fn fetch(&self, _item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "customer corpus fetch backend",
        })
    }

    async fn delta(&self, _since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "customer corpus delta backend",
        })
    }
}
