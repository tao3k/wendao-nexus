use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use wendao_nexus_core::{
    CanonicalEvidenceSlot, EvidenceAttribute, EvidenceAttributes, EvidenceFieldType, EvidenceValue,
    FieldDescriptor, FieldKey,
};

#[test]
fn field_key_is_namespaced_and_orderable() {
    let key = FieldKey::new("medical", "pmid");

    assert_eq!(key.dotted(), "medical.pmid");
    assert_eq!(
        serde_json::to_string(&key).unwrap(),
        r#"{"namespace":"medical","name":"pmid"}"#
    );
}

#[test]
fn evidence_value_wire_shape_is_typed() {
    let timestamp = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    let values = [
        EvidenceValue::String("PMID:123".to_string()),
        EvidenceValue::Number(0.95),
        EvidenceValue::Bool(true),
        EvidenceValue::Timestamp(timestamp),
        EvidenceValue::StringList(vec!["CRISPR".to_string(), "Cas9".to_string()]),
        EvidenceValue::Json(serde_json::json!({"mesh": ["Genome Editing"]})),
    ];

    assert_eq!(values[0].value_type(), EvidenceFieldType::String);
    assert_eq!(values[1].value_type(), EvidenceFieldType::Number);
    assert_eq!(values[2].value_type(), EvidenceFieldType::Bool);
    assert_eq!(values[3].value_type(), EvidenceFieldType::Timestamp);
    assert_eq!(values[4].value_type(), EvidenceFieldType::StringList);
    assert_eq!(values[5].value_type(), EvidenceFieldType::Json);
    assert!(
        serde_json::to_string(&values)
            .unwrap()
            .contains(r#""type":"timestamp""#)
    );
}

#[test]
fn evidence_attributes_can_wrap_legacy_string_metadata() {
    let metadata = BTreeMap::from([
        ("pmid".to_string(), "37952131".to_string()),
        ("doi".to_string(), "10.1000/example".to_string()),
    ]);
    let attributes = EvidenceAttributes::from_string_map("medical", &metadata);

    assert_eq!(attributes.values.len(), 2);
    assert_eq!(
        attributes
            .get("medical", "pmid")
            .map(|attribute| &attribute.value),
        Some(&EvidenceValue::String("37952131".to_string()))
    );
}

#[test]
fn field_descriptor_declares_profile_projection_without_rust_metadata_keys() {
    let mut descriptor =
        FieldDescriptor::new(FieldKey::new("legal", "article"), EvidenceFieldType::String);
    descriptor.required = true;
    descriptor.aliases = vec!["section".to_string(), "cfr_section".to_string()];
    descriptor.canonical_slot = Some(CanonicalEvidenceSlot::Identifier);

    let json = serde_json::to_string(&descriptor).unwrap();
    let roundtrip: FieldDescriptor = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip.key.dotted(), "legal.article");
    assert!(roundtrip.required);
    assert_eq!(
        roundtrip.canonical_slot,
        Some(CanonicalEvidenceSlot::Identifier)
    );
}

#[test]
fn evidence_attribute_can_carry_confidence_and_source_path() {
    let attribute = EvidenceAttribute {
        value: EvidenceValue::String("corn".to_string()),
        confidence: Some(0.8),
        source_path: Some("$.crop".to_string()),
    };

    assert_eq!(attribute.confidence, Some(0.8));
    assert_eq!(attribute.source_path.as_deref(), Some("$.crop"));
}
