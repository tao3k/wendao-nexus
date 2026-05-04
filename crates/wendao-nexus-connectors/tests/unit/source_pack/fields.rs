use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use wendao_nexus_connectors::SourcePackManifest;
use wendao_nexus_core::{CanonicalEvidenceSlot, EvidenceFieldType, FieldKey};

#[test]
fn source_pack_manifest_accepts_profile_field_descriptors() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("documents.jsonl"), "").unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "profile-field-pack"
version = "2026.05-fixture"
schema_version = 1
domain = "legal"

[[sources]]
source_id = "legal-source"
kind = "LegalCorpus"
fixture_path = "documents.jsonl"
license = "Fixture License"

[[source_profiles]]
source_id = "legal-source"
domain = "legal"
authority_level = "Official"

[[source_profiles.fields]]
key = { namespace = "legal", name = "article" }
value_type = "string"
required = true
aliases = ["section"]
canonical_slot = "identifier"
"#,
    )
    .unwrap();

    let manifest = SourcePackManifest::from_path(root.join("source_pack.toml")).unwrap();
    cleanup_dir(&root);

    let field = &manifest.source_profiles[0].fields[0];
    assert_eq!(field.key, FieldKey::new("legal", "article"));
    assert_eq!(field.value_type, EvidenceFieldType::String);
    assert!(field.required);
    assert_eq!(field.aliases, vec!["section"]);
    assert_eq!(
        field.canonical_slot,
        Some(CanonicalEvidenceSlot::Identifier)
    );
}

#[test]
fn source_pack_rejects_duplicate_profile_field_descriptors() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("documents.jsonl"), "").unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "duplicate-profile-field-pack"
version = "2026.05-fixture"
schema_version = 1
domain = "legal"

[[sources]]
source_id = "legal-source"
kind = "LegalCorpus"
fixture_path = "documents.jsonl"
license = "Fixture License"

[[source_profiles]]
source_id = "legal-source"
domain = "legal"
authority_level = "Official"

[[source_profiles.fields]]
key = { namespace = "legal", name = "article" }
value_type = "string"
required = true

[[source_profiles.fields]]
key = { namespace = "legal", name = "article" }
value_type = "string"
required = false
"#,
    )
    .unwrap();

    let error = SourcePackManifest::from_path(root.join("source_pack.toml"))
        .unwrap_err()
        .to_string();
    cleanup_dir(&root);

    assert!(error.contains("duplicate field `legal.article`"), "{error}");
}

#[test]
fn source_pack_rejects_invalid_profile_field_labels() {
    let root = temporary_export_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("documents.jsonl"), "").unwrap();
    std::fs::write(
        root.join("source_pack.toml"),
        r#"[source_pack]
id = "invalid-profile-field-pack"
version = "2026.05-fixture"
schema_version = 1
domain = "agriculture"

[[sources]]
source_id = "agri-source"
kind = "GovernmentDatabase"
fixture_path = "documents.jsonl"
license = "Fixture License"

[[source_profiles]]
source_id = "agri-source"
domain = "agriculture"
authority_level = "Official"

[[source_profiles.fields]]
key = { namespace = " agriculture", name = "crop" }
value_type = "string"
required = true
aliases = ["crop", " crop_name"]
"#,
    )
    .unwrap();

    let error = SourcePackManifest::from_path(root.join("source_pack.toml"))
        .unwrap_err()
        .to_string();
    cleanup_dir(&root);

    assert!(error.contains("field namespace ` agriculture`"), "{error}");
    assert!(
        error.contains("must not contain leading or trailing whitespace"),
        "{error}"
    );
}

fn temporary_export_root() -> std::path::PathBuf {
    static NEXT_TEMP_ROOT_ID: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_id = NEXT_TEMP_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "wendao-nexus-source-pack-fields-{}-{nanos}-{unique_id}",
        std::process::id()
    ))
}

fn cleanup_dir(path: &std::path::Path) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}
