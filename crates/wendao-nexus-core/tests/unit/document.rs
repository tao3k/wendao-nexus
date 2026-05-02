use wendao_nexus_core::{ExtractedDocumentResource, ExtractedDocumentResourceSet};

#[test]
fn extracted_resource_contract_matches_wendao_document_extract_shape() {
    let payload = r#"{
        "sourcePath": "tenant/protocol.docx",
        "sourceFormat": "docx",
        "totalResources": 1,
        "totalPages": 3,
        "resources": [
            {
                "sourcePath": "tenant/protocol.docx",
                "resourceType": "document",
                "resourcePath": "pages/page-1.md",
                "pageIndex": 0,
                "caption": "Protocol",
                "content": "Parsed by Wendao attachment extraction.",
                "mimeType": "text/markdown",
                "status": "ok",
                "elementId": "body"
            }
        ]
    }"#;

    let resource_set: ExtractedDocumentResourceSet = serde_json::from_str(payload).unwrap();

    assert_eq!(resource_set.source_path, "tenant/protocol.docx");
    assert_eq!(resource_set.source_format, "docx");
    assert_eq!(resource_set.total_resources, 1);
    assert_eq!(resource_set.total_pages, 3);
    assert_eq!(resource_set.resources[0].element_id, "body");
}

#[test]
fn extracted_resource_row_allows_missing_optional_fields() {
    let payload = r#"{
        "resourceType": "text",
        "content": "Wendao parsed text"
    }"#;

    let resource: ExtractedDocumentResource = serde_json::from_str(payload).unwrap();

    assert_eq!(resource.source_path, None);
    assert_eq!(resource.resource_type, "text");
    assert_eq!(resource.content, "Wendao parsed text");
    assert_eq!(resource.page_index, 0);
    assert_eq!(resource.status, "");
}
