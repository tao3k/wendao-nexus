//! Artifact persistence boundary for raw and normalized Nexus payloads.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use wendao_nexus_core::{NexusError, NexusResult};

/// Artifact class written by Nexus runtime flows.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ArtifactKind {
    RawSourcePayload,
    NormalizedDocument,
}

impl ArtifactKind {
    pub fn as_slug(self) -> &'static str {
        match self {
            Self::RawSourcePayload => "raw-source-payload",
            Self::NormalizedDocument => "normalized-document",
        }
    }
}

/// Metadata sidecar for one persisted artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactDescriptor {
    pub source_id: String,
    pub external_id: String,
    pub content_hash: String,
    pub kind: ArtifactKind,
    pub media_type: String,
    pub byte_len: u64,
    pub relative_path: String,
    pub created_at: DateTime<Utc>,
    pub metadata: BTreeMap<String, String>,
}

/// Bytes and metadata accepted by an artifact backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactWrite {
    pub source_id: String,
    pub external_id: String,
    pub content_hash: String,
    pub kind: ArtifactKind,
    pub media_type: String,
    pub bytes: Vec<u8>,
    pub metadata: BTreeMap<String, String>,
}

impl ArtifactWrite {
    pub fn new(
        source_id: impl Into<String>,
        external_id: impl Into<String>,
        content_hash: impl Into<String>,
        kind: ArtifactKind,
        media_type: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            external_id: external_id.into(),
            content_hash: content_hash.into(),
            kind,
            media_type: media_type.into(),
            bytes,
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: BTreeMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Artifact read result with descriptor and payload bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactPayload {
    pub descriptor: ArtifactDescriptor,
    pub bytes: Vec<u8>,
}

/// Persistence facade for source artifacts.
#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn put_artifact(&self, artifact: ArtifactWrite) -> NexusResult<ArtifactDescriptor>;

    async fn get_artifact(
        &self,
        source_id: &str,
        external_id: &str,
        kind: ArtifactKind,
        content_hash: &str,
    ) -> NexusResult<Option<ArtifactPayload>>;

    async fn list_artifacts(
        &self,
        source_id: &str,
        external_id: &str,
    ) -> NexusResult<Vec<ArtifactDescriptor>>;
}

/// Local filesystem artifact backend for deterministic tests and embedded runs.
#[derive(Clone, Debug)]
pub struct LocalFileArtifactStore {
    root: Arc<PathBuf>,
}

struct ArtifactPaths {
    directory: PathBuf,
    payload_path: PathBuf,
    descriptor_path: PathBuf,
    payload_relative_path: String,
}

impl LocalFileArtifactStore {
    pub fn open(root: impl AsRef<Path>) -> NexusResult<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)
            .map_err(|error| artifact_error(format!("create root `{}`", root.display()), error))?;
        Ok(Self {
            root: Arc::new(root),
        })
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    fn artifact_paths(
        &self,
        source_id: &str,
        external_id: &str,
        kind: ArtifactKind,
        content_hash: &str,
    ) -> ArtifactPaths {
        let source = sanitize_path_component(source_id);
        let external = sanitize_path_component(external_id);
        let hash = sanitize_path_component(content_hash);
        let kind_slug = kind.as_slug();
        let relative_dir = format!("{source}/{external}/{kind_slug}");
        let payload_relative_path = format!("{relative_dir}/{hash}.payload");
        let descriptor_relative_path = format!("{relative_dir}/{hash}.artifact.json");
        let directory = self.root.join(&relative_dir);
        ArtifactPaths {
            directory,
            payload_path: self.root.join(&payload_relative_path),
            descriptor_path: self.root.join(descriptor_relative_path),
            payload_relative_path,
        }
    }
}

#[async_trait]
impl ArtifactStore for LocalFileArtifactStore {
    async fn put_artifact(&self, artifact: ArtifactWrite) -> NexusResult<ArtifactDescriptor> {
        validate_artifact_write(&artifact)?;

        let paths = self.artifact_paths(
            &artifact.source_id,
            &artifact.external_id,
            artifact.kind,
            &artifact.content_hash,
        );
        if let Some(descriptor) = existing_descriptor(&paths).await? {
            return Ok(descriptor);
        }
        fs::create_dir_all(&paths.directory)
            .await
            .map_err(|error| {
                artifact_error(format!("create `{}`", paths.directory.display()), error)
            })?;
        fs::write(&paths.payload_path, &artifact.bytes)
            .await
            .map_err(|error| {
                artifact_error(format!("write `{}`", paths.payload_path.display()), error)
            })?;

        let descriptor = ArtifactDescriptor {
            source_id: artifact.source_id,
            external_id: artifact.external_id,
            content_hash: artifact.content_hash,
            kind: artifact.kind,
            media_type: artifact.media_type,
            byte_len: artifact.bytes.len() as u64,
            relative_path: paths.payload_relative_path,
            created_at: Utc::now(),
            metadata: artifact.metadata,
        };
        let descriptor_json = serde_json::to_vec_pretty(&descriptor)
            .map_err(|error| NexusError::Artifact(format!("serialize descriptor: {error}")))?;
        fs::write(&paths.descriptor_path, descriptor_json)
            .await
            .map_err(|error| {
                artifact_error(
                    format!("write `{}`", paths.descriptor_path.display()),
                    error,
                )
            })?;

        Ok(descriptor)
    }

    async fn get_artifact(
        &self,
        source_id: &str,
        external_id: &str,
        kind: ArtifactKind,
        content_hash: &str,
    ) -> NexusResult<Option<ArtifactPayload>> {
        let paths = self.artifact_paths(source_id, external_id, kind, content_hash);
        if fs::metadata(&paths.descriptor_path).await.is_err() {
            return Ok(None);
        }

        let descriptor_bytes = fs::read(&paths.descriptor_path).await.map_err(|error| {
            artifact_error(format!("read `{}`", paths.descriptor_path.display()), error)
        })?;
        let descriptor: ArtifactDescriptor =
            serde_json::from_slice(&descriptor_bytes).map_err(|error| {
                NexusError::Artifact(format!(
                    "parse descriptor `{}`: {error}",
                    paths.descriptor_path.display()
                ))
            })?;
        let payload_path = self.root.join(&descriptor.relative_path);
        let bytes = fs::read(&payload_path)
            .await
            .map_err(|error| artifact_error(format!("read `{}`", payload_path.display()), error))?;

        Ok(Some(ArtifactPayload { descriptor, bytes }))
    }

    async fn list_artifacts(
        &self,
        source_id: &str,
        external_id: &str,
    ) -> NexusResult<Vec<ArtifactDescriptor>> {
        let base = self
            .root
            .join(sanitize_path_component(source_id))
            .join(sanitize_path_component(external_id));
        if fs::metadata(&base).await.is_err() {
            return Ok(Vec::new());
        }

        let mut descriptors = Vec::new();
        let mut kind_entries = fs::read_dir(&base)
            .await
            .map_err(|error| artifact_error(format!("read `{}`", base.display()), error))?;
        while let Some(kind_entry) = kind_entries
            .next_entry()
            .await
            .map_err(|error| artifact_error(format!("read `{}`", base.display()), error))?
        {
            if !kind_entry
                .file_type()
                .await
                .map_err(|error| {
                    artifact_error(format!("inspect `{}`", kind_entry.path().display()), error)
                })?
                .is_dir()
            {
                continue;
            }

            let kind_path = kind_entry.path();
            let mut artifact_entries = fs::read_dir(&kind_path).await.map_err(|error| {
                artifact_error(format!("read `{}`", kind_path.display()), error)
            })?;
            while let Some(artifact_entry) = artifact_entries
                .next_entry()
                .await
                .map_err(|error| artifact_error(format!("read `{}`", kind_path.display()), error))?
            {
                let file_name = artifact_entry.file_name();
                let file_name = file_name.to_string_lossy();
                if !file_name.ends_with(".artifact.json") {
                    continue;
                }
                let descriptor_path = artifact_entry.path();
                let descriptor_bytes = fs::read(&descriptor_path).await.map_err(|error| {
                    artifact_error(format!("read `{}`", descriptor_path.display()), error)
                })?;
                let descriptor = serde_json::from_slice(&descriptor_bytes).map_err(|error| {
                    NexusError::Artifact(format!(
                        "parse descriptor `{}`: {error}",
                        descriptor_path.display()
                    ))
                })?;
                descriptors.push(descriptor);
            }
        }

        descriptors.sort_by(|left: &ArtifactDescriptor, right: &ArtifactDescriptor| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.content_hash.cmp(&right.content_hash))
                .then_with(|| left.relative_path.cmp(&right.relative_path))
        });
        Ok(descriptors)
    }
}

async fn existing_descriptor(paths: &ArtifactPaths) -> NexusResult<Option<ArtifactDescriptor>> {
    if fs::metadata(&paths.descriptor_path).await.is_err()
        || fs::metadata(&paths.payload_path).await.is_err()
    {
        return Ok(None);
    }

    let descriptor_bytes = fs::read(&paths.descriptor_path).await.map_err(|error| {
        artifact_error(format!("read `{}`", paths.descriptor_path.display()), error)
    })?;
    let descriptor = serde_json::from_slice(&descriptor_bytes).map_err(|error| {
        NexusError::Artifact(format!(
            "parse descriptor `{}`: {error}",
            paths.descriptor_path.display()
        ))
    })?;
    Ok(Some(descriptor))
}

fn validate_artifact_write(artifact: &ArtifactWrite) -> NexusResult<()> {
    if artifact.source_id.trim().is_empty() {
        return Err(NexusError::Artifact(
            "artifact requires a source_id".to_string(),
        ));
    }
    if artifact.external_id.trim().is_empty() {
        return Err(NexusError::Artifact(
            "artifact requires an external_id".to_string(),
        ));
    }
    if artifact.content_hash.trim().is_empty() {
        return Err(NexusError::Artifact(
            "artifact requires a content_hash".to_string(),
        ));
    }
    if artifact.media_type.trim().is_empty() {
        return Err(NexusError::Artifact(
            "artifact requires a media_type".to_string(),
        ));
    }
    Ok(())
}

fn sanitize_path_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    let sanitized = sanitized.trim_matches('.');
    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized.to_string()
    }
}

fn artifact_error(action: String, error: std::io::Error) -> NexusError {
    NexusError::Artifact(format!("{action}: {error}"))
}
