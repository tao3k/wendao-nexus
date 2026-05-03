use arrow_array::{Array, BooleanArray, RecordBatch, StringArray};
use wendao_nexus_core::{
    AuthorityLevel, EvidenceConflictMode, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest, TrustPolicy,
};
use wendao_nexus_flight::{
    EXTERNAL_KNOWLEDGE_STATUS_ROUTE, NexusFlightCommand, NexusFlightCommandError,
    NexusFlightProviderError, NexusFlightStatusRequest, NexusFlightSyncRequest,
};
use wendao_nexus_runtime::{ArtifactKind, ArtifactStore};

use crate::fixture_flight_support::{
    FixtureFlightHarness, agriculture_pack_fixture_manifest,
    customer_private_pack_fixture_manifest, legal_pack_fixture_manifest,
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
        "trial_result"
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
    assert_eq!(
        string_column(&search_batch, "evidence_kind").value(0),
        "customer_internal_note"
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
async fn source_pack_customer_sop_ingest_roundtrip_detects_insufficient_authority() {
    let harness =
        FixtureFlightHarness::build_with_manifest(customer_private_pack_fixture_manifest()).await;

    let compare_batch = harness
        .handle_descriptor(NexusFlightCommand::Compare(
            ExternalKnowledgeCompareRequest {
                claim: "QA reviewer approval".to_string(),
                sources: vec!["customer-sop-demo".to_string()],
                mode: EvidenceConflictMode::EvidenceConflictCheck,
                trust_policy: TrustPolicy::authority_at_least(AuthorityLevel::Official),
            },
        ))
        .await;

    assert_eq!(
        string_column(&compare_batch, "verdict").value(0),
        "insufficient_authority"
    );
    assert!(bool_column(&compare_batch, "insufficient_authority").value(0));
    assert!(!bool_column(&compare_batch, "stale_evidence").value(0));

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

#[tokio::test]
async fn fixture_command_client_surfaces_protocol_and_handler_errors() {
    let harness = FixtureFlightHarness::build().await;

    let schema_error = harness
        .handle_command_json_result(
            br#"{"schema_version":2,"route":"/knowledge/external/status","payload":{"sources":[]}}"#
                .to_vec(),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        schema_error,
        NexusFlightProviderError::Command(NexusFlightCommandError::UnsupportedSchemaVersion(2))
    ));

    let route_error = harness
        .handle_command_json_result(
            br#"{"schema_version":1,"route":"/knowledge/external/unknown","payload":{}}"#.to_vec(),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        route_error,
        NexusFlightProviderError::Command(NexusFlightCommandError::UnsupportedRoute(_))
    ));

    let sync_error = harness
        .handle_command_descriptor_result(NexusFlightCommand::Sync(NexusFlightSyncRequest {
            source_id: "demo-pubmed".to_string(),
            external_id: Some("medical/pubmed-demo-1".to_string()),
            force: false,
        }))
        .await
        .unwrap_err();
    assert!(matches!(sync_error, NexusFlightProviderError::Handler(_)));

    let status_batch = harness
        .handle_command_descriptor_result(NexusFlightCommand::Status(NexusFlightStatusRequest {
            sources: vec!["demo-pubmed".to_string()],
        }))
        .await
        .unwrap();
    assert_eq!(
        status_batch
            .schema()
            .metadata()
            .get("wendao_nexus.route")
            .map(String::as_str),
        Some(EXTERNAL_KNOWLEDGE_STATUS_ROUTE)
    );

    harness.cleanup();
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
