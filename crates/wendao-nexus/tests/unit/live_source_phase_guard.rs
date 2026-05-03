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
