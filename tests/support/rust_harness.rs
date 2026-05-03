use std::path::Path;

use rust_lang_project_harness::{
    RustHarnessConfig, RustOwnerResponsibility, RustVerificationDependencySignal,
    RustVerificationProfileHint, build_rust_verification_profile_index_with_config,
    default_rust_harness_config, render_rust_project_harness, render_rust_verification_profile_index,
    run_rust_project_harness_with_config,
};

#[test]
fn enforce_rust_project_harness_gate() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = wendao_nexus_harness_config();
    let report =
        run_rust_project_harness_with_config(project_root, &config).expect("harness run");

    assert!(report.is_clean(), "{}", render_rust_project_harness(&report));
}

#[test]
fn verification_profile_facts_are_configured() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = wendao_nexus_harness_config();
    let index = build_rust_verification_profile_index_with_config(project_root, &config)
        .expect("verification profile index");

    assert!(
        index.is_clear(),
        "{}",
        render_rust_verification_profile_index(&index)
    );
}

fn wendao_nexus_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/customer_corpus.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("customer corpus connector exposes source-specific adapter contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/external_database.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale(
                "external database connector exposes source identity, endpoint, auth, and capability contracts",
            ),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/local_corpus.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::Persistence,
                ],
            )
            .with_rationale(
                "local corpus connector reads deterministic fixture files for source-pack validation",
            ),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/pubmed.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("PubMed connector exposes source-specific adapter contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/source_pack.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::Persistence,
                ],
            )
            .with_rationale(
                "source pack loader parses deterministic manifest files into source records and local corpus connectors",
            ),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/static_connector.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("static connector backs deterministic runtime and protocol tests"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/wikipedia.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("Wikipedia connector exposes source-specific adapter contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/authority.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("authority policy types are part of the public evidence contract"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/connector.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("connector trait binds public source contracts to external adapters"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/document.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("normalized document types are public wire-domain contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/error.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("error types are public API surface for all Nexus crates"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/query.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale(
                "query request and evidence response types are public protocol contracts",
            ),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/source.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("source identity and checkpoint types are public contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/sync.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("sync job records are public runtime-independent contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/trust.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("provenance and evidence boundaries are public trust contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/batch.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("Flight batch builders bind public Arrow and Nexus contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/command.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::AvailabilityCritical,
                ],
            )
            .with_rationale("Flight command envelopes are public route dispatch contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/headers.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("Flight metadata headers are public wire constants"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/provider.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::AvailabilityCritical,
                ],
            )
            .with_rationale("Flight provider dispatches public commands into Arrow batches"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/routes.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::AvailabilityCritical,
                ],
            )
            .with_rationale("Flight routes and tickets are public wire identity"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/schema.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("Arrow schemas are public batch contracts"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/hash.rs", [RustOwnerResponsibility::PublicApi])
                .with_rationale("content hash helpers are public dedup contract support"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/artifact.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::AvailabilityCritical,
                ],
            )
            .with_rationale(
                "artifact store persists raw source payloads and normalized evidence sidecars",
            ),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/normalize.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("normalizers map raw source payloads into public evidence documents"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/registry.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("registry traits define runtime state persistence boundaries"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/runtime.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::ExternalDependency,
                ],
            )
            .with_rationale("sync runtime orchestrates connectors, registries, and normalized evidence handoff"),
        )
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "arrow-array",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::PublicApi,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "arrow-flight",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::PublicApi,
                RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "arrow-schema",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::PublicApi,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "async-trait",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "chrono",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "serde",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "serde_json",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "sha2",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "tokio",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "toml",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::Persistence,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "thiserror",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "tonic",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::PublicApi,
                RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "url",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "uuid",
            [RustOwnerResponsibility::PublicApi],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "wendao-nexus-core",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::PublicApi,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "wendao-nexus-flight",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::PublicApi,
                RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_dependency_signal(RustVerificationDependencySignal::new(
            "wendao-nexus-runtime",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
}
