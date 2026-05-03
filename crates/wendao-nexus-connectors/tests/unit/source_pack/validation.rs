use std::time::{SystemTime, UNIX_EPOCH};

use wendao_nexus_connectors::{SourcePack, SourcePackManifest, validate_source_pack_export};
use wendao_nexus_core::SourceDomain;

use super::fixtures::{
    all_disabled_source_manifest, customer_private_pack_root, duplicate_source_id_manifest,
    empty_display_name_manifest, empty_source_license_manifest, fixture_manifest,
    unsupported_schema_version_manifest, whitespace_canonical_uri_prefix_manifest,
    whitespace_fixture_path_manifest, whitespace_license_manifest, whitespace_producer_manifest,
    whitespace_source_display_name_manifest, whitespace_source_id_manifest,
};

#[test]
fn source_pack_rejects_all_disabled_sources_for_connector_loading() {
    let error = SourcePack::from_path(all_disabled_source_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("has no enabled sources"));
}

#[test]
fn source_pack_rejects_unsupported_manifest_extension() {
    let error = SourcePackManifest::from_path(fixture_manifest().with_extension("txt"))
        .unwrap_err()
        .to_string();

    assert!(error.contains("must use .toml or .json"));
}

#[test]
fn source_pack_rejects_duplicate_source_ids() {
    let error = SourcePackManifest::from_path(duplicate_source_id_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("duplicate source_id `duplicate-source`"));
}

#[test]
fn source_pack_rejects_whitespace_padded_source_ids() {
    let error = SourcePackManifest::from_path(whitespace_source_id_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("source_id ` padded-source ` must not contain"));
}

#[test]
fn source_pack_rejects_whitespace_padded_fixture_paths() {
    let error = SourcePackManifest::from_path(whitespace_fixture_path_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("fixture_path must not contain"));
}

#[test]
fn source_pack_rejects_unsupported_schema_version() {
    let error = SourcePackManifest::from_path(unsupported_schema_version_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("schema_version 2 is unsupported"));
    assert!(error.contains("expected 1"));
}

#[test]
fn source_pack_rejects_whitespace_padded_producer() {
    let error = SourcePackManifest::from_path(whitespace_producer_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("producer ` fixture-builder ` must not contain"));
}

#[test]
fn source_pack_rejects_whitespace_padded_canonical_uri_prefix() {
    let error = SourcePackManifest::from_path(whitespace_canonical_uri_prefix_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("canonical_uri_prefix ` https://pubmed.ncbi.nlm.nih.gov/ `"));
    assert!(error.contains("must not contain leading or trailing whitespace"));
}

#[test]
fn source_pack_rejects_empty_display_name() {
    let error = SourcePackManifest::from_path(empty_display_name_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("display_name must not be empty"));
}

#[test]
fn source_pack_rejects_whitespace_padded_license() {
    let error = SourcePackManifest::from_path(whitespace_license_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("license ` Fixture License ` must not contain"));
}

#[test]
fn source_pack_rejects_whitespace_padded_source_display_name() {
    let error = SourcePackManifest::from_path(whitespace_source_display_name_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("source `demo-pubmed` display_name ` Demo PubMed `"));
    assert!(error.contains("must not contain leading or trailing whitespace"));
}

#[test]
fn source_pack_rejects_empty_source_license() {
    let error = SourcePackManifest::from_path(empty_source_license_manifest())
        .unwrap_err()
        .to_string();

    assert!(error.contains("source `demo-pubmed` license must not be empty"));
}

#[test]
fn source_pack_export_validator_accepts_directory_first_pack() {
    let report = validate_source_pack_export(customer_private_pack_root()).unwrap();

    assert_eq!(report.pack_id, "customer-private-sop-pack");
    assert_eq!(report.domain, SourceDomain::CustomerPrivate);
    assert_eq!(report.source_count, 2);
    assert_eq!(report.enabled_source_count, 1);
    assert_eq!(report.fixture_paths.len(), 2);
    assert!(
        report
            .fixture_paths
            .iter()
            .any(|fixture| fixture.source_id == "customer-crm-demo" && !fixture.enabled)
    );
}

#[test]
fn source_pack_export_validator_rejects_missing_fixture_payload() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "missing-fixture-export"
version = "2026.05-fixture"
schema_version = 1
domain = "generic"
license = "Fixture License"

[[sources]]
source_id = "missing-fixture-source"
kind = "WebPage"
fixture_path = "missing.jsonl"
license = "Fixture License"
"#,
    )
    .unwrap();

    let error = validate_source_pack_export(&root).unwrap_err().to_string();
    cleanup_dir(&root);

    assert!(error.contains("fixture file"));
    assert!(error.contains("missing-fixture-source"));
}

#[test]
fn source_pack_rejects_source_profile_for_unknown_source() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("documents.jsonl"), "").unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "unknown-profile-pack"
version = "2026.05-fixture"
schema_version = 1

[[sources]]
source_id = "known-source"
kind = "WebPage"
fixture_path = "documents.jsonl"
license = "Fixture License"

[[source_profiles]]
source_id = "missing-source"
domain = "generic"
authority_level = "Curated"
"#,
    )
    .unwrap();

    let error = SourcePackManifest::from_path(root.join("source_pack.toml"))
        .unwrap_err()
        .to_string();
    cleanup_dir(&root);

    assert!(error.contains("source_profile references unknown source_id `missing-source`"));
}

#[test]
fn source_pack_rejects_duplicate_source_profiles() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("documents.jsonl"), "").unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "duplicate-profile-pack"
version = "2026.05-fixture"
schema_version = 1

[[sources]]
source_id = "profile-source"
kind = "WebPage"
fixture_path = "documents.jsonl"
license = "Fixture License"

[[source_profiles]]
source_id = "profile-source"
domain = "generic"
authority_level = "Curated"

[[source_profiles]]
source_id = "profile-source"
domain = "generic"
authority_level = "Curated"
"#,
    )
    .unwrap();

    let error = SourcePackManifest::from_path(root.join("source_pack.toml"))
        .unwrap_err()
        .to_string();
    cleanup_dir(&root);

    assert!(error.contains("duplicate source_profile for `profile-source`"));
}

#[test]
fn source_pack_rejects_whitespace_padded_source_profile_license_policy() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("documents.jsonl"), "").unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "whitespace-profile-license-pack"
version = "2026.05-fixture"
schema_version = 1
domain = "medical"

[[sources]]
source_id = "profile-source"
kind = "PubMed"
fixture_path = "documents.jsonl"

[[source_profiles]]
source_id = "profile-source"
domain = "medical"
authority_level = "PeerReviewed"
license_policy = "  PubMed metadata  "
"#,
    )
    .unwrap();

    let error = SourcePackManifest::from_path(root.join("source_pack.toml"))
        .unwrap_err()
        .to_string();
    cleanup_dir(&root);

    assert!(error.contains("source_profile license_policy `  PubMed metadata  ` must not contain"));
}

fn temporary_export_root() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "wendao-nexus-source-pack-export-{}-{nanos}",
        std::process::id()
    ))
}

fn cleanup_dir(path: &std::path::Path) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}
