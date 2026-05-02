use wendao_nexus_flight::{search_result_schema, status_schema};

#[test]
fn search_schema_carries_provenance_boundary_columns() {
    let schema = search_result_schema();

    assert!(schema.field_with_name("source_id").is_ok());
    assert!(schema.field_with_name("authority_level").is_ok());
    assert!(schema.field_with_name("provenance_json").is_ok());
}

#[test]
fn status_schema_carries_checkpoint_columns() {
    let schema = status_schema();

    assert!(schema.field_with_name("last_success_at").is_ok());
    assert!(schema.field_with_name("last_seen_revision").is_ok());
    assert!(schema.field_with_name("last_content_hash").is_ok());
}
