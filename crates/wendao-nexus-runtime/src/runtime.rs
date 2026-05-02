//! Minimal source sync runtime over pluggable registry traits.

use chrono::Utc;
use wendao_nexus_core::{
    DiscoveryBatch, ExternalKnowledgeDocument, KnowledgeSourceConnector, NexusError, NexusJobKind,
    NexusJobRecord, NexusJobStatus, NexusResult, RawSourceDocument, SourceCheckpoint,
    SourceItemRef,
};

use crate::hash::sha256_content_hash;
use crate::normalize::{KnowledgeDocumentNormalizer, NormalizationContext};
use crate::query::LocalKnowledgeStore;
use crate::registry::{CheckpointRegistry, ContentHashRegistry, JobRegistry};

/// Result of one discovery pass.
#[derive(Clone, Debug)]
pub struct DiscoveryOutcome {
    pub job: NexusJobRecord,
    pub batch: DiscoveryBatch,
}

/// Result of one fetch pass.
#[derive(Clone, Debug)]
pub struct FetchOutcome {
    pub job: NexusJobRecord,
    pub document: RawSourceDocument,
    pub content_hash: String,
    pub dedup_hit: bool,
}

/// Result of a fetch, normalize, and local mirror write pass.
#[derive(Clone, Debug)]
pub struct IngestOutcome {
    pub fetch: FetchOutcome,
    pub normalize_job: NexusJobRecord,
    pub document: ExternalKnowledgeDocument,
}

/// Result of accepting a normalized document produced outside Nexus.
#[derive(Clone, Debug)]
pub struct NormalizedIngestOutcome {
    pub job: NexusJobRecord,
    pub document: ExternalKnowledgeDocument,
    pub dedup_hit: bool,
}

/// Minimal sync runtime over pluggable registries.
#[derive(Clone, Debug)]
pub struct NexusSyncRuntime<R> {
    registry: R,
}

impl<R> NexusSyncRuntime<R> {
    pub fn new(registry: R) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &R {
        &self.registry
    }
}

impl<R> NexusSyncRuntime<R>
where
    R: JobRegistry + CheckpointRegistry + ContentHashRegistry,
{
    pub async fn discover_once<C>(&self, connector: &C) -> NexusResult<DiscoveryOutcome>
    where
        C: KnowledgeSourceConnector + ?Sized,
    {
        let mut job = NexusJobRecord::new(connector.source_id(), NexusJobKind::Discover).running();
        self.registry.put_job(job.clone()).await?;

        let prior_checkpoint = self.registry.get_checkpoint(connector.source_id()).await?;
        let cursor = prior_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.cursor.clone());

        match connector.discover(cursor).await {
            Ok(batch) => {
                let mut checkpoint = prior_checkpoint
                    .unwrap_or_else(|| SourceCheckpoint::new(connector.source_id()));
                if let Some(next_cursor) = batch.next_cursor.clone() {
                    job.cursor = Some(next_cursor.clone());
                    checkpoint.cursor = Some(next_cursor);
                }
                checkpoint.last_success_at = Some(Utc::now());
                self.registry.upsert_checkpoint(checkpoint).await?;

                let job = job.finish(NexusJobStatus::Succeeded);
                self.registry.put_job(job.clone()).await?;
                Ok(DiscoveryOutcome { job, batch })
            }
            Err(error) => {
                let failed_job = job.fail(error.to_string());
                self.registry.put_job(failed_job).await?;
                Err(error)
            }
        }
    }

    pub async fn fetch_once<C>(
        &self,
        connector: &C,
        item: SourceItemRef,
    ) -> NexusResult<FetchOutcome>
    where
        C: KnowledgeSourceConnector + ?Sized,
    {
        let job = NexusJobRecord::new(connector.source_id(), NexusJobKind::Fetch).running();
        self.registry.put_job(job.clone()).await?;

        match connector.fetch(item).await {
            Ok(mut document) => {
                let content_hash = document
                    .content_hash
                    .clone()
                    .unwrap_or_else(|| sha256_content_hash(&document.payload));
                let dedup_hit = !self
                    .registry
                    .mark_content_hash(content_hash.clone())
                    .await?;
                document.content_hash = Some(content_hash.clone());

                let mut finished_job = if dedup_hit {
                    job.finish(NexusJobStatus::Deduped)
                } else {
                    job.finish(NexusJobStatus::Succeeded)
                };
                finished_job.dedup_hit = dedup_hit;
                self.registry.put_job(finished_job.clone()).await?;

                Ok(FetchOutcome {
                    job: finished_job,
                    document,
                    content_hash,
                    dedup_hit,
                })
            }
            Err(error) => {
                let failed_job = job.fail(error.to_string());
                self.registry.put_job(failed_job).await?;
                Err(error)
            }
        }
    }

    pub async fn ingest_once<C, N, S>(
        &self,
        connector: &C,
        item: SourceItemRef,
        normalizer: &N,
        store: &S,
        context: NormalizationContext,
    ) -> NexusResult<IngestOutcome>
    where
        C: KnowledgeSourceConnector + ?Sized,
        N: KnowledgeDocumentNormalizer + ?Sized,
        S: LocalKnowledgeStore + ?Sized,
    {
        let fetch = self.fetch_once(connector, item).await?;
        let normalize_job =
            NexusJobRecord::new(connector.source_id(), NexusJobKind::Normalize).running();
        self.registry.put_job(normalize_job.clone()).await?;

        let normalize_result = match normalizer.normalize(fetch.document.clone(), context).await {
            Ok(document) => store.upsert_document(document).await,
            Err(error) => Err(error),
        };

        match normalize_result {
            Ok(document) => {
                let mut checkpoint = self
                    .registry
                    .get_checkpoint(&document.source_id)
                    .await?
                    .unwrap_or_else(|| SourceCheckpoint::new(&document.source_id));
                checkpoint.last_success_at = Some(Utc::now());
                checkpoint.last_content_hash = Some(document.content_hash.clone());
                checkpoint.last_seen_revision = document.provenance.revision_id.clone();
                self.registry.upsert_checkpoint(checkpoint).await?;

                let normalize_job = normalize_job.finish(NexusJobStatus::Succeeded);
                self.registry.put_job(normalize_job.clone()).await?;

                Ok(IngestOutcome {
                    fetch,
                    normalize_job,
                    document,
                })
            }
            Err(error) => {
                let failed_job = normalize_job.fail(error.to_string());
                self.registry.put_job(failed_job).await?;
                Err(error)
            }
        }
    }

    pub async fn upsert_normalized_document<S>(
        &self,
        document: ExternalKnowledgeDocument,
        store: &S,
    ) -> NexusResult<NormalizedIngestOutcome>
    where
        S: LocalKnowledgeStore + ?Sized,
    {
        let job = NexusJobRecord::new(document.source_id.clone(), NexusJobKind::Ingest).running();
        self.registry.put_job(job.clone()).await?;

        if let Err(error) = validate_normalized_document_for_ingest(&document) {
            let failed_job = job.fail(error.to_string());
            self.registry.put_job(failed_job).await?;
            return Err(error);
        }

        let dedup_hit = !self
            .registry
            .mark_content_hash(document.content_hash.clone())
            .await?;
        let upsert_result = store.upsert_document(document).await;

        match upsert_result {
            Ok(document) => {
                let mut checkpoint = self
                    .registry
                    .get_checkpoint(&document.source_id)
                    .await?
                    .unwrap_or_else(|| SourceCheckpoint::new(&document.source_id));
                checkpoint.last_success_at = Some(Utc::now());
                checkpoint.last_content_hash = Some(document.content_hash.clone());
                checkpoint.last_seen_revision = document.provenance.revision_id.clone();
                self.registry.upsert_checkpoint(checkpoint).await?;

                let mut job = if dedup_hit {
                    job.finish(NexusJobStatus::Deduped)
                } else {
                    job.finish(NexusJobStatus::Succeeded)
                };
                job.dedup_hit = dedup_hit;
                self.registry.put_job(job.clone()).await?;

                Ok(NormalizedIngestOutcome {
                    job,
                    document,
                    dedup_hit,
                })
            }
            Err(error) => {
                let failed_job = job.fail(error.to_string());
                self.registry.put_job(failed_job).await?;
                Err(error)
            }
        }
    }
}

fn validate_normalized_document_for_ingest(
    document: &ExternalKnowledgeDocument,
) -> NexusResult<()> {
    if document.source_id.trim().is_empty() {
        return Err(NexusError::Normalize(
            "normalized document handoff requires a source_id".to_string(),
        ));
    }
    if document.external_id.trim().is_empty() {
        return Err(NexusError::Normalize(
            "normalized document handoff requires an external_id".to_string(),
        ));
    }
    if document.canonical_uri.trim().is_empty() {
        return Err(NexusError::Normalize(
            "normalized document handoff requires a canonical_uri".to_string(),
        ));
    }
    if document.content_hash.trim().is_empty() {
        return Err(NexusError::Normalize(
            "normalized document handoff requires a content_hash".to_string(),
        ));
    }
    if document.source_id != document.provenance.source_id {
        return Err(NexusError::Normalize(format!(
            "normalized document handoff source_id `{}` does not match provenance source_id `{}`",
            document.source_id, document.provenance.source_id
        )));
    }
    if document.canonical_uri != document.provenance.canonical_uri {
        return Err(NexusError::Normalize(format!(
            "normalized document handoff canonical_uri `{}` does not match provenance canonical_uri `{}`",
            document.canonical_uri, document.provenance.canonical_uri
        )));
    }
    if document.content_hash != document.provenance.content_hash {
        return Err(NexusError::Normalize(format!(
            "normalized document handoff content_hash `{}` does not match provenance content_hash `{}`",
            document.content_hash, document.provenance.content_hash
        )));
    }
    if document.body.trim().is_empty() && document.sections.is_empty() {
        return Err(NexusError::Normalize(
            "normalized document handoff requires body text or sections".to_string(),
        ));
    }

    Ok(())
}
