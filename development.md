# Development

## Format, Test, Lint

```shell
direnv exec . cargo fmt --all -- --check
direnv exec . cargo fetch --locked
direnv exec . cargo test --workspace --all-targets --all-features --locked
direnv exec . cargo test --workspace
direnv exec . cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
direnv exec . cargo test --workspace rust_project_harness_gate --locked -- --nocapture
direnv exec . git diff --check
```

Rust doc comments follow Clippy's `doc_markdown` style. Wrap API names,
literal identifiers, command names, and rule IDs in backticks rather than
leaving mixed-case technical terms as prose.

GitHub Actions lives in `.github/workflows/ci.yml` and mirrors this locked
workspace validation on Linux, macOS, and Windows. Ubuntu also runs formatting,
strict Clippy, the explicit harness policy gate, and whitespace checks.
The facade crate also carries a deterministic phase guard that rejects live
source clients, parser crates, storage/search/cache engines, CocoIndex, and
public local knowledge-store ownership from the current contract-only phase.
The facade crate's `tests/conformance/` suite is the pre-adapter contract gate
for routes, command envelopes, Arrow schemas and batches, directory-first source
packs, serverless fixture command handling, and artifact replay.

## Workspace Policy

`wendao-nexus` is an independent service workspace:

- `crates/wendao-nexus-core` owns stable domain contracts only.
- `crates/wendao-nexus-flight` owns Arrow Flight protocol contracts.
- `crates/wendao-nexus-connectors` owns source-specific adapters.
- `crates/wendao-nexus-runtime` owns sync orchestration and registry facades.

Keep `xiuxian-*` dependencies out of this repository. Xiuxian should consume
Nexus through Arrow Flight contracts, linked runtime/connectors, or through an
adapter that lives in `xiuxian-artisan-workshop`.

Do not add a standalone Nexus server crate here. Wendao already owns the server
surface; Nexus should provide the protocol, ingestion, connector, and runtime
pieces that the Wendao-side server mounts.

Keep storage backends out of the core contract crate. DuckDB, LanceDB, Valkey,
graph indexes, and gateway routes should attach through runtime traits, Flight
contracts, future storage crates, or downstream Wendao integration crates.
The current phase does not add those backend crates at all; fixture evidence
view tests are only a serverless proof for the Arrow Flight contract.

Each library crate carries `rust-lang-project-harness` as a dev-dependency and
mounts `rust_project_harness_source_gate!()` from `src/lib.rs`. The shared
source-backed gate in `tests/support/rust_harness.rs` runs the parser-native
policy gate with project configuration and asserts the upstream verification
profile fact index is fully configured for each crate.

## Documentation Policy

Documentation uses a path-first Johnny.Decimal layout. The project-local
topology manifest is `docs/topology.toml`, and the JDex is
`docs/00_index/00.01_jdex.md`.

Use `docs/AC_slug/AC.ID_semantic_name.md` for durable notes. The category prefix
in the directory and the file coordinate must agree. If a new category is
needed, propose a reviewable `docs/topology.toml` change rather than silently
inventing category meaning.

When the Wendao CLI is available, run the documentation audit commands in
`docs/90_operations/90.01_validation_and_governance.md` after editing docs.

## Test Shape

Unit tests live beside the modules they exercise while the workspace is still
small. Add integration tests once a cross-crate behavior needs a stable public
fixture or source-specific contract proof.
