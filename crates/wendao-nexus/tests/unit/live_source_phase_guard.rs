use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_DIRECT_DEPENDENCIES: &[&str] = &[
    "reqwest", "ureq", "surf", "isahc", "awc", "futures", "tower",
];
const FORBIDDEN_DOCUMENT_PARSER_DEPENDENCIES: &[&str] = &[
    "pulldown-cmark",
    "comrak",
    "markdown",
    "scraper",
    "html5ever",
    "kuchiki",
    "lol_html",
    "quick-xml",
    "roxmltree",
    "lopdf",
    "pdf",
    "pdfium-render",
    "docx-rs",
    "calamine",
    "docling",
];
const FORBIDDEN_CONNECTOR_SOURCE_TOKENS: &[&str] = &[
    "reqwest::",
    "ureq::",
    "surf::",
    "isahc::",
    "awc::",
    "hyper::Client",
    "hyper_util::client",
    "tokio_tungstenite",
];

// Test-only guard for the current deterministic fixture phase.
//
// These crates are valid backend choices for Wendao-side integration or a
// later Nexus storage/search adapter. They are not part of the public Nexus
// contract in this repository phase, where the facade only proves SourcePack
// ingestion and serverless Arrow Flight batches.
const CURRENT_PHASE_FORBIDDEN_BACKEND_DEPENDENCIES: &[&str] = &[
    "duckdb",
    "datafusion",
    "tantivy",
    "lance",
    "lance-arrow",
    "lancedb",
    "redis",
    "valkey",
    "fred",
    "deadpool-redis",
    "bb8-redis",
    "cocoindex",
];
const FORBIDDEN_RUNTIME_OWNERSHIP_TOKENS: &[&str] = &[
    "pub struct LocalKnowledgeStore",
    "pub trait LocalKnowledgeStore",
    "pub type LocalKnowledgeStore",
    "pub struct InMemoryKnowledgeStore",
    "pub trait InMemoryKnowledgeStore",
    "pub type InMemoryKnowledgeStore",
    "CocoIndex",
    "cocoindex",
];

#[test]
fn current_phase_has_no_live_source_client_or_scheduler_direct_dependencies() {
    let root = workspace_root();
    for manifest in workspace_manifests(&root) {
        let content = fs::read_to_string(&manifest).unwrap();
        let package = package_name(&content).unwrap_or_else(|| manifest.display().to_string());
        for dependency in FORBIDDEN_DIRECT_DEPENDENCIES {
            assert!(
                !manifest_declares_dependency(&content, dependency),
                "{package} must not declare `{dependency}` during the deterministic fixture phase"
            );
        }
    }
}

#[test]
fn current_phase_has_no_document_parser_direct_dependencies() {
    let root = workspace_root();
    for manifest in workspace_manifests(&root) {
        let content = fs::read_to_string(&manifest).unwrap();
        let package = package_name(&content).unwrap_or_else(|| manifest.display().to_string());
        for dependency in FORBIDDEN_DOCUMENT_PARSER_DEPENDENCIES {
            assert!(
                !manifest_declares_dependency(&content, dependency),
                "{package} must not declare parser dependency `{dependency}`; parsing belongs to Wendao-side pipelines"
            );
        }
    }
}

#[test]
fn current_phase_has_no_storage_search_cache_or_cocoindex_direct_dependencies() {
    let root = workspace_root();
    for manifest in workspace_manifests(&root) {
        let content = fs::read_to_string(&manifest).unwrap();
        let package = package_name(&content).unwrap_or_else(|| manifest.display().to_string());
        for dependency in CURRENT_PHASE_FORBIDDEN_BACKEND_DEPENDENCIES {
            assert!(
                !manifest_declares_dependency(&content, dependency),
                "{package} must not declare `{dependency}` during the contract-only fixture phase"
            );
        }
    }
}

#[test]
fn current_phase_connector_sources_do_not_import_live_clients() {
    let root = workspace_root();
    for source_file in rust_sources(&root.join("crates/wendao-nexus-connectors/src")) {
        let content = fs::read_to_string(&source_file).unwrap();
        for token in FORBIDDEN_CONNECTOR_SOURCE_TOKENS {
            assert!(
                !content.contains(token),
                "{} must not use `{token}` during the deterministic fixture phase",
                source_file.display()
            );
        }
    }
}

#[test]
fn current_phase_crate_sources_do_not_own_local_knowledge_store_or_cocoindex() {
    let root = workspace_root();
    for source_file in crate_src_rust_sources(&root) {
        let content = fs::read_to_string(&source_file).unwrap();
        for token in FORBIDDEN_RUNTIME_OWNERSHIP_TOKENS {
            assert!(
                !content.contains(token),
                "{} must not introduce `{token}`; Nexus stays a contract and fixture harness layer here",
                source_file.display()
            );
        }
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn workspace_manifests(root: &Path) -> Vec<PathBuf> {
    let mut manifests = vec![root.join("Cargo.toml")];
    let crates_dir = root.join("crates");
    for entry in fs::read_dir(crates_dir).unwrap() {
        let manifest = entry.unwrap().path().join("Cargo.toml");
        if manifest.exists() {
            manifests.push(manifest);
        }
    }
    manifests.sort();
    manifests
}

fn crate_src_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    for entry in fs::read_dir(root.join("crates")).unwrap() {
        let src_dir = entry.unwrap().path().join("src");
        if src_dir.exists() {
            sources.extend(rust_sources(&src_dir));
        }
    }
    sources.sort();
    sources
}

fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
    sources.sort();
    sources
}

fn manifest_declares_dependency(content: &str, dependency: &str) -> bool {
    content.lines().any(|line| {
        let line = line.trim();
        line == dependency
            || line
                .strip_prefix(dependency)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
    })
}

fn package_name(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("name")
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .map(|value| value.trim().trim_matches('"').to_string())
    })
}
