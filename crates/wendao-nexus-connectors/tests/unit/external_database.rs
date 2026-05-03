use wendao_nexus_connectors::{
    ExternalDatabaseAccessMode, ExternalDatabaseAuthMode, ExternalDatabaseConfig,
    ExternalDatabaseConnector,
};
use wendao_nexus_core::{
    AuthorityLevel, KnowledgeSourceConnector, KnowledgeSourceKind, SourceItemRef,
};

#[test]
fn external_database_connector_declares_identity_without_runtime_backend_capabilities() {
    let mut config = ExternalDatabaseConfig::new(
        "customer-crm",
        KnowledgeSourceKind::ApiFeed,
        "https://api.customer.example/v1/knowledge",
    );
    config.display_name = "Customer CRM".to_string();
    config.auth_mode = ExternalDatabaseAuthMode::ApiKeyHeader {
        header_name: "x-api-key".to_string(),
    };
    config.access_mode = ExternalDatabaseAccessMode::MirrorAndFederated;
    config.supports_revisions = true;
    config.license_metadata = true;
    config.access_control = true;

    let connector = ExternalDatabaseConnector::try_new(config).unwrap();
    let capabilities = connector.capabilities();

    assert_eq!(connector.source_id(), "customer-crm");
    assert_eq!(connector.source_kind(), KnowledgeSourceKind::ApiFeed);
    assert_eq!(connector.endpoint().scheme(), "https");
    assert!(!capabilities.discover);
    assert!(!capabilities.fetch);
    assert!(!capabilities.delta);
    assert!(!capabilities.live_query);
    assert!(!capabilities.local_mirror);
    assert!(capabilities.revisions);
    assert!(capabilities.structured_metadata);
    assert!(capabilities.license_metadata);
    assert!(capabilities.access_control);
}

#[test]
fn external_database_connector_can_model_mirror_only_sources() {
    let mut config = ExternalDatabaseConfig::new(
        "regulatory-db",
        KnowledgeSourceKind::GovernmentDatabase,
        "https://regulator.example/export",
    );
    config.access_mode = ExternalDatabaseAccessMode::MirrorOnly;
    config.supports_delta = false;

    let connector = ExternalDatabaseConnector::try_new(config).unwrap();
    let capabilities = connector.capabilities();

    assert!(!capabilities.local_mirror);
    assert!(!capabilities.live_query);
    assert!(!capabilities.delta);
    assert_eq!(
        connector.config().default_media_type.as_str(),
        "application/json"
    );
}

#[test]
fn external_database_source_record_keeps_access_mode_as_metadata_not_execution() {
    let mut config = ExternalDatabaseConfig::new(
        "customer-crm",
        KnowledgeSourceKind::ApiFeed,
        "https://api.customer.example/v1/knowledge",
    );
    config.display_name = "Customer CRM".to_string();
    config.access_mode = ExternalDatabaseAccessMode::FederatedLive;
    config.auth_mode = ExternalDatabaseAuthMode::BearerToken;

    let record = config.source_record(AuthorityLevel::CustomerInternal);

    assert_eq!(record.source_id, "customer-crm");
    assert_eq!(record.display_name, "Customer CRM");
    assert_eq!(record.authority_level, AuthorityLevel::CustomerInternal);
    assert_eq!(record.sync_policy.as_deref(), Some("federated_live"));
    assert_eq!(record.auth_mode.as_deref(), Some("bearer_token"));
    assert!(!record.capabilities.discover);
    assert!(!record.capabilities.fetch);
    assert!(!record.capabilities.delta);
    assert!(!record.capabilities.live_query);
    assert!(!record.capabilities.local_mirror);
}

#[test]
fn external_database_connector_rejects_invalid_endpoint() {
    let config = ExternalDatabaseConfig::new("bad-db", KnowledgeSourceKind::ApiFeed, "not a url");

    let error = ExternalDatabaseConnector::try_new(config).unwrap_err();

    assert!(error.to_string().contains("endpoint_uri"));
}

#[tokio::test]
async fn external_database_connector_leaves_backend_execution_unsupported() {
    let connector = ExternalDatabaseConnector::try_new(ExternalDatabaseConfig::new(
        "customer-db",
        KnowledgeSourceKind::ApiFeed,
        "https://api.customer.example/v1",
    ))
    .unwrap();

    let error = connector
        .fetch(SourceItemRef::new("customer-db", "record-1"))
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("external database fetch backend")
    );
}
