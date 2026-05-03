//! Local fixture corpus connector for deterministic source-pack validation.
//!
//! This connector is fixture-only. It loads already-normalized JSONL/Markdown
//! records so contracts can be tested without live services. Its frontmatter
//! support is a deterministic test convenience, not a production Markdown
//! parser or document extraction layer.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use wendao_nexus_core::{
    DeltaBatch, DiscoveryBatch, KnowledgeSourceConnector, KnowledgeSourceKind, NexusError,
    NexusResult, RawSourceDocument, SOURCE_METADATA_TITLE_KEY, SOURCE_METADATA_UPDATED_AT_KEY,
    SourceCapabilities, SourceChange, SourceCheckpoint, SourceCursor, SourceItemRef,
};

/// Configuration for a deterministic local corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCorpusConfig {
    pub source_id: String,
    pub source_kind: KnowledgeSourceKind,
    pub canonical_uri_prefix: String,
}

impl LocalCorpusConfig {
    pub fn new(source_id: impl Into<String>, source_kind: KnowledgeSourceKind) -> Self {
        let source_id = source_id.into();
        Self {
            canonical_uri_prefix: format!("local-corpus://{source_id}"),
            source_id,
            source_kind,
        }
    }
}

/// File-backed deterministic fixture corpus connector.
///
/// Production external documents should be parsed by Wendao-side parser or
/// Docling pipelines before they are handed to Nexus contracts.
#[derive(Clone, Debug)]
pub struct LocalCorpusConnector {
    config: LocalCorpusConfig,
    items: Vec<SourceItemRef>,
    documents: BTreeMap<String, RawSourceDocument>,
}

impl LocalCorpusConnector {
    pub fn from_path(config: LocalCorpusConfig, path: impl AsRef<Path>) -> NexusResult<Self> {
        let path = path.as_ref();
        let root = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        let mut documents = Vec::new();
        load_path(&config, path, &root, &mut documents)?;
        Self::from_raw_documents(config, documents)
    }

    pub fn from_raw_documents(
        config: LocalCorpusConfig,
        documents: Vec<RawSourceDocument>,
    ) -> NexusResult<Self> {
        let mut items = Vec::with_capacity(documents.len());
        let mut indexed_documents = BTreeMap::new();

        for document in documents {
            if document.source_id != config.source_id {
                return Err(NexusError::InvalidSource(format!(
                    "local corpus document `{}` belongs to source `{}` but connector source is `{}`",
                    document.external_id, document.source_id, config.source_id
                )));
            }

            let mut item =
                SourceItemRef::new(document.source_id.clone(), document.external_id.clone());
            item.canonical_uri = Some(document.canonical_uri.clone());
            item.revision_id = document.revision_id_metadata().map(ToOwned::to_owned);
            item.metadata = document.metadata.clone();

            if indexed_documents
                .insert(document.external_id.clone(), document)
                .is_some()
            {
                return Err(NexusError::InvalidSource(
                    "local corpus contains duplicate external_id values".to_string(),
                ));
            }
            items.push(item);
        }

        items.sort_by(|left, right| left.external_id.cmp(&right.external_id));

        Ok(Self {
            config,
            items,
            documents: indexed_documents,
        })
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[async_trait]
impl KnowledgeSourceConnector for LocalCorpusConnector {
    fn source_id(&self) -> &str {
        &self.config.source_id
    }

    fn source_kind(&self) -> KnowledgeSourceKind {
        self.config.source_kind.clone()
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::local_corpus_fixture()
    }

    async fn discover(&self, _cursor: Option<SourceCursor>) -> NexusResult<DiscoveryBatch> {
        Ok(DiscoveryBatch {
            source_id: self.config.source_id.clone(),
            items: self.items.clone(),
            next_cursor: None,
            observed_at: Utc::now(),
        })
    }

    async fn fetch(&self, item: SourceItemRef) -> NexusResult<RawSourceDocument> {
        self.documents
            .get(&item.external_id)
            .cloned()
            .ok_or_else(|| NexusError::NotFound {
                source_id: self.config.source_id.clone(),
                external_id: item.external_id,
            })
    }

    async fn delta(&self, mut since: SourceCheckpoint) -> NexusResult<DeltaBatch> {
        since.last_success_at = Some(Utc::now());

        Ok(DeltaBatch {
            source_id: self.config.source_id.clone(),
            changes: self
                .items
                .iter()
                .cloned()
                .map(SourceChange::Upsert)
                .collect(),
            next_checkpoint: since,
            observed_at: Utc::now(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct LocalCorpusJsonRecord {
    #[serde(default)]
    source_id: Option<String>,
    external_id: String,
    #[serde(default)]
    canonical_uri: Option<String>,
    #[serde(default)]
    media_type: Option<String>,
    body: String,
    #[serde(default)]
    source_updated_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    content_hash: Option<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

fn load_path(
    config: &LocalCorpusConfig,
    path: &Path,
    root: &Path,
    documents: &mut Vec<RawSourceDocument>,
) -> NexusResult<()> {
    if path.is_dir() {
        let mut entries = fs::read_dir(path)
            .map_err(|error| {
                invalid_source(format!("failed to read `{}`: {error}", path.display()))
            })?
            .map(|entry| {
                entry.map(|entry| entry.path()).map_err(|error| {
                    invalid_source(format!(
                        "failed to read entry in `{}`: {error}",
                        path.display()
                    ))
                })
            })
            .collect::<NexusResult<Vec<_>>>()?;
        entries.sort();

        for entry in entries {
            load_path(config, &entry, root, documents)?;
        }

        return Ok(());
    }

    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jsonl") => load_jsonl(config, path, documents),
        Some("md") | Some("markdown") => load_markdown(config, path, root, documents),
        _ => Ok(()),
    }
}

fn load_jsonl(
    config: &LocalCorpusConfig,
    path: &Path,
    documents: &mut Vec<RawSourceDocument>,
) -> NexusResult<()> {
    let content = fs::read_to_string(path)
        .map_err(|error| invalid_source(format!("failed to read `{}`: {error}", path.display())))?;

    for (line_index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let record: LocalCorpusJsonRecord = serde_json::from_str(line).map_err(|error| {
            invalid_source(format!(
                "failed to parse JSONL record {} in `{}`: {error}",
                line_index + 1,
                path.display()
            ))
        })?;
        documents.push(raw_from_json_record(config, record)?);
    }

    Ok(())
}

fn raw_from_json_record(
    config: &LocalCorpusConfig,
    record: LocalCorpusJsonRecord,
) -> NexusResult<RawSourceDocument> {
    if let Some(source_id) = &record.source_id
        && source_id != &config.source_id
    {
        return Err(NexusError::InvalidSource(format!(
            "local corpus record `{}` belongs to source `{source_id}` but connector source is `{}`",
            record.external_id, config.source_id
        )));
    }

    let mut metadata = record.metadata;
    if !metadata.contains_key(SOURCE_METADATA_TITLE_KEY) {
        metadata.insert(
            SOURCE_METADATA_TITLE_KEY.to_string(),
            record.external_id.clone(),
        );
    }

    Ok(RawSourceDocument {
        source_id: config.source_id.clone(),
        canonical_uri: record
            .canonical_uri
            .unwrap_or_else(|| default_canonical_uri(config, &record.external_id)),
        media_type: record
            .media_type
            .unwrap_or_else(|| "text/plain".to_string()),
        payload: record.body.into_bytes(),
        fetched_at: Utc::now(),
        source_updated_at: record.source_updated_at,
        content_hash: record.content_hash,
        metadata,
        external_id: record.external_id,
    })
}

fn load_markdown(
    config: &LocalCorpusConfig,
    path: &Path,
    root: &Path,
    documents: &mut Vec<RawSourceDocument>,
) -> NexusResult<()> {
    let content = fs::read_to_string(path)
        .map_err(|error| invalid_source(format!("failed to read `{}`: {error}", path.display())))?;
    let (mut metadata, body) = split_markdown_front_matter(&content);
    let fallback_external_id = external_id_from_path(path, root);
    let external_id = metadata
        .remove("external_id")
        .unwrap_or(fallback_external_id);
    let canonical_uri = metadata
        .remove("canonical_uri")
        .unwrap_or_else(|| default_canonical_uri(config, &external_id));
    let title = metadata
        .get(SOURCE_METADATA_TITLE_KEY)
        .cloned()
        .or_else(|| markdown_h1(&body))
        .unwrap_or_else(|| external_id.clone());
    metadata.insert(SOURCE_METADATA_TITLE_KEY.to_string(), title);
    metadata.insert(
        "corpus_path".to_string(),
        path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string(),
    );

    documents.push(RawSourceDocument {
        source_id: config.source_id.clone(),
        external_id,
        canonical_uri,
        media_type: "text/markdown".to_string(),
        payload: body.into_bytes(),
        fetched_at: Utc::now(),
        source_updated_at: metadata
            .get(SOURCE_METADATA_UPDATED_AT_KEY)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc)),
        content_hash: metadata.get("content_hash").cloned(),
        metadata,
    });

    Ok(())
}

fn split_markdown_front_matter(content: &str) -> (BTreeMap<String, String>, String) {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return (BTreeMap::new(), content.to_string());
    }

    let mut metadata = BTreeMap::new();
    let mut body = Vec::new();
    let mut in_front_matter = true;

    for line in lines {
        if in_front_matter {
            if line.trim() == "---" {
                in_front_matter = false;
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                metadata.insert(key.trim().to_string(), value.trim().to_string());
            }
        } else {
            body.push(line);
        }
    }

    if in_front_matter {
        (BTreeMap::new(), content.to_string())
    } else {
        (metadata, body.join("\n"))
    }
}

fn markdown_h1(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        line.trim_start()
            .strip_prefix("# ")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn external_id_from_path(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let without_extension = relative.with_extension("");
    without_extension
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn default_canonical_uri(config: &LocalCorpusConfig, external_id: &str) -> String {
    format!(
        "{}/{}",
        config.canonical_uri_prefix.trim_end_matches('/'),
        external_id
    )
}

fn invalid_source(message: String) -> NexusError {
    NexusError::InvalidSource(message)
}
