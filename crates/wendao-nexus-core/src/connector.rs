//! Connector trait for external knowledge sources.

use async_trait::async_trait;

use crate::document::RawSourceDocument;
use crate::error::NexusResult;
use crate::source::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceKind, SourceCapabilities, SourceCheckpoint,
    SourceCursor, SourceItemRef,
};

/// External knowledge source connector.
///
/// Connectors expose source identity and capabilities up front so the runtime
/// can make scheduling, rate-limit, and recovery decisions before doing work.
#[async_trait]
pub trait KnowledgeSourceConnector: Send + Sync {
    fn source_id(&self) -> &str;

    fn source_kind(&self) -> KnowledgeSourceKind;

    fn capabilities(&self) -> SourceCapabilities;

    async fn discover(&self, cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch>;

    async fn fetch(&self, item: SourceItemRef) -> NexusResult<RawSourceDocument>;

    async fn delta(&self, since: SourceCheckpoint) -> NexusResult<DeltaBatch>;
}
