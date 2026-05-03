use std::path::PathBuf;

pub(super) fn fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/source_packs/medical_baseline/source_pack.toml")
}

pub(super) fn json_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/source_packs/medical_demo_pack.json")
}

pub(super) fn disabled_source_manifest() -> PathBuf {
    flat_source_pack_fixture("disabled_source.toml")
}

pub(super) fn all_disabled_source_manifest() -> PathBuf {
    flat_source_pack_fixture("all_disabled.toml")
}

pub(super) fn duplicate_source_id_manifest() -> PathBuf {
    flat_source_pack_fixture("duplicate_source_id.toml")
}

pub(super) fn whitespace_source_id_manifest() -> PathBuf {
    flat_source_pack_fixture("whitespace_source_id.toml")
}

pub(super) fn whitespace_fixture_path_manifest() -> PathBuf {
    flat_source_pack_fixture("whitespace_fixture_path.toml")
}

pub(super) fn unsupported_schema_version_manifest() -> PathBuf {
    flat_source_pack_fixture("unsupported_schema_version.toml")
}

pub(super) fn whitespace_producer_manifest() -> PathBuf {
    flat_source_pack_fixture("whitespace_producer.toml")
}

pub(super) fn whitespace_canonical_uri_prefix_manifest() -> PathBuf {
    flat_source_pack_fixture("whitespace_canonical_uri_prefix.toml")
}

pub(super) fn empty_display_name_manifest() -> PathBuf {
    flat_source_pack_fixture("empty_display_name.toml")
}

pub(super) fn whitespace_license_manifest() -> PathBuf {
    flat_source_pack_fixture("whitespace_license.toml")
}

pub(super) fn whitespace_source_display_name_manifest() -> PathBuf {
    flat_source_pack_fixture("whitespace_source_display_name.toml")
}

pub(super) fn empty_source_license_manifest() -> PathBuf {
    flat_source_pack_fixture("empty_source_license.toml")
}

fn flat_source_pack_fixture(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/source_packs/{file_name}"))
}
