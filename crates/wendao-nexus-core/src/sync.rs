//! Runtime-independent sync job contracts for recoverable source ingestion.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::source::SourceCursor;

/// Source sync job kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum NexusJobKind {
    Discover,
    Fetch,
    Normalize,
    Ingest,
    Delta,
    Refresh,
}

/// Recoverable source sync job status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum NexusJobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cached,
    Deduped,
}

/// Runtime-independent job record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NexusJobRecord {
    pub job_id: Uuid,
    pub source_id: String,
    pub job_kind: NexusJobKind,
    pub status: NexusJobStatus,
    pub cursor: Option<SourceCursor>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub dedup_hit: bool,
}

impl NexusJobRecord {
    pub fn new(source_id: impl Into<String>, job_kind: NexusJobKind) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            source_id: source_id.into(),
            job_kind,
            status: NexusJobStatus::Pending,
            cursor: None,
            started_at: Utc::now(),
            finished_at: None,
            error: None,
            dedup_hit: false,
        }
    }

    pub fn running(mut self) -> Self {
        self.status = NexusJobStatus::Running;
        self
    }

    pub fn finish(mut self, status: NexusJobStatus) -> Self {
        self.status = status;
        self.finished_at = Some(Utc::now());
        self
    }

    pub fn fail(mut self, error: impl Into<String>) -> Self {
        self.status = NexusJobStatus::Failed;
        self.finished_at = Some(Utc::now());
        self.error = Some(error.into());
        self
    }
}
