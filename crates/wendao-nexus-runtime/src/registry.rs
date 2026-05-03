//! Registry traits for jobs, checkpoints, and content-hash dedup.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use uuid::Uuid;
use wendao_nexus_core::{
    NexusError, NexusJobRecord, NexusResult, NexusSourceRecord, SourceCheckpoint,
};

/// Registry facade for configured external sources.
#[async_trait]
pub trait SourceRegistry: Send + Sync {
    async fn upsert_source(&self, source: NexusSourceRecord) -> NexusResult<NexusSourceRecord>;

    async fn get_source(&self, source_id: &str) -> NexusResult<Option<NexusSourceRecord>>;

    async fn list_sources(&self, include_disabled: bool) -> NexusResult<Vec<NexusSourceRecord>>;
}

/// Registry facade for recoverable sync jobs.
#[async_trait]
pub trait JobRegistry: Send + Sync {
    async fn put_job(&self, job: NexusJobRecord) -> NexusResult<NexusJobRecord>;

    async fn get_job(&self, job_id: Uuid) -> NexusResult<Option<NexusJobRecord>>;

    async fn list_jobs(&self, source_id: Option<&str>) -> NexusResult<Vec<NexusJobRecord>>;
}

/// Registry facade for source checkpoints.
#[async_trait]
pub trait CheckpointRegistry: Send + Sync {
    async fn upsert_checkpoint(
        &self,
        checkpoint: SourceCheckpoint,
    ) -> NexusResult<SourceCheckpoint>;

    async fn get_checkpoint(&self, source_id: &str) -> NexusResult<Option<SourceCheckpoint>>;
}

/// Registry facade for content-hash dedup.
#[async_trait]
pub trait ContentHashRegistry: Send + Sync {
    async fn mark_content_hash(&self, content_hash: String) -> NexusResult<bool>;

    async fn contains_content_hash(&self, content_hash: &str) -> NexusResult<bool>;
}

/// Deterministic in-memory registry for tests and early embedding.
#[derive(Clone, Default)]
pub struct InMemoryNexusRegistry {
    inner: Arc<RwLock<RegistryState>>,
}

#[derive(Default)]
struct RegistryState {
    sources: BTreeMap<String, NexusSourceRecord>,
    jobs: BTreeMap<Uuid, NexusJobRecord>,
    checkpoints: BTreeMap<String, SourceCheckpoint>,
    content_hashes: BTreeSet<String>,
}

impl InMemoryNexusRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn read_state(&self) -> NexusResult<std::sync::RwLockReadGuard<'_, RegistryState>> {
        self.inner
            .read()
            .map_err(|_| NexusError::Registry("in-memory registry read lock poisoned".to_string()))
    }

    fn write_state(&self) -> NexusResult<std::sync::RwLockWriteGuard<'_, RegistryState>> {
        self.inner
            .write()
            .map_err(|_| NexusError::Registry("in-memory registry write lock poisoned".to_string()))
    }
}

#[async_trait]
impl SourceRegistry for InMemoryNexusRegistry {
    async fn upsert_source(&self, source: NexusSourceRecord) -> NexusResult<NexusSourceRecord> {
        let mut state = self.write_state()?;
        state
            .sources
            .insert(source.source_id.clone(), source.clone());
        Ok(source)
    }

    async fn get_source(&self, source_id: &str) -> NexusResult<Option<NexusSourceRecord>> {
        let state = self.read_state()?;
        Ok(state.sources.get(source_id).cloned())
    }

    async fn list_sources(&self, include_disabled: bool) -> NexusResult<Vec<NexusSourceRecord>> {
        let state = self.read_state()?;
        let sources = state
            .sources
            .values()
            .filter(|source| include_disabled || source.enabled)
            .cloned()
            .collect();
        Ok(sources)
    }
}

#[async_trait]
impl JobRegistry for InMemoryNexusRegistry {
    async fn put_job(&self, job: NexusJobRecord) -> NexusResult<NexusJobRecord> {
        let mut state = self.write_state()?;
        state.jobs.insert(job.job_id, job.clone());
        Ok(job)
    }

    async fn get_job(&self, job_id: Uuid) -> NexusResult<Option<NexusJobRecord>> {
        let state = self.read_state()?;
        Ok(state.jobs.get(&job_id).cloned())
    }

    async fn list_jobs(&self, source_id: Option<&str>) -> NexusResult<Vec<NexusJobRecord>> {
        let state = self.read_state()?;
        let jobs = state
            .jobs
            .values()
            .filter(|job| match source_id {
                Some(source_id) => job.source_id == source_id,
                None => true,
            })
            .cloned()
            .collect();
        Ok(jobs)
    }
}

#[async_trait]
impl CheckpointRegistry for InMemoryNexusRegistry {
    async fn upsert_checkpoint(
        &self,
        checkpoint: SourceCheckpoint,
    ) -> NexusResult<SourceCheckpoint> {
        let mut state = self.write_state()?;
        state
            .checkpoints
            .insert(checkpoint.source_id.clone(), checkpoint.clone());
        Ok(checkpoint)
    }

    async fn get_checkpoint(&self, source_id: &str) -> NexusResult<Option<SourceCheckpoint>> {
        let state = self.read_state()?;
        Ok(state.checkpoints.get(source_id).cloned())
    }
}

#[async_trait]
impl ContentHashRegistry for InMemoryNexusRegistry {
    async fn mark_content_hash(&self, content_hash: String) -> NexusResult<bool> {
        let mut state = self.write_state()?;
        Ok(state.content_hashes.insert(content_hash))
    }

    async fn contains_content_hash(&self, content_hash: &str) -> NexusResult<bool> {
        let state = self.read_state()?;
        Ok(state.content_hashes.contains(content_hash))
    }
}
