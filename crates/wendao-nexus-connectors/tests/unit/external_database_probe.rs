use wendao_nexus_connectors::{
    ExternalDatabaseAuthMode, ExternalDatabaseConfig, ExternalDatabaseProbeConfig,
    ExternalDatabaseProbeConnector,
};
use wendao_nexus_core::{KnowledgeSourceConnector, KnowledgeSourceKind, SourceItemRef};

#[test]
fn external_database_probe_capabilities_are_live_probe_only() {
    let connector = ExternalDatabaseProbeConnector::try_new(ExternalDatabaseProbeConfig::new(
        ExternalDatabaseConfig::new(
            "probe-json",
            KnowledgeSourceKind::ApiFeed,
            "https://example.com/probe.json",
        ),
    ))
    .unwrap();
    let capabilities = connector.capabilities();

    assert!(!capabilities.discover);
    assert!(capabilities.fetch);
    assert!(!capabilities.delta);
    assert!(capabilities.live_query);
    assert!(!capabilities.local_mirror);
}

#[test]
fn external_database_probe_rejects_invalid_probe_config() {
    let mut zero_timeout = ExternalDatabaseProbeConfig::new(ExternalDatabaseConfig::new(
        "probe-json",
        KnowledgeSourceKind::ApiFeed,
        "https://example.com/probe.json",
    ));
    zero_timeout.timeout_millis = 0;
    assert!(
        ExternalDatabaseProbeConnector::try_new(zero_timeout)
            .unwrap_err()
            .to_string()
            .contains("timeout_millis must be greater than zero")
    );

    let mut zero_bytes = ExternalDatabaseProbeConfig::new(ExternalDatabaseConfig::new(
        "probe-json",
        KnowledgeSourceKind::ApiFeed,
        "https://example.com/probe.json",
    ));
    zero_bytes.max_bytes = 0;
    assert!(
        ExternalDatabaseProbeConnector::try_new(zero_bytes)
            .unwrap_err()
            .to_string()
            .contains("max_bytes must be greater than zero")
    );

    let mut empty_agent = ExternalDatabaseProbeConfig::new(ExternalDatabaseConfig::new(
        "probe-json",
        KnowledgeSourceKind::ApiFeed,
        "https://example.com/probe.json",
    ));
    empty_agent.user_agent = " ".to_string();
    assert!(
        ExternalDatabaseProbeConnector::try_new(empty_agent)
            .unwrap_err()
            .to_string()
            .contains("user_agent must not be empty")
    );

    let invalid_endpoint = ExternalDatabaseProbeConfig::new(ExternalDatabaseConfig::new(
        "probe-json",
        KnowledgeSourceKind::ApiFeed,
        "not a url",
    ));
    assert!(
        ExternalDatabaseProbeConnector::try_new(invalid_endpoint)
            .unwrap_err()
            .to_string()
            .contains("endpoint_uri is invalid")
    );

    let mut authenticated = ExternalDatabaseProbeConfig::new(ExternalDatabaseConfig::new(
        "probe-json",
        KnowledgeSourceKind::ApiFeed,
        "https://example.com/probe.json",
    ));
    authenticated.database.auth_mode = ExternalDatabaseAuthMode::BearerToken;
    assert!(
        ExternalDatabaseProbeConnector::try_new(authenticated)
            .unwrap_err()
            .to_string()
            .contains("only supports unauthenticated public GET probes")
    );
}

#[tokio::test]
async fn external_database_probe_fetches_local_json_endpoint_without_external_network() {
    let (endpoint, server) =
        serve_probe_response("application/json; charset=utf-8", r#"{"fixture":true}"#).await;
    let connector = ExternalDatabaseProbeConnector::try_new(ExternalDatabaseProbeConfig::new(
        ExternalDatabaseConfig::new("probe-json", KnowledgeSourceKind::ApiFeed, endpoint.clone()),
    ))
    .unwrap();

    let document = connector
        .fetch(SourceItemRef::new("probe-json", "loopback-probe"))
        .await
        .unwrap();
    server.await.unwrap();

    assert_eq!(document.source_id, "probe-json");
    assert_eq!(document.external_id, "loopback-probe");
    assert_eq!(document.canonical_uri, endpoint);
    assert_eq!(document.media_type, "application/json");
    assert_eq!(document.payload, br#"{"fixture":true}"#);
    assert_eq!(
        document.metadata.get("live_probe").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        document.metadata.get("http_status").map(String::as_str),
        Some("200")
    );
}

#[tokio::test]
async fn external_database_probe_rejects_non_json_loopback_response() {
    let (endpoint, server) = serve_probe_response("text/plain", "not-json").await;
    let connector = ExternalDatabaseProbeConnector::try_new(ExternalDatabaseProbeConfig::new(
        ExternalDatabaseConfig::new("probe-json", KnowledgeSourceKind::ApiFeed, endpoint),
    ))
    .unwrap();

    let error = connector
        .fetch(SourceItemRef::new("probe-json", "loopback-probe"))
        .await
        .unwrap_err()
        .to_string();
    server.await.unwrap();

    assert!(error.contains("expected JSON media type"));
}

#[tokio::test]
async fn external_database_probe_rejects_oversized_loopback_response() {
    let (endpoint, server) = serve_probe_response("application/json", r#"{"too":"large"}"#).await;
    let mut config = ExternalDatabaseProbeConfig::new(ExternalDatabaseConfig::new(
        "probe-json",
        KnowledgeSourceKind::ApiFeed,
        endpoint,
    ));
    config.max_bytes = 4;
    let connector = ExternalDatabaseProbeConnector::try_new(config).unwrap();

    let error = connector
        .fetch(SourceItemRef::new("probe-json", "loopback-probe"))
        .await
        .unwrap_err()
        .to_string();
    server.await.unwrap();

    assert!(error.contains("exceeded max_bytes 4"));
}

#[tokio::test]
async fn external_database_probe_fetch_is_env_gated() {
    if std::env::var("WENDAO_NEXUS_RUN_LIVE_PROBE").ok().as_deref() != Some("1") {
        return;
    }

    let endpoint = std::env::var("WENDAO_NEXUS_LIVE_PROBE_ENDPOINT")
        .unwrap_or_else(|_| "https://httpbin.org/json".to_string());
    let connector = ExternalDatabaseProbeConnector::try_new(ExternalDatabaseProbeConfig::new(
        ExternalDatabaseConfig::new("probe-json", KnowledgeSourceKind::ApiFeed, endpoint),
    ))
    .unwrap();

    let document = connector
        .fetch(SourceItemRef::new("probe-json", "live-probe"))
        .await
        .unwrap();

    assert_eq!(document.source_id, "probe-json");
    assert_eq!(document.external_id, "live-probe");
    assert_eq!(
        document.metadata.get("live_probe").map(String::as_str),
        Some("true")
    );
    assert!(!document.payload.is_empty());
}

async fn serve_probe_response(
    content_type: &'static str,
    body: &'static str,
) -> (String, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/probe.json", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await.unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    (endpoint, handle)
}
