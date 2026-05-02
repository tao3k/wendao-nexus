use wendao_nexus_core::{SourceCapabilities, SourceCheckpoint, SourceCursor, SourceItemRef};

#[test]
fn mirror_fetch_capability_preserves_structured_metadata_boundary() {
    let capabilities = SourceCapabilities::mirror_fetch();

    assert!(capabilities.discover);
    assert!(capabilities.fetch);
    assert!(capabilities.local_mirror);
    assert!(capabilities.structured_metadata);
    assert!(capabilities.license_metadata);
    assert!(!capabilities.access_control);
}

#[test]
fn checkpoint_and_item_refs_keep_connector_specific_state_opaque() {
    let mut checkpoint = SourceCheckpoint::new("pubmed");
    checkpoint.cursor = Some(SourceCursor::new("retstart=100"));

    let item = SourceItemRef::new("pubmed", "PMID:123");

    assert_eq!(checkpoint.source_id, "pubmed");
    assert_eq!(checkpoint.cursor.unwrap().value, "retstart=100");
    assert_eq!(item.external_id, "PMID:123");
}
