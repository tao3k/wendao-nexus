use std::path::PathBuf;

use arrow_array::RecordBatch;
use wendao_nexus::NexusFixtureHarness;
use wendao_nexus_flight::{NexusFlightCommand, NexusFlightProviderError};
use wendao_nexus_runtime::LocalFileArtifactStore;

pub(crate) struct FixtureFlightHarness {
    inner: NexusFixtureHarness,
}

impl FixtureFlightHarness {
    pub(crate) async fn build() -> Self {
        Self::build_with_manifest(source_pack_fixture_manifest()).await
    }

    pub(crate) async fn build_with_manifest(manifest: PathBuf) -> Self {
        let artifact_root = artifact_dir("fixture_flight_harness_artifacts");
        cleanup_dir(&artifact_root);
        let inner = NexusFixtureHarness::load_source_pack(manifest, &artifact_root)
            .await
            .unwrap();
        assert!(inner.ingest_report().ingested_documents > 0);

        Self { inner }
    }

    pub(crate) async fn handle_descriptor(&self, command: NexusFlightCommand) -> RecordBatch {
        self.handle_command_descriptor_result(command)
            .await
            .unwrap()
    }

    pub(crate) async fn handle_command_descriptor_result(
        &self,
        command: NexusFlightCommand,
    ) -> Result<RecordBatch, NexusFlightProviderError> {
        self.inner.handle_command(command).await
    }

    pub(crate) async fn handle_command_json_result(
        &self,
        bytes: Vec<u8>,
    ) -> Result<RecordBatch, NexusFlightProviderError> {
        self.inner.handle_encoded_command(bytes).await
    }

    pub(crate) fn artifact_store(&self) -> &LocalFileArtifactStore {
        self.inner.artifact_store()
    }

    pub(crate) fn cleanup(self) {
        cleanup_dir(&self.inner.artifact_root().to_path_buf());
    }
}

pub(crate) fn source_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/medical_baseline/source_pack.toml",
    )
}

pub(crate) fn customer_private_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/customer_private_sop/source_pack.toml",
    )
}

pub(crate) fn legal_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/legal_compliance/source_pack.toml",
    )
}

pub(crate) fn agriculture_pack_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/agriculture_market/source_pack.toml",
    )
}

pub(crate) fn real_medical_pubmed_snapshot_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/real_medical_pubmed_snapshot/source_pack.toml",
    )
}

pub(crate) fn real_wikipedia_science_subset_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../wendao-nexus-connectors/tests/fixtures/source_packs/real_wikipedia_science_subset/source_pack.toml",
    )
}

pub(crate) fn artifact_dir(test_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wendao-nexus-{test_name}-{}", uuid::Uuid::new_v4()))
}

fn cleanup_dir(path: &PathBuf) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}
