use std::path::PathBuf;

use wendao_nexus::NexusFixtureHarness;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceConflictMode, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeDocument, ExternalKnowledgeOpenRequest, ExternalKnowledgeSearchRequest,
    TrustPolicy,
};
use wendao_nexus_flight::{
    EXTERNAL_KNOWLEDGE_COMPARE_ROUTE, EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, EXTERNAL_KNOWLEDGE_STATUS_ROUTE, NexusFlightCommand,
    NexusFlightCommandError, NexusFlightProviderError, NexusFlightStatusRequest,
    NexusFlightSyncRequest,
};
use wendao_nexus_runtime::{ArtifactKind, ArtifactStore};

use crate::fixture_flight_support::{
    agriculture_pack_fixture_manifest, artifact_dir, customer_private_pack_fixture_manifest,
    legal_pack_fixture_manifest, source_pack_fixture_manifest,
};

use super::support::{assert_batch_route, bool_column, string_column, string_values};

#[tokio::test]
async fn serverless_fixture_harness_conforms_for_vertical_packs() {
    for case in harness_cases() {
        let artifact_root = artifact_dir(case.source_id);
        let harness = NexusFixtureHarness::load_source_pack(case.manifest(), &artifact_root)
            .await
            .unwrap();
        assert_eq!(
            harness.ingest_report().ingested_documents,
            harness.ingest_report().normalized_artifacts
        );

        let mut search = ExternalKnowledgeSearchRequest::new(case.query);
        search.sources = vec![case.source_id.to_string()];
        search.trust_policy = TrustPolicy::authority_at_least(case.search_authority);
        search.limit = 10;
        let search_batch = harness
            .handle_encoded_command(NexusFlightCommand::Search(search).encode_json().unwrap())
            .await
            .unwrap();
        assert_batch_route(&search_batch, EXTERNAL_KNOWLEDGE_SEARCH_ROUTE);
        assert!(
            string_values(&search_batch, "evidence_kind")
                .iter()
                .any(|value| value == case.evidence_kind),
            "{} search did not return {}",
            case.source_id,
            case.evidence_kind
        );

        let open_batch = harness
            .handle_encoded_command(
                NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
                    source_id: case.source_id.to_string(),
                    external_id: case.external_id.to_string(),
                    include_sections: true,
                    include_provenance: true,
                })
                .encode_json()
                .unwrap(),
            )
            .await
            .unwrap();
        assert_batch_route(&open_batch, EXTERNAL_KNOWLEDGE_OPEN_ROUTE);
        assert!(
            string_column(&open_batch, "metadata_json")
                .value(0)
                .contains(case.metadata_probe)
        );

        let status_batch = harness
            .handle_encoded_command(
                NexusFlightCommand::Status(NexusFlightStatusRequest::all_sources())
                    .encode_json()
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_batch_route(&status_batch, EXTERNAL_KNOWLEDGE_STATUS_ROUTE);
        assert!(
            string_values(&status_batch, "source_id")
                .iter()
                .any(|source_id| source_id == case.source_id)
        );

        let compare_batch = harness
            .handle_encoded_command(
                NexusFlightCommand::Compare(ExternalKnowledgeCompareRequest {
                    claim: case.compare_claim.to_string(),
                    sources: vec![case.source_id.to_string()],
                    mode: EvidenceConflictMode::EvidenceConflictCheck,
                    trust_policy: TrustPolicy::authority_at_least(case.compare_authority),
                })
                .encode_json()
                .unwrap(),
            )
            .await
            .unwrap();
        assert_batch_route(&compare_batch, EXTERNAL_KNOWLEDGE_COMPARE_ROUTE);
        assert_eq!(
            string_column(&compare_batch, "verdict").value(0),
            case.compare_verdict
        );
        assert_eq!(
            bool_column(&compare_batch, "insufficient_authority").value(0),
            case.compare_verdict == "insufficient_authority"
        );

        assert_normalized_artifact_replays(&harness, case.source_id, case.external_id).await;
        cleanup_dir(&artifact_root);
    }
}

#[tokio::test]
async fn fixture_provider_reports_protocol_and_handler_errors_conformantly() {
    let artifact_root = artifact_dir("fixture_provider_error_contract");
    let harness =
        NexusFixtureHarness::load_source_pack(source_pack_fixture_manifest(), &artifact_root)
            .await
            .unwrap();

    let schema_error = harness
        .handle_encoded_command(
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
        .handle_encoded_command(
            br#"{"schema_version":1,"route":"/knowledge/external/unknown","payload":{}}"#.to_vec(),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        route_error,
        NexusFlightProviderError::Command(NexusFlightCommandError::UnsupportedRoute(_))
    ));

    let handler_error = harness
        .handle_command(NexusFlightCommand::Sync(NexusFlightSyncRequest {
            source_id: "demo-pubmed".to_string(),
            external_id: Some("medical/pubmed-demo-1".to_string()),
            force: false,
        }))
        .await
        .unwrap_err();
    assert!(matches!(
        handler_error,
        NexusFlightProviderError::Handler(_)
    ));

    cleanup_dir(&artifact_root);
}

async fn assert_normalized_artifact_replays(
    harness: &NexusFixtureHarness,
    source_id: &str,
    external_id: &str,
) {
    let artifacts = harness
        .artifact_store()
        .list_artifacts(source_id, external_id)
        .await
        .unwrap();
    let normalized = artifacts
        .iter()
        .find(|artifact| artifact.kind == ArtifactKind::NormalizedDocument)
        .unwrap();
    let payload = harness
        .artifact_store()
        .get_artifact(
            source_id,
            external_id,
            ArtifactKind::NormalizedDocument,
            &normalized.content_hash,
        )
        .await
        .unwrap()
        .unwrap();
    let document: ExternalKnowledgeDocument = serde_json::from_slice(&payload.bytes).unwrap();

    assert_eq!(document.source_id, source_id);
    assert_eq!(document.external_id, external_id);
    assert_eq!(document.content_hash, normalized.content_hash);
    assert_eq!(payload.descriptor.kind, ArtifactKind::NormalizedDocument);
}

fn cleanup_dir(path: &std::path::Path) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}

struct HarnessCase {
    manifest: fn() -> PathBuf,
    source_id: &'static str,
    external_id: &'static str,
    query: &'static str,
    evidence_kind: &'static str,
    metadata_probe: &'static str,
    search_authority: AuthorityLevel,
    compare_claim: &'static str,
    compare_authority: AuthorityLevel,
    compare_verdict: &'static str,
}

impl HarnessCase {
    fn manifest(&self) -> PathBuf {
        (self.manifest)()
    }
}

fn harness_cases() -> [HarnessCase; 4] {
    [
        HarnessCase {
            manifest: source_pack_fixture_manifest,
            source_id: "demo-pubmed",
            external_id: "medical/pubmed-demo-1",
            query: "GLP-1 cardiovascular",
            evidence_kind: "trial_result",
            metadata_probe: "10.1000/demo1",
            search_authority: AuthorityLevel::PeerReviewed,
            compare_claim: "GLP-1 cardiovascular",
            compare_authority: AuthorityLevel::PeerReviewed,
            compare_verdict: "evidence_available",
        },
        HarnessCase {
            manifest: customer_private_pack_fixture_manifest,
            source_id: "customer-sop-demo",
            external_id: "customer/sop/clinical-trial-intake",
            query: "QA reviewer approval",
            evidence_kind: "customer_internal_note",
            metadata_probe: "tenant_id",
            search_authority: AuthorityLevel::CustomerInternal,
            compare_claim: "QA reviewer approval",
            compare_authority: AuthorityLevel::Official,
            compare_verdict: "insufficient_authority",
        },
        HarnessCase {
            manifest: legal_pack_fixture_manifest,
            source_id: "legal-compliance-demo",
            external_id: "legal/privacy/data-retention-clause",
            query: "retain audit evidence",
            evidence_kind: "law_clause",
            metadata_probe: "Article 12",
            search_authority: AuthorityLevel::Official,
            compare_claim: "retain audit evidence",
            compare_authority: AuthorityLevel::Official,
            compare_verdict: "evidence_available",
        },
        HarnessCase {
            manifest: agriculture_pack_fixture_manifest,
            source_id: "agriculture-market-demo",
            external_id: "agriculture/market/corn-midwest-weekly",
            query: "dry seven-day weather",
            evidence_kind: "market_signal",
            metadata_probe: "corn",
            search_authority: AuthorityLevel::Official,
            compare_claim: "dry seven-day weather",
            compare_authority: AuthorityLevel::Official,
            compare_verdict: "evidence_available",
        },
    ]
}
