//! PubMed connector boundary and capability declaration.

use async_trait::async_trait;
use wendao_nexus_core::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, KnowledgeSourceKind, NexusError,
    NexusResult, RawSourceDocument, SourceCapabilities, SourceCheckpoint, SourceCursor,
    SourceItemRef,
};

/// Configuration for a PubMed source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PubMedConfig {
    pub source_id: String,
    pub api_endpoint: String,
    pub tool_name: Option<String>,
    pub contact_email: Option<String>,
}

impl Default for PubMedConfig {
    fn default() -> Self {
        Self {
            source_id: "pubmed".to_string(),
            api_endpoint: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils".to_string(),
            tool_name: None,
            contact_email: None,
        }
    }
}

/// Capability declaration for PubMed metadata ingestion.
#[derive(Clone, Debug)]
pub struct PubMedConnector {
    config: PubMedConfig,
}

impl PubMedConnector {
    pub fn new(config: PubMedConfig) -> Self {
        Self { config }
    }
}

impl Default for PubMedConnector {
    fn default() -> Self {
        Self::new(PubMedConfig::default())
    }
}

#[async_trait]
impl KnowledgeSourceConnector for PubMedConnector {
    fn source_id(&self) -> &str {
        &self.config.source_id
    }

    fn source_kind(&self) -> KnowledgeSourceKind {
        KnowledgeSourceKind::PubMed
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            discover: true,
            fetch: true,
            delta: true,
            live_query: true,
            local_mirror: true,
            revisions: false,
            structured_metadata: true,
            license_metadata: true,
            access_control: false,
        }
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "live pubmed discovery",
        })
    }

    async fn fetch(&self, _item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "live pubmed fetch",
        })
    }

    async fn delta(&self, _since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        Err(NexusError::Unsupported {
            source_id: self.source_id().to_string(),
            operation: "live pubmed delta",
        })
    }
}
