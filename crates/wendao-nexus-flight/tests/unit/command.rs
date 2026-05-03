use arrow_flight::FlightDescriptor;
use wendao_nexus_core::{
    AuthorityLevel, EvidenceConflictMode, ExternalKnowledgeCompareRequest,
    ExternalKnowledgeOpenRequest, ExternalKnowledgeRefreshRequest, ExternalKnowledgeSearchRequest,
    TrustPolicy,
};
use wendao_nexus_flight::{
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, NexusFlightCommand, NexusFlightCommandError,
    NexusFlightStatusRequest, NexusFlightSyncRequest,
};

#[test]
fn search_command_round_trips_through_flight_descriptor() {
    let mut request = ExternalKnowledgeSearchRequest::new("GLP-1 cardiovascular risk");
    request.sources = vec!["pubmed".to_string(), "fda".to_string()];
    request.trust_policy = TrustPolicy::authority_at_least(AuthorityLevel::PeerReviewed);
    request.freshness_days = Some(365);

    let command = NexusFlightCommand::Search(request.clone());
    let descriptor = command.to_descriptor().unwrap();
    let decoded = NexusFlightCommand::from_descriptor(&descriptor).unwrap();

    assert_eq!(decoded, NexusFlightCommand::Search(request));
}

#[test]
fn encoded_command_uses_canonical_route_string() {
    let command =
        NexusFlightCommand::Search(ExternalKnowledgeSearchRequest::new("authority boundary"));
    let json = String::from_utf8(command.encode_json().unwrap()).unwrap();

    assert!(json.contains(EXTERNAL_KNOWLEDGE_SEARCH_ROUTE));
    assert!(!json.contains("\"Search\""));
}

#[test]
fn status_command_json_matches_wire_envelope_snapshot() {
    let command = NexusFlightCommand::Status(NexusFlightStatusRequest::all_sources());
    let json = String::from_utf8(command.encode_json().unwrap()).unwrap();

    assert_eq!(
        json,
        r#"{"route":"/knowledge/external/status","payload":{"sources":[]}}"#
    );
}

#[test]
fn open_command_round_trips_through_json() {
    let command = NexusFlightCommand::Open(ExternalKnowledgeOpenRequest {
        source_id: "wikipedia".to_string(),
        external_id: "page:Rust".to_string(),
        include_sections: true,
        include_provenance: true,
    });

    let decoded = NexusFlightCommand::decode_json(&command.encode_json().unwrap()).unwrap();

    assert_eq!(decoded, command);
}

#[test]
fn sync_command_can_be_built_from_refresh_request() {
    let refresh = ExternalKnowledgeRefreshRequest {
        source_id: "pubmed".to_string(),
        external_id: "PMID:123".to_string(),
        force: true,
    };
    let command = NexusFlightCommand::Sync(NexusFlightSyncRequest::from(refresh));

    let decoded = NexusFlightCommand::decode_json(&command.encode_json().unwrap()).unwrap();

    assert_eq!(decoded, command);
}

#[test]
fn status_command_supports_all_sources_request() {
    let command = NexusFlightCommand::Status(NexusFlightStatusRequest::all_sources());

    let decoded = NexusFlightCommand::from_descriptor(&command.to_descriptor().unwrap()).unwrap();

    assert_eq!(decoded, command);
}

#[test]
fn compare_command_round_trips_through_descriptor() {
    let command = NexusFlightCommand::Compare(ExternalKnowledgeCompareRequest {
        claim: "A treatment reduces risk".to_string(),
        sources: vec!["pubmed".to_string()],
        mode: EvidenceConflictMode::EvidenceConflictCheck,
        trust_policy: TrustPolicy::authority_at_least(AuthorityLevel::PeerReviewed),
    });

    let decoded = NexusFlightCommand::from_descriptor(&command.to_descriptor().unwrap()).unwrap();

    assert_eq!(decoded, command);
}

#[test]
fn path_descriptor_is_rejected_for_command_decode() {
    let descriptor = FlightDescriptor::new_path(vec!["knowledge".to_string()]);
    let error = NexusFlightCommand::from_descriptor(&descriptor).unwrap_err();

    assert!(matches!(
        error,
        NexusFlightCommandError::UnsupportedDescriptorType(_)
    ));
}

#[test]
fn unsupported_route_is_reported() {
    let bytes = br#"{"route":"/knowledge/external/unknown","payload":{}}"#;
    let error = NexusFlightCommand::decode_json(bytes).unwrap_err();

    assert!(matches!(
        error,
        NexusFlightCommandError::UnsupportedRoute(_)
    ));
}

#[test]
fn malformed_json_is_reported() {
    let error = NexusFlightCommand::decode_json(br#"{"route":"#).unwrap_err();

    assert!(matches!(error, NexusFlightCommandError::Json(_)));
}
