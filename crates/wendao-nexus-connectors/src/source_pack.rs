//! Source pack manifest loader for deterministic local corpus packs.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use wendao_nexus_core::{
    AuthorityLevel, KnowledgeSourceConnector, KnowledgeSourceKind, NexusError, NexusResult,
    NexusSourceRecord, SOURCE_PACK_DISPLAY_NAME_METADATA_KEY, SOURCE_PACK_DOMAIN_METADATA_KEY,
    SOURCE_PACK_FIXTURE_PATH_METADATA_KEY, SOURCE_PACK_ID_METADATA_KEY,
    SOURCE_PACK_PRODUCER_METADATA_KEY, SOURCE_PACK_SCHEMA_VERSION_METADATA_KEY,
    SOURCE_PACK_VERSION_METADATA_KEY, SourceCapabilities, SourceDomain,
};

use crate::local_corpus::{LocalCorpusConfig, LocalCorpusConnector};

/// Current source-pack manifest schema version.
pub const SOURCE_PACK_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Source-pack manifest shape used by deterministic fixture packs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourcePackManifest {
    pub source_pack: SourcePackMetadata,
    #[serde(default)]
    pub sources: Vec<SourcePackSource>,
}

/// Top-level source-pack identity and default policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourcePackMetadata {
    pub id: String,
    pub version: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub domain: SourceDomain,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub producer: Option<String>,
    #[serde(default)]
    pub authority_level: Option<AuthorityLevel>,
    #[serde(default)]
    pub license: Option<String>,
}

/// One local corpus source declared inside a source pack.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourcePackSource {
    pub source_id: String,
    pub kind: KnowledgeSourceKind,
    pub fixture_path: String,
    #[serde(default)]
    pub canonical_uri_prefix: Option<String>,
    #[serde(default)]
    pub authority_level: Option<AuthorityLevel>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl SourcePackSource {
    pub fn source_record(&self, pack: &SourcePackMetadata) -> NexusSourceRecord {
        let mut record = NexusSourceRecord::new(self.source_id.clone(), self.kind.clone());
        record.base_uri = self
            .canonical_uri_prefix
            .clone()
            .or_else(|| Some(self.fixture_path.clone()));
        record.license_policy = pack.license.clone();
        record.authority_level = self
            .authority_level
            .or(pack.authority_level)
            .unwrap_or(AuthorityLevel::Unknown);
        record.sync_policy = Some("source_pack_fixture".to_string());
        record.capabilities = SourceCapabilities::local_corpus_fixture();
        record.enabled = self.enabled;
        record
            .metadata
            .insert(SOURCE_PACK_ID_METADATA_KEY.to_string(), pack.id.clone());
        record.metadata.insert(
            SOURCE_PACK_VERSION_METADATA_KEY.to_string(),
            pack.version.clone(),
        );
        record.metadata.insert(
            SOURCE_PACK_SCHEMA_VERSION_METADATA_KEY.to_string(),
            pack.schema_version.to_string(),
        );
        record.metadata.insert(
            SOURCE_PACK_DOMAIN_METADATA_KEY.to_string(),
            pack.domain.wire_label(),
        );
        record.metadata.insert(
            SOURCE_PACK_FIXTURE_PATH_METADATA_KEY.to_string(),
            self.fixture_path.clone(),
        );
        if let Some(display_name) = &pack.display_name {
            record.metadata.insert(
                SOURCE_PACK_DISPLAY_NAME_METADATA_KEY.to_string(),
                display_name.clone(),
            );
        }
        if let Some(producer) = &pack.producer {
            record.metadata.insert(
                SOURCE_PACK_PRODUCER_METADATA_KEY.to_string(),
                producer.clone(),
            );
        }
        record
    }
}

/// Resolved source-pack connectors and manifest metadata.
#[derive(Clone, Debug)]
pub struct SourcePack {
    manifest_path: PathBuf,
    manifest: SourcePackManifest,
    connectors: Vec<LocalCorpusConnector>,
}

impl SourcePack {
    pub fn from_path(path: impl AsRef<Path>) -> NexusResult<Self> {
        let path = path.as_ref();
        let manifest = SourcePackManifest::from_path(path)?;
        let base_dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        if manifest.sources.iter().all(|source| !source.enabled) {
            return Err(NexusError::InvalidSource(format!(
                "source pack `{}` has no enabled sources",
                manifest.source_pack.id
            )));
        }

        let mut connectors = Vec::new();
        for source in manifest.sources.iter().filter(|source| source.enabled) {
            let mut config = LocalCorpusConfig::new(&source.source_id, source.kind.clone());
            if let Some(prefix) = &source.canonical_uri_prefix {
                config.canonical_uri_prefix = prefix.clone();
            }
            connectors.push(LocalCorpusConnector::from_path(
                config,
                base_dir.join(&source.fixture_path),
            )?);
        }

        Ok(Self {
            manifest_path: path.to_path_buf(),
            manifest,
            connectors,
        })
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn manifest(&self) -> &SourcePackManifest {
        &self.manifest
    }

    pub fn connectors(&self) -> &[LocalCorpusConnector] {
        &self.connectors
    }

    pub fn source_records(&self) -> Vec<NexusSourceRecord> {
        self.manifest
            .sources
            .iter()
            .map(|source| source.source_record(&self.manifest.source_pack))
            .collect()
    }

    pub fn connector(&self, source_id: &str) -> Option<&LocalCorpusConnector> {
        self.connectors
            .iter()
            .find(|connector| connector.source_id() == source_id)
    }

    pub fn source(&self, source_id: &str) -> Option<&SourcePackSource> {
        self.manifest
            .sources
            .iter()
            .find(|source| source.source_id == source_id)
    }
}

impl SourcePackManifest {
    pub fn from_path(path: impl AsRef<Path>) -> NexusResult<Self> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        if !matches!(extension.as_deref(), Some("toml" | "json")) {
            return Err(NexusError::InvalidSource(format!(
                "source pack manifest `{}` must use .toml or .json",
                path.display()
            )));
        }

        let content = fs::read_to_string(path).map_err(|error| {
            invalid_source(format!("failed to read `{}`: {error}", path.display()))
        })?;
        let manifest = match extension.as_deref() {
            Some("toml") => toml::from_str(&content).map_err(|error| {
                invalid_source(format!(
                    "failed to parse TOML `{}`: {error}",
                    path.display()
                ))
            })?,
            Some("json") => serde_json::from_str(&content).map_err(|error| {
                invalid_source(format!(
                    "failed to parse JSON `{}`: {error}",
                    path.display()
                ))
            })?,
            _ => unreachable!("source pack manifest extension is checked before reading"),
        };

        validate_manifest(&manifest)?;
        Ok(manifest)
    }
}

fn validate_manifest(manifest: &SourcePackManifest) -> NexusResult<()> {
    if manifest.source_pack.id.trim().is_empty() {
        return Err(NexusError::InvalidSource(
            "source pack id must not be empty".to_string(),
        ));
    }
    if manifest.source_pack.id != manifest.source_pack.id.trim() {
        return Err(NexusError::InvalidSource(format!(
            "source pack id `{}` must not contain leading or trailing whitespace",
            manifest.source_pack.id
        )));
    }
    if manifest.source_pack.version.trim().is_empty() {
        return Err(NexusError::InvalidSource(format!(
            "source pack `{}` version must not be empty",
            manifest.source_pack.id
        )));
    }
    if manifest.source_pack.version != manifest.source_pack.version.trim() {
        return Err(NexusError::InvalidSource(format!(
            "source pack `{}` version `{}` must not contain leading or trailing whitespace",
            manifest.source_pack.id, manifest.source_pack.version
        )));
    }
    if manifest.source_pack.schema_version != SOURCE_PACK_MANIFEST_SCHEMA_VERSION {
        return Err(NexusError::InvalidSource(format!(
            "source pack `{}` schema_version {} is unsupported; expected {}",
            manifest.source_pack.id,
            manifest.source_pack.schema_version,
            SOURCE_PACK_MANIFEST_SCHEMA_VERSION
        )));
    }
    if let Some(producer) = &manifest.source_pack.producer {
        validate_optional_metadata_value(&manifest.source_pack.id, "producer", producer)?;
    }
    if let Some(display_name) = &manifest.source_pack.display_name {
        validate_optional_metadata_value(&manifest.source_pack.id, "display_name", display_name)?;
    }
    if let Some(license) = &manifest.source_pack.license {
        validate_optional_metadata_value(&manifest.source_pack.id, "license", license)?;
    }
    if manifest.sources.is_empty() {
        return Err(NexusError::InvalidSource(format!(
            "source pack `{}` must declare at least one source",
            manifest.source_pack.id
        )));
    }

    for source in &manifest.sources {
        if source.source_id.trim().is_empty() {
            return Err(NexusError::InvalidSource(format!(
                "source pack `{}` contains an empty source_id",
                manifest.source_pack.id
            )));
        }
        if source.source_id != source.source_id.trim() {
            return Err(NexusError::InvalidSource(format!(
                "source pack `{}` source_id `{}` must not contain leading or trailing whitespace",
                manifest.source_pack.id, source.source_id
            )));
        }
        if source.fixture_path.trim().is_empty() {
            return Err(NexusError::InvalidSource(format!(
                "source `{}` fixture_path must not be empty",
                source.source_id
            )));
        }
        if source.fixture_path != source.fixture_path.trim() {
            return Err(NexusError::InvalidSource(format!(
                "source `{}` fixture_path must not contain leading or trailing whitespace",
                source.source_id
            )));
        }
        if let Some(canonical_uri_prefix) = &source.canonical_uri_prefix {
            if canonical_uri_prefix.trim().is_empty() {
                return Err(NexusError::InvalidSource(format!(
                    "source `{}` canonical_uri_prefix must not be empty",
                    source.source_id
                )));
            }
            if canonical_uri_prefix != canonical_uri_prefix.trim() {
                return Err(NexusError::InvalidSource(format!(
                    "source `{}` canonical_uri_prefix `{canonical_uri_prefix}` must not contain leading or trailing whitespace",
                    source.source_id
                )));
            }
        }
    }

    let mut source_ids = BTreeSet::new();
    for source in &manifest.sources {
        if !source_ids.insert(source.source_id.as_str()) {
            return Err(NexusError::InvalidSource(format!(
                "source pack `{}` contains duplicate source_id `{}`",
                manifest.source_pack.id, source.source_id
            )));
        }
    }

    Ok(())
}

fn default_enabled() -> bool {
    true
}

fn default_schema_version() -> u32 {
    SOURCE_PACK_MANIFEST_SCHEMA_VERSION
}

fn validate_optional_metadata_value(
    pack_id: &str,
    field_name: &str,
    value: &str,
) -> NexusResult<()> {
    if value.trim().is_empty() {
        return Err(NexusError::InvalidSource(format!(
            "source pack `{pack_id}` {field_name} must not be empty"
        )));
    }
    if value != value.trim() {
        return Err(NexusError::InvalidSource(format!(
            "source pack `{pack_id}` {field_name} `{value}` must not contain leading or trailing whitespace"
        )));
    }
    Ok(())
}

fn invalid_source(message: String) -> NexusError {
    NexusError::InvalidSource(message)
}
