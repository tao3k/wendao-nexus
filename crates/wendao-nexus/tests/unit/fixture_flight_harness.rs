use std::collections::BTreeSet;
use std::path::PathBuf;

use arrow_array::{Array, BooleanArray, RecordBatch, StringArray};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use wendao_nexus_connectors::SourcePack;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceBoundary, EvidenceConflictMode, EvidenceRecord,
    ExternalKnowledgeCompareRequest, ExternalKnowledgeDocument, ExternalKnowledgeOpenRequest,
    ExternalKnowledgeSearchRequest, ExternalKnowledgeSearchResponse, KnowledgeSourceConnector,
    ProvenanceBundle, SourceItemRef, TrustPolicy,
};
use wendao_nexus_flight::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, NexusFlightBatchProvider, NexusFlightCommand, NexusFlightCommandHandler,
    NexusFlightHandlerError, NexusFlightStatusRequest, NexusFlightSyncRequest,
    open_rows_from_document, search_rows_from_response,
};
use wendao_nexus_runtime::{
    ArtifactKind, ArtifactStore, CheckpointRegistry, InMemoryNexusRegistry, LocalFileArtifactStore,
    NexusSyncRuntime, NormalizationContext, PlainTextNormalizer, SourceRegistry,
};

#[tokio::test]
async fn fixture_flight_harness_serves_source_pack_without_server_or_backend_database() {
    let harness = FixtureFlightHarness::build().await;

    let mut search = ExternalKnowledgeSearchRequest::new("deterministic fixture");
    search.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::Curated);
    search.limit = 10;
    let search_batch = harness
        .handle_descriptor(NexusFlightCommand::Search(search))
        .await;

    assert_eq!(search_batch.num_rows(), 2);
    assert_eq!(
        string_values(&search_batch, "title"),
        vec![
            "GLP-1 cardiovascular fixture article".to_string(),
            "Demo Clinical Guideline".to_string(),
        ]
    );
    assert_eq!(
        string_values(&search_batch, "source_kind"),
        vec!["PubMed".to_string(), "MedicalJournal".to_string()]
    );
    assert_eq!(
        string_column(&search_batch, "doi").value(0),
        "10.1000/demo1"
    );
    assert_eq!(
        string_column(&search_batch, "evidence_kind").value(0),
        "document"
    );

    let open_batch = harness
        .handle_descriptor(NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
            source_id: "demo-guideline".to_string(),
            external_id: "medical/guideline-demo".to_string(),
            include_sections: true,
            include_provenance: true,
        }))
        .await;

    assert_eq!(open_batch.num_rows(), 1);
    assert_eq!(
        string_column(&open_batch, "title").value(0),
        "Demo Clinical Guideline"
    );
    assert!(
        string_column(&open_batch, "body")
            .value(0)
            .contains("Deterministic clinical guidance fixture")
    );
    assert!(
        !open_batch
            .column_by_name("provenance_json")
            .unwrap()
            .is_null(0)
    );

    let status_batch = harness
        .handle_descriptor(NexusFlightCommand::Status(
            NexusFlightStatusRequest::all_sources(),
        ))
        .await;

    assert_eq!(status_batch.num_rows(), 2);
    assert_eq!(
        string_values(&status_batch, "source_id"),
        vec!["demo-guideline".to_string(), "demo-pubmed".to_string()]
    );
    assert!(
        !status_batch
            .column_by_name("last_content_hash")
            .unwrap()
            .is_null(0)
    );
    assert!(
        !status_batch
            .column_by_name("last_content_hash")
            .unwrap()
            .is_null(1)
    );

    let compare_batch = harness
        .handle_descriptor(NexusFlightCommand::Compare(
            ExternalKnowledgeCompareRequest {
                claim: "GLP-1 cardiovascular".to_string(),
                sources: vec!["demo-pubmed".to_string()],
                mode: EvidenceConflictMode::EvidenceConflictCheck,
                trust_policy: TrustPolicy::authority_at_least(AuthorityLevel::PeerReviewed),
            },
        ))
        .await;
    assert_eq!(
        string_column(&compare_batch, "verdict").value(0),
        "evidence_available"
    );
    assert!(!bool_column(&compare_batch, "insufficient_authority").value(0));
    assert!(
        !compare_batch
            .column_by_name("provenance_json")
            .unwrap()
            .is_null(0)
    );

    let artifacts = harness
        .artifact_store
        .list_artifacts("demo-pubmed", "medical/pubmed-demo-1")
        .await
        .unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0].kind, ArtifactKind::RawSourcePayload);
    assert_eq!(artifacts[1].kind, ArtifactKind::NormalizedDocument);

    harness.cleanup();
}

#[tokio::test]
async fn fixture_flight_harness_serves_customer_private_business_scenario() {
    let harness =
        FixtureFlightHarness::build_with_manifest(customer_private_pack_fixture_manifest()).await;

    let mut search = ExternalKnowledgeSearchRequest::new("QA reviewer approval");
    search.sources = vec!["customer-sop-demo".to_string()];
    search.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::CustomerInternal);
    search.limit = 10;
    let search_batch = harness
        .handle_descriptor(NexusFlightCommand::Search(search))
        .await;

    assert_eq!(search_batch.num_rows(), 1);
    assert_eq!(
        string_column(&search_batch, "title").value(0),
        "Clinical Trial Intake SOP"
    );
    assert_eq!(
        string_column(&search_batch, "source_kind").value(0),
        "CustomerPrivateCorpus"
    );
    assert_eq!(
        string_column(&search_batch, "authority_level").value(0),
        "CustomerInternal"
    );
    assert!(
        string_column(&search_batch, "snippet")
            .value(0)
            .contains("QA reviewer approval")
    );

    let open_batch = harness
        .handle_descriptor(NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
            source_id: "customer-sop-demo".to_string(),
            external_id: "customer/sop/clinical-trial-intake".to_string(),
            include_sections: true,
            include_provenance: true,
        }))
        .await;

    assert_eq!(open_batch.num_rows(), 1);
    assert!(
        string_column(&open_batch, "metadata_json")
            .value(0)
            .contains("\"tenant_id\":\"acme-bio\"")
    );
    assert!(
        string_column(&open_batch, "metadata_json")
            .value(0)
            .contains("Customer Confidential")
    );
    assert!(
        !open_batch
            .column_by_name("provenance_json")
            .unwrap()
            .is_null(0)
    );

    let status_batch = harness
        .handle_descriptor(NexusFlightCommand::Status(
            NexusFlightStatusRequest::all_sources(),
        ))
        .await;
    assert_eq!(
        string_values(&status_batch, "source_id"),
        vec![
            "customer-crm-demo".to_string(),
            "customer-sop-demo".to_string()
        ]
    );
    assert!(!bool_column(&status_batch, "enabled").value(0));
    assert!(bool_column(&status_batch, "enabled").value(1));
    assert!(
        status_batch
            .column_by_name("last_content_hash")
            .unwrap()
            .is_null(0)
    );
    assert!(
        !status_batch
            .column_by_name("last_content_hash")
            .unwrap()
            .is_null(1)
    );

    harness.cleanup();
}

#[tokio::test]
async fn fixture_flight_harness_serves_legal_and_agriculture_evidence_kinds() {
    let legal_harness =
        FixtureFlightHarness::build_with_manifest(legal_pack_fixture_manifest()).await;
    let mut legal_search = ExternalKnowledgeSearchRequest::new("retain audit evidence");
    legal_search.sources = vec!["legal-compliance-demo".to_string()];
    legal_search.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::Official);
    legal_search.limit = 10;
    let legal_batch = legal_harness
        .handle_descriptor(NexusFlightCommand::Search(legal_search))
        .await;

    assert_eq!(legal_batch.num_rows(), 1);
    assert_eq!(
        string_column(&legal_batch, "title").value(0),
        "Example Privacy Code Article 12"
    );
    assert_eq!(
        string_column(&legal_batch, "source_kind").value(0),
        "LegalCorpus"
    );
    assert_eq!(
        string_column(&legal_batch, "authority_level").value(0),
        "Official"
    );
    assert_eq!(
        string_column(&legal_batch, "jurisdiction").value(0),
        "US-EXAMPLE"
    );
    assert_eq!(
        string_column(&legal_batch, "evidence_kind").value(0),
        "law_clause"
    );

    let agriculture_harness =
        FixtureFlightHarness::build_with_manifest(agriculture_pack_fixture_manifest()).await;
    let mut agriculture_search = ExternalKnowledgeSearchRequest::new("dry seven-day weather");
    agriculture_search.sources = vec!["agriculture-market-demo".to_string()];
    agriculture_search.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::Official);
    agriculture_search.limit = 10;
    let agriculture_batch = agriculture_harness
        .handle_descriptor(NexusFlightCommand::Search(agriculture_search))
        .await;

    assert_eq!(agriculture_batch.num_rows(), 1);
    assert_eq!(
        string_column(&agriculture_batch, "title").value(0),
        "Midwest Corn Weekly Market Signal"
    );
    assert_eq!(
        string_column(&agriculture_batch, "source_kind").value(0),
        "GovernmentDatabase"
    );
    assert_eq!(
        string_column(&agriculture_batch, "authority_level").value(0),
        "Official"
    );
    assert_eq!(
        string_column(&agriculture_batch, "evidence_kind").value(0),
        "market_signal"
    );

    let agriculture_open = agriculture_harness
        .handle_descriptor(NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
            source_id: "agriculture-market-demo".to_string(),
            external_id: "agriculture/market/corn-midwest-weekly".to_string(),
            include_sections: true,
            include_provenance: true,
        }))
        .await;
    assert!(
        string_column(&agriculture_open, "metadata_json")
            .value(0)
            .contains("\"crop\":\"corn\"")
    );
    assert!(
        string_column(&agriculture_open, "metadata_json")
            .value(0)
            .contains("\"price_date\":\"2026-04-21\"")
    );

    let legal_status = legal_harness
        .handle_descriptor(NexusFlightCommand::Status(
            NexusFlightStatusRequest::all_sources(),
        ))
        .await;
    assert_eq!(
        string_values(&legal_status, "source_id"),
        vec!["legal-compliance-demo".to_string()]
    );

    legal_harness.cleanup();
    agriculture_harness.cleanup();
}

struct FixtureFlightHarness {
    provider: NexusFlightBatchProvider<FixtureFlightHandler>,
    artifact_store: LocalFileArtifactStore,
    artifact_root: PathBuf,
}

impl FixtureFlightHarness {
    async fn build() -> Self {
        Self::build_with_manifest(source_pack_fixture_manifest()).await
    }

    async fn build_with_manifest(manifest: PathBuf) -> Self {
        let artifact_root = artifact_dir("fixture_flight_harness_artifacts");
        cleanup_dir(&artifact_root);

        let pack = SourcePack::from_path(manifest).unwrap();
        let registry = InMemoryNexusRegistry::new();
        let artifact_store = LocalFileArtifactStore::open(&artifact_root).unwrap();
        let runtime = NexusSyncRuntime::new(registry.clone());
        let normalizer = PlainTextNormalizer;
        let mut documents = Vec::new();

        for record in pack.source_records() {
            registry.upsert_source(record).await.unwrap();
        }

        for connector in pack.connectors() {
            let source = pack.source(connector.source_id()).unwrap();
            let discovered = runtime.discover_once(connector).await.unwrap();
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
                    .await
                    .unwrap();
                documents.push(outcome.ingest.document);
            }
        }

        assert!(!documents.is_empty());
        let provider = NexusFlightBatchProvider::new(FixtureFlightHandler {
            documents,
            registry,
        });

        Self {
            provider,
            artifact_store,
            artifact_root,
        }
    }

    async fn handle_descriptor(&self, command: NexusFlightCommand) -> RecordBatch {
        let descriptor = command.to_descriptor().unwrap();
        self.provider.handle_descriptor(&descriptor).await.unwrap()
    }

    fn cleanup(self) {
        cleanup_dir(&self.artifact_root);
    }
}

#[derive(Clone)]
struct FixtureFlightHandler {
    documents: Vec<ExternalKnowledgeDocument>,
    registry: InMemoryNexusRegistry,
}

#[async_trait]
impl NexusFlightCommandHandler for FixtureFlightHandler {
    async fn search(
        &self,
        request: ExternalKnowledgeSearchRequest,
    ) -> Result<Vec<FlightSearchResultRow>, NexusFlightHandlerError> {
        let response = search_documents(&self.documents, request);
        search_rows_from_response(&response).map_err(handler_error)
    }

    async fn open(
        &self,
        request: ExternalKnowledgeOpenRequest,
    ) -> Result<Vec<FlightOpenDocumentRow>, NexusFlightHandlerError> {
        let document = self
            .documents
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
            })?;
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

        let response = search_documents(&self.documents, search);
        let evidence_records = response
            .records
            .iter()
            .map(|record| record.provenance.primary.clone())
            .collect::<Vec<_>>();
        let insufficient_authority = response.records.is_empty();
        let boundary = EvidenceBoundary {
            records: evidence_records,
            minimum_authority,
            insufficient_authority,
            stale_evidence: false,
            conflict_detected: false,
        };

        Ok(vec![FlightCompareResultRow {
            claim: request.claim,
            verdict: if insufficient_authority {
                "insufficient_authority".to_string()
            } else {
                "evidence_available".to_string()
            },
            conflict_detected: boundary.conflict_detected,
            insufficient_authority: boundary.insufficient_authority,
            stale_evidence: boundary.stale_evidence,
            provenance_json: Some(serde_json::to_string(&boundary).map_err(handler_error)?),
        }])
    }
}

impl FixtureFlightHandler {
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
}

fn search_documents(
    documents: &[ExternalKnowledgeDocument],
    request: ExternalKnowledgeSearchRequest,
) -> ExternalKnowledgeSearchResponse {
    let mut records = documents
        .iter()
        .filter(|document| source_filter_allows(document, &request.sources))
        .filter(|document| trust_policy_allows(document, &request.trust_policy))
        .filter(|document| freshness_filter_allows(document, request.freshness_days))
        .filter(|document| document_matches_query(document, &request.query))
        .map(|document| evidence_record_from_document(document, &request.query))
        .collect::<Vec<_>>();

    records.sort_by(|left, right| {
        right
            .provenance
            .primary
            .authority_level
            .cmp(&left.provenance.primary.authority_level)
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.external_id.cmp(&right.external_id))
    });
    records.truncate(request.limit);

    ExternalKnowledgeSearchResponse {
        query: request.query,
        records,
        generated_at: Utc::now(),
    }
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

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a StringArray {
    let index = batch.schema().index_of(name).unwrap();
    batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap()
}

fn string_values(batch: &RecordBatch, name: &str) -> Vec<String> {
    let column = string_column(batch, name);
    (0..column.len())
        .map(|row| column.value(row).to_string())
        .collect()
}

fn bool_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a BooleanArray {
    let index = batch.schema().index_of(name).unwrap();
    batch
        .column(index)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap()
}

fn source_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../wendao-nexus-connectors/tests/fixtures/source_packs/medical_demo_pack.toml")
}

fn customer_private_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/customer_private_knowledge_pack.toml",
    )
}

fn legal_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../wendao-nexus-connectors/tests/fixtures/source_packs/legal_compliance_pack.toml")
}

fn agriculture_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../wendao-nexus-connectors/tests/fixtures/source_packs/agriculture_market_pack.toml")
}

fn artifact_dir(test_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wendao-nexus-{test_name}-{}", uuid::Uuid::new_v4()))
}

fn cleanup_dir(path: &PathBuf) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}
