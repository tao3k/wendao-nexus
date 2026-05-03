use chrono::{TimeZone, Utc};
use wendao_nexus_flight::{
    FlightCompareResultRow, FlightOpenDocumentRow, FlightSearchResultRow, FlightStatusRow,
    FlightSyncResultRow, compare_result_record_batch, open_document_record_batch,
    search_result_record_batch, status_record_batch, sync_result_record_batch,
};

use super::fixtures::compact_batch_snapshot;

#[test]
fn route_batch_rows_match_snapshots() {
    let fetched_at = Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap();
    let search_batch = search_result_record_batch(&[FlightSearchResultRow {
        source_id: "legal-compliance-demo".to_string(),
        external_id: "legal/privacy/data-retention-clause".to_string(),
        title: "Example Privacy Code Article 12".to_string(),
        snippet: Some("retain audit evidence".to_string()),
        score: Some(1.0),
        authority_level: "Official".to_string(),
        canonical_uri: "https://law.example.test/privacy-code/article-12".to_string(),
        fetched_at: Some(fetched_at),
        content_hash: "sha256:legal".to_string(),
        provenance_json: Some(r#"{"primary":{"source_id":"legal-compliance-demo"}}"#.to_string()),
        section_id: Some("article-12".to_string()),
        heading_path_json: Some(r#"["Privacy Code","Article 12"]"#.to_string()),
        source_kind: Some("LegalCorpus".to_string()),
        published_at: Some(fetched_at),
        source_updated_at: Some(fetched_at),
        trust_score: Some(1.0),
        freshness_score: Some(0.9),
        semantic_score: None,
        lexical_score: Some(1.0),
        rerank_score: Some(1.0),
        license_json: Some(r#"{"name":"Official Example License"}"#.to_string()),
        metadata_json: Some(r#"{"jurisdiction":"US-EXAMPLE"}"#.to_string()),
        doi: None,
        pmid: None,
        jurisdiction: Some("US-EXAMPLE".to_string()),
        evidence_kind: Some("law_clause".to_string()),
    }])
    .unwrap();
    let open_batch = open_document_record_batch(&[FlightOpenDocumentRow {
        source_id: "legal-compliance-demo".to_string(),
        external_id: "legal/privacy/data-retention-clause".to_string(),
        canonical_uri: "https://law.example.test/privacy-code/article-12".to_string(),
        title: "Example Privacy Code Article 12".to_string(),
        section_id: Some("article-12".to_string()),
        heading_path_json: Some(r#"["Privacy Code","Article 12"]"#.to_string()),
        body: Some("Agencies retain audit evidence.".to_string()),
        metadata_json: Some(r#"{"article":"Article 12"}"#.to_string()),
        provenance_json: Some(r#"{"source_id":"legal-compliance-demo"}"#.to_string()),
    }])
    .unwrap();
    let sync_batch = sync_result_record_batch(&[FlightSyncResultRow {
        job_id: "job-1".to_string(),
        source_id: "legal-compliance-demo".to_string(),
        job_kind: "Normalize".to_string(),
        status: "Succeeded".to_string(),
        cursor: Some("cursor-1".to_string()),
        dedup_hit: false,
        error: None,
    }])
    .unwrap();
    let status_batch = status_record_batch(&[FlightStatusRow {
        source_id: "legal-compliance-demo".to_string(),
        enabled: true,
        last_success_at: Some(fetched_at),
        last_seen_revision: Some("rev-1".to_string()),
        last_content_hash: Some("sha256:legal".to_string()),
        rate_limit_state: None,
    }])
    .unwrap();
    let compare_batch = compare_result_record_batch(&[FlightCompareResultRow {
        claim: "retain audit evidence".to_string(),
        verdict: "evidence_available".to_string(),
        conflict_detected: false,
        insufficient_authority: false,
        stale_evidence: false,
        provenance_json: Some(r#"{"records":[{"source_id":"legal-compliance-demo"}]}"#.to_string()),
    }])
    .unwrap();

    assert_eq!(
        compact_batch_snapshot(&search_batch),
        r#"source_id=legal-compliance-demo
external_id=legal/privacy/data-retention-clause
title=Example Privacy Code Article 12
snippet=retain audit evidence
score=1
authority_level=Official
canonical_uri=https://law.example.test/privacy-code/article-12
fetched_at=1777723200000000000
content_hash=sha256:legal
provenance_json={"primary":{"source_id":"legal-compliance-demo"}}
section_id=article-12
heading_path_json=["Privacy Code","Article 12"]
source_kind=LegalCorpus
published_at=1777723200000000000
source_updated_at=1777723200000000000
trust_score=1
freshness_score=0.9
semantic_score=<null>
lexical_score=1
rerank_score=1
license_json={"name":"Official Example License"}
metadata_json={"jurisdiction":"US-EXAMPLE"}
doi=<null>
pmid=<null>
jurisdiction=US-EXAMPLE
evidence_kind=law_clause"#
    );
    assert_eq!(
        compact_batch_snapshot(&open_batch),
        r#"source_id=legal-compliance-demo
external_id=legal/privacy/data-retention-clause
canonical_uri=https://law.example.test/privacy-code/article-12
title=Example Privacy Code Article 12
section_id=article-12
heading_path_json=["Privacy Code","Article 12"]
body=Agencies retain audit evidence.
metadata_json={"article":"Article 12"}
provenance_json={"source_id":"legal-compliance-demo"}"#
    );
    assert_eq!(
        compact_batch_snapshot(&sync_batch),
        r#"job_id=job-1
source_id=legal-compliance-demo
job_kind=Normalize
status=Succeeded
cursor=cursor-1
dedup_hit=false
error=<null>"#
    );
    assert_eq!(
        compact_batch_snapshot(&status_batch),
        r#"source_id=legal-compliance-demo
enabled=true
last_success_at=1777723200000000000
last_seen_revision=rev-1
last_content_hash=sha256:legal
rate_limit_state=<null>"#
    );
    assert_eq!(
        compact_batch_snapshot(&compare_batch),
        r#"claim=retain audit evidence
verdict=evidence_available
conflict_detected=false
insufficient_authority=false
stale_evidence=false
provenance_json={"records":[{"source_id":"legal-compliance-demo"}]}"#
    );
}
