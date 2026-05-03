//! Thin handler-to-`RecordBatch` adapter for Wendao-side Flight routing.

use arrow_array::RecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_schema::ArrowError;
use async_trait::async_trait;
use thiserror::Error;
use wendao_nexus_core::{
    ExternalKnowledgeCompareRequest, ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest,
};

use crate::batch::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, compare_result_record_batch, open_document_record_batch,
    search_result_record_batch, status_record_batch, sync_result_record_batch,
};
use crate::command::{
    NexusFlightCommand, NexusFlightCommandError, NexusFlightStatusRequest, NexusFlightSyncRequest,
};

/// Wendao-side handler contract for Nexus Flight commands.
///
/// Implementors own storage, query, connector, and runtime integration. This
/// trait only fixes the boundary between decoded commands and Arrow batches.
#[async_trait]
pub trait NexusFlightCommandHandler: Send + Sync {
    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> Result<Vec<FlightSearchResultRow>, NexusFlightHandlerError>;

    async fn open(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> Result<Vec<FlightOpenDocumentRow>, NexusFlightHandlerError>;

    async fn sync(
        &self,
        request: NexusFlightSyncRequest,
    ) -> Result<Vec<FlightSyncResultRow>, NexusFlightHandlerError>;

    async fn status(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<FlightStatusRow>, NexusFlightHandlerError>;

    async fn compare(
        &self,
        request: ExternalKnowledgeCompareRequest,
    ) -> Result<Vec<FlightCompareResultRow>, NexusFlightHandlerError>;
}

/// Converts decoded Nexus commands into route-specific Arrow `RecordBatch`es.
#[derive(Clone, Debug)]
pub struct NexusFlightBatchProvider<H> {
    handler: H,
}

impl<H> NexusFlightBatchProvider<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    pub fn handler(&self) -> &H {
        &self.handler
    }
}

impl<H> NexusFlightBatchProvider<H>
where
    H: NexusFlightCommandHandler,
{
    /// Decode a command descriptor and return the route-specific batch.
    pub async fn handle_descriptor(
        &self,
        descriptor: &FlightDescriptor,
    ) -> Result<RecordBatch, NexusFlightProviderError> {
        let command = NexusFlightCommand::from_descriptor(descriptor)?;
        self.handle_command(command).await
    }

    /// Dispatch one typed command and return the route-specific batch.
    pub async fn handle_command(
        &self,
        command: NexusFlightCommand,
    ) -> Result<RecordBatch, NexusFlightProviderError> {
        let batch = match command {
            NexusFlightCommand::Search(request) => {
                let rows = self.handler.search(request).await?;
                search_result_record_batch(&rows)?
            }
            NexusFlightCommand::Open(request) => {
                let rows = self.handler.open(request).await?;
                open_document_record_batch(&rows)?
            }
            NexusFlightCommand::Sync(request) => {
                let rows = self.handler.sync(request).await?;
                sync_result_record_batch(&rows)?
            }
            NexusFlightCommand::Status(request) => {
                let rows = self.handler.status(request).await?;
                status_record_batch(&rows)?
            }
            NexusFlightCommand::Compare(request) => {
                let rows = self.handler.compare(request).await?;
                compare_result_record_batch(&rows)?
            }
        };

        Ok(batch)
    }
}

/// Error returned by a downstream handler implementation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum NexusFlightHandlerError {
    #[error("{message}")]
    Message { message: String },
}

impl NexusFlightHandlerError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message {
            message: message.into(),
        }
    }
}

impl From<&str> for NexusFlightHandlerError {
    fn from(message: &str) -> Self {
        Self::message(message)
    }
}

impl From<String> for NexusFlightHandlerError {
    fn from(message: String) -> Self {
        Self::message(message)
    }
}

/// Error returned while decoding, dispatching, or building Flight batches.
#[derive(Debug, Error)]
pub enum NexusFlightProviderError {
    #[error(transparent)]
    Command(#[from] NexusFlightCommandError),

    #[error(transparent)]
    Handler(#[from] NexusFlightHandlerError),

    #[error(transparent)]
    Arrow(#[from] ArrowError),
}
