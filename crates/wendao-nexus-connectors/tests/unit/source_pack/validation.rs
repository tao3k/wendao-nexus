use wendao_nexus_connectors::{SourcePack, SourcePackManifest};

use super::fixtures::{
    all_disabled_source_manifest, duplicate_source_id_manifest, empty_display_name_manifest,
    empty_source_license_manifest, fixture_manifest, unsupported_schema_version_manifest,
    whitespace_canonical_uri_prefix_manifest, whitespace_fixture_path_manifest,
    whitespace_license_manifest, whitespace_producer_manifest,
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
