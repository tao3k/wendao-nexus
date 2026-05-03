//! Serverless fixture harness for validating Nexus source-pack contracts.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use arrow_array::RecordBatch;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use wendao_nexus_connectors::SourcePack;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceBoundary, EvidenceRecord, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeDocument, ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest,
    ExternalKnowledgeSearchResponse, KnowledgeSourceConnector, NexusError, NexusResult,
    ProvenanceBundle, SourceDomain, SourceItemRef, TrustPolicy,
};
use wendao_nexus_flight::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, NexusFlightBatchProvider, NexusFlightCommand, NexusFlightCommandHandler,
    NexusFlightHandlerError, NexusFlightProviderError, NexusFlightStatusRequest,
    NexusFlightSyncRequest, command_descriptor_from_json, open_rows_from_document,
    search_rows_from_response,
};
use wendao_nexus_runtime::{
    CheckpointRegistry, InMemoryNexusRegistry, LocalFileArtifactStore, NexusSyncRuntime,
    NormalizationContext, PlainTextNormalizer, SourceRegistry,
};

/// Serverless fixture harness for source-pack ingest and Flight command proof.
///
/// This type is intentionally a fixture and conformance tool. It does not own a
/// production server, durable backend, parser, search engine, or local
/// knowledge-store abstraction.
pub struct NexusFixtureHarness {
    provider: NexusFlightBatchProvider<FixtureFlightHandler>,
    artifact_store: LocalFileArtifactStore,
    artifact_root: PathBuf,
    ingest_report: FixtureIngestReport,
}

/// Summary of one source-pack fixture ingest run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureIngestReport {
    pub source_pack_id: String,
    pub source_pack_version: String,
    pub artifact_root: PathBuf,
    pub total_sources: usize,
    pub enabled_sources: usize,
    pub discovered_items: usize,
    pub ingested_documents: usize,
    pub raw_artifacts: usize,
    pub normalized_artifacts: usize,
    pub sources: Vec<FixtureSourceIngestReport>,
}

/// Per-source summary for fixture ingest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureSourceIngestReport {
    pub source_id: String,
    pub discovered_items: usize,
    pub ingested_documents: usize,
    pub raw_artifacts: usize,
    pub normalized_artifacts: usize,
}

impl NexusFixtureHarness {
    /// Load a deterministic source pack, ingest all enabled local corpus
    /// sources, mirror raw and normalized artifacts, and prepare a serverless
    /// Flight command provider.
    pub async fn load_source_pack(
        manifest_path: impl AsRef<Path>,
        artifact_root: impl AsRef<Path>,
    ) -> NexusResult<Self> {
        let pack = SourcePack::from_path(manifest_path)?;
        let artifact_root = artifact_root.as_ref().to_path_buf();
        let registry = InMemoryNexusRegistry::new();
        let artifact_store = LocalFileArtifactStore::open(&artifact_root)?;
        let runtime = NexusSyncRuntime::new(registry.clone());
        let normalizer = PlainTextNormalizer;
        let mut documents = Vec::new();
        let mut source_reports = Vec::new();

        for record in pack.source_records() {
            registry.upsert_source(record).await?;
        }

        for connector in pack.connectors() {
            let source = pack.source(connector.source_id()).ok_or_else(|| {
                NexusError::InvalidSource(format!(
                    "source pack `{}` has connector `{}` without source metadata",
                    pack.manifest().source_pack.id,
                    connector.source_id()
                ))
            })?;
            let discovered = runtime.discover_once(connector).await?;
            let discovered_items = discovered.batch.items.len();
            let mut ingested_documents = 0;
            let mut raw_artifacts = 0;
            let mut normalized_artifacts = 0;

            for item in discovered.batch.items {
                let outcome = runtime
                    .ingest_once_with_artifact_store(
                        connector,
                        SourceItemRef::new(item.source_id, item.external_id),
                        &normalizer,
                        &artifact_store,
                        NormalizationContext::new(
                            source.kind.clone(),
                            source.authority_level.unwrap_or(AuthorityLevel::Unknown),
                        ),
                    )
                    .await?;
                documents.push(outcome.ingest.document);
                ingested_documents += 1;
                raw_artifacts += 1;
                normalized_artifacts += 1;
            }

            source_reports.push(FixtureSourceIngestReport {
                source_id: connector.source_id().to_string(),
                discovered_items,
                ingested_documents,
                raw_artifacts,
                normalized_artifacts,
            });
        }

        let ingest_report = FixtureIngestReport {
            source_pack_id: pack.manifest().source_pack.id.clone(),
            source_pack_version: pack.manifest().source_pack.version.clone(),
            artifact_root: artifact_root.clone(),
            total_sources: pack.manifest().sources.len(),
            enabled_sources: pack.connectors().len(),
            discovered_items: source_reports
                .iter()
                .map(|report| report.discovered_items)
                .sum(),
            ingested_documents: source_reports
                .iter()
                .map(|report| report.ingested_documents)
                .sum(),
            raw_artifacts: source_reports
                .iter()
                .map(|report| report.raw_artifacts)
                .sum(),
            normalized_artifacts: source_reports
                .iter()
                .map(|report| report.normalized_artifacts)
                .sum(),
            sources: source_reports,
        };
        let provider = NexusFlightBatchProvider::new(FixtureFlightHandler {
            evidence_view: FixtureEvidenceView {
                documents,
                registry,
            },
        });

        Ok(Self {
            provider,
            artifact_store,
            artifact_root,
            ingest_report,
        })
    }

    /// Return the fixture ingest report captured during construction.
    pub fn ingest_report(&self) -> &FixtureIngestReport {
        &self.ingest_report
    }

    /// Return the artifact root used by this fixture harness.
    pub fn artifact_root(&self) -> &Path {
        &self.artifact_root
    }

    /// Return the local artifact store used by this fixture harness.
    pub fn artifact_store(&self) -> &LocalFileArtifactStore {
        &self.artifact_store
    }

    /// Dispatch one typed Nexus Flight command into a route-specific Arrow
    /// batch without starting a server.
    pub async fn handle_command(
        &self,
        command: NexusFlightCommand,
    ) -> Result<RecordBatch, NexusFlightProviderError> {
        self.provider.handle_command(command).await
    }

    /// Decode an already-encoded `FlightDescriptor::cmd` JSON payload and
    /// dispatch it into a route-specific Arrow batch.
    pub async fn handle_encoded_command(
        &self,
        bytes: Vec<u8>,
    ) -> Result<RecordBatch, NexusFlightProviderError> {
        let descriptor = command_descriptor_from_json(bytes);
        self.provider.handle_descriptor(&descriptor).await
    }
}

#[derive(Clone)]
struct FixtureFlightHandler {
    evidence_view: FixtureEvidenceView,
}

#[async_trait]
impl NexusFlightCommandHandler for FixtureFlightHandler {
    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> Result<Vec<FlightSearchResultRow>, NexusFlightHandlerError> {
        let response = self.evidence_view.search(request).await?;
        search_rows_from_response(&response).map_err(handler_error)
    }

    async fn open(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> Result<Vec<FlightOpenDocumentRow>, NexusFlightHandlerError> {
        let document = self.evidence_view.open(&request)?;
        open_rows_from_document(
            document,
            request.include_sections,
            request.include_provenance,
        )
        .map_err(handler_error)
    }

    async fn sync(
        &self,
        request: NexusFlightSyncRequest,
    ) -> Result<Vec<FlightSyncResultRow>, NexusFlightHandlerError> {
        Err(NexusFlightHandlerError::message(format!(
            "fixture source `{}` sync belongs to the embedding Wendao runtime",
            request.source_id
        )))
    }

    async fn status(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<FlightStatusRow>, NexusFlightHandlerError> {
        self.evidence_view.status(request).await
    }

    async fn compare(
        &self,
        request: ExternalKnowledgeCompareRequest,
    ) -> Result<Vec<FlightCompareResultRow>, NexusFlightHandlerError> {
        self.evidence_view.compare(request).await
    }
}

#[derive(Clone)]
struct FixtureEvidenceView {
    documents: Vec<ExternalKnowledgeDocument>,
    registry: InMemoryNexusRegistry,
}

impl FixtureEvidenceView {
    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> Result<ExternalKnowledgeSearchResponse, NexusFlightHandlerError> {
        let mut ranked_records = Vec::new();
        for document in self
            .documents
            .iter()
            .filter(|document| source_filter_allows(document, &request.sources))
            .filter(|document| trust_policy_allows(document, &request.trust_policy))
            .filter(|document| freshness_filter_allows(document, request.freshness_days))
            .filter(|document| document_matches_query(document, &request.query))
        {
            let domain = self.source_domain(&document.source_id).await?;
            ranked_records.push(RankedEvidenceRecord {
                record: evidence_record_from_document(document, &request.query),
                authority_rank: authority_rank(document.provenance.authority_level),
                freshness_rank: freshness_rank(document),
                evidence_kind_rank: evidence_kind_rank(document),
                domain_rank: domain_rank(&domain),
                exact_match_rank: exact_match_rank(document, &request.query),
            });
        }

        ranked_records.sort_by(|left, right| {
            right
                .authority_rank
                .cmp(&left.authority_rank)
                .then_with(|| right.freshness_rank.cmp(&left.freshness_rank))
                .then_with(|| right.evidence_kind_rank.cmp(&left.evidence_kind_rank))
                .then_with(|| right.domain_rank.cmp(&left.domain_rank))
                .then_with(|| right.exact_match_rank.cmp(&left.exact_match_rank))
                .then_with(|| left.record.source_id.cmp(&right.record.source_id))
                .then_with(|| left.record.external_id.cmp(&right.record.external_id))
        });

        let records = ranked_records
            .into_iter()
            .take(request.limit)
            .map(|ranked| ranked.record)
            .collect();

        Ok(ExternalKnowledgeSearchResponse {
            query: request.query,
            records,
            generated_at: Utc::now(),
        })
    }

    fn open(
        &self,
        request: &ExternalKnowledgeOpenRequest,
    ) -> Result<&ExternalKnowledgeDocument, NexusFlightHandlerError> {
        self.documents
            .iter()
            .find(|document| {
                document.source_id == request.source_id
                    && document.external_id == request.external_id
            })
            .ok_or_else(|| {
                NexusFlightHandlerError::message(format!(
                    "fixture document `{}/{}` not found",
                    request.source_id, request.external_id
                ))
            })
    }

    async fn status(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<FlightStatusRow>, NexusFlightHandlerError> {
        let source_ids = self.status_source_ids(request).await?;
        let mut rows = Vec::with_capacity(source_ids.len());

        for source_id in source_ids {
            let source = self
                .registry
                .get_source(&source_id)
                .await
                .map_err(handler_error)?;
            let enabled = source.as_ref().is_none_or(|source| source.enabled);
            match self
                .registry
                .get_checkpoint(&source_id)
                .await
                .map_err(handler_error)?
            {
                Some(checkpoint) => {
                    let mut row = FlightStatusRow::from(&checkpoint);
                    row.enabled = enabled;
                    rows.push(row);
                }
                None => rows.push(FlightStatusRow {
                    source_id,
                    enabled,
                    last_success_at: None,
                    last_seen_revision: None,
                    last_content_hash: None,
                    rate_limit_state: None,
                }),
            }
        }

        Ok(rows)
    }

    async fn compare(
        &self,
        request: ExternalKnowledgeCompareRequest,
    ) -> Result<Vec<FlightCompareResultRow>, NexusFlightHandlerError> {
        let mut search = ExternalKnowledgeSearchRequest::new(request.claim.clone());
        search.sources = request.sources;
        let minimum_authority = request.trust_policy.minimum_authority;
        search.trust_policy = request.trust_policy;
        search.limit = 20;

        let response = self.search(search).await?;
        let evidence_records = response
            .records
            .iter()
            .map(|record| record.provenance.primary.clone())
            .collect::<Vec<_>>();
        let insufficient_authority = response.records.is_empty();
        let stale_evidence = response
            .records
            .iter()
            .any(|record| provenance_is_stale(&record.provenance.primary));
        let boundary = EvidenceBoundary {
            records: evidence_records,
            minimum_authority,
            insufficient_authority,
            stale_evidence,
            conflict_detected: false,
        };

        Ok(vec![FlightCompareResultRow {
            claim: request.claim,
            verdict: compare_verdict(&boundary).to_string(),
            conflict_detected: boundary.conflict_detected,
            insufficient_authority: boundary.insufficient_authority,
            stale_evidence: boundary.stale_evidence,
            provenance_json: Some(serde_json::to_string(&boundary).map_err(handler_error)?),
        }])
    }

    async fn status_source_ids(
        &self,
        request: NexusFlightStatusRequest,
    ) -> Result<Vec<String>, NexusFlightHandlerError> {
        if !request.sources.is_empty() {
            return Ok(request.sources);
        }

        let registered_sources = self
            .registry
            .list_sources(true)
            .await
            .map_err(handler_error)?;
        if !registered_sources.is_empty() {
            return Ok(registered_sources
                .into_iter()
                .map(|source| source.source_id)
                .collect());
        }

        Ok(self
            .documents
            .iter()
            .map(|document| document.source_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect())
    }

    async fn source_domain(
        &self,
        source_id: &str,
    ) -> Result<SourceDomain, NexusFlightHandlerError> {
        Ok(self
            .registry
            .get_source(source_id)
            .await
            .map_err(handler_error)?
            .map(|source| source.source_pack_domain())
            .unwrap_or_default())
    }
}

struct RankedEvidenceRecord {
    record: EvidenceRecord,
    authority_rank: u8,
    freshness_rank: i64,
    evidence_kind_rank: u8,
    domain_rank: u8,
    exact_match_rank: u8,
}

fn source_filter_allows(document: &ExternalKnowledgeDocument, sources: &[String]) -> bool {
    sources.is_empty()
        || sources
            .iter()
            .any(|source_id| source_id == &document.source_id)
}

fn trust_policy_allows(document: &ExternalKnowledgeDocument, policy: &TrustPolicy) -> bool {
    let authority = document.provenance.authority_level;
    authority >= policy.minimum_authority
        && (policy.allow_community_sources || authority != AuthorityLevel::Community)
}

fn freshness_filter_allows(
    document: &ExternalKnowledgeDocument,
    freshness_days: Option<u32>,
) -> bool {
    match freshness_days {
        Some(days) => document.fetched_at >= Utc::now() - Duration::days(days.into()),
        None => true,
    }
}

fn document_matches_query(document: &ExternalKnowledgeDocument, query: &str) -> bool {
    let terms = normalized_terms(query);
    if terms.is_empty() {
        return true;
    }

    let haystack = normalized_document_text(document);
    terms.iter().all(|term| haystack.contains(term))
}

fn evidence_record_from_document(
    document: &ExternalKnowledgeDocument,
    query: &str,
) -> EvidenceRecord {
    EvidenceRecord {
        source_id: document.source_id.clone(),
        external_id: document.external_id.clone(),
        title: document.title.clone(),
        snippet: best_snippet(document, query),
        score: Some("1.0".to_string()),
        evidence_kind: document.metadata.evidence_kind(),
        provenance: ProvenanceBundle {
            primary: document.provenance.clone(),
            corroborating: Vec::new(),
            conflicting: Vec::new(),
        },
    }
}

fn authority_rank(authority: AuthorityLevel) -> u8 {
    match authority {
        AuthorityLevel::Official => 5,
        AuthorityLevel::PeerReviewed => 4,
        AuthorityLevel::CustomerInternal => 3,
        AuthorityLevel::Curated => 2,
        AuthorityLevel::Community => 1,
        AuthorityLevel::Unknown => 0,
    }
}

fn freshness_rank(document: &ExternalKnowledgeDocument) -> i64 {
    document
        .source_updated_at
        .or(document.metadata.published_at)
        .unwrap_or(document.fetched_at)
        .timestamp()
}

fn evidence_kind_rank(document: &ExternalKnowledgeDocument) -> u8 {
    match document.metadata.evidence_kind().wire_label().as_str() {
        "law_clause" | "guideline" | "trial_result" => 4,
        "market_signal" | "customer_internal_note" => 3,
        "statistic" | "definition" | "claim" => 2,
        "review_article" => 1,
        _ => 0,
    }
}

fn domain_rank(domain: &SourceDomain) -> u8 {
    match domain {
        SourceDomain::Generic => 0,
        SourceDomain::Other(_) => 1,
        _ => 2,
    }
}

fn exact_match_rank(document: &ExternalKnowledgeDocument, query: &str) -> u8 {
    let phrase = normalize_text(query);
    if phrase.is_empty() {
        return 0;
    }
    u8::from(normalized_document_text(document).contains(&phrase))
}

fn provenance_is_stale(provenance: &wendao_nexus_core::ProvenanceRecord) -> bool {
    provenance.published_at.unwrap_or(provenance.fetched_at) < Utc::now() - Duration::days(365)
}

fn compare_verdict(boundary: &EvidenceBoundary) -> &'static str {
    if boundary.insufficient_authority {
        "insufficient_authority"
    } else if boundary.stale_evidence {
        "stale_evidence"
    } else {
        "evidence_available"
    }
}

fn best_snippet(document: &ExternalKnowledgeDocument, query: &str) -> String {
    let terms = normalized_terms(query);
    let candidate = document
        .sections
        .iter()
        .map(|section| section.text.as_str())
        .find(|text| {
            let normalized = normalize_text(text);
            terms.iter().all(|term| normalized.contains(term))
        })
        .unwrap_or(&document.body);

    truncate_snippet(candidate)
}

fn normalized_document_text(document: &ExternalKnowledgeDocument) -> String {
    let mut text = format!("{} {}", document.title, document.body);
    for section in &document.sections {
        text.push(' ');
        text.push_str(&section.heading_path.join(" "));
        text.push(' ');
        text.push_str(&section.text);
    }
    normalize_text(&text)
}

fn normalized_terms(query: &str) -> Vec<String> {
    normalize_text(query)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
}

fn truncate_snippet(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(240).collect()
}

fn handler_error(error: impl std::fmt::Display) -> NexusFlightHandlerError {
    NexusFlightHandlerError::message(error.to_string())
}
