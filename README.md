# Wendao Nexus

Wendao Nexus is the independent external knowledge source ingestion and trust
registry service for Wendao.

It is not a generic web crawler, not an LLM answer authority, and not a
`xiuxian-*` internal crate. Its job is to connect external authoritative
sources, normalize their evidence, preserve provenance, and expose a clean
Arrow Flight boundary that Wendao or other consumers can call.

## Workspace

```text
crates/wendao-nexus
crates/wendao-nexus-core
crates/wendao-nexus-flight
crates/wendao-nexus-runtime
crates/wendao-nexus-connectors
```

## Documentation

Project documentation uses a local Johnny.Decimal topology:

- `docs/topology.toml` declares the category catalog.
- `docs/00_index/00.01_jdex.md` is the documentation index.
- `docs/10_architecture/10.01_nexus_boundary.md` records the standalone Nexus
  boundary.
- `docs/10_architecture/10.02_flight_protocol.md` records the Flight contract.
- `docs/20_runtime/20.01_runtime_and_connectors.md` records runtime and
  connector ownership.
- `docs/90_operations/90.01_validation_and_governance.md` records validation and
  audit rules.

## Dependency Direction

The boundary is intentionally one-way:

```text
wendao-nexus independent protocol and ingestion crates
  -> linked or mounted by the Wendao-side server
xiuxian-wendao / Wendao Flight service / agent tools
```

`wendao-nexus` must not depend on `xiuxian-db-store`, `xiuxian-vector`,
`xiuxian-wendao-core`, or `xiuxian-wendao-runtime`. If xiuxian needs a Rust-level
integration, that adapter belongs in `xiuxian-artisan-workshop` or the Wendao
server crate, where it can depend on Nexus protocol crates and xiuxian internal
crates in the legal direction.

## Crate Boundaries

### `wendao-nexus`

Facade adapters:

- local mirror Flight handler;
- composition glue across runtime and protocol crates;
- mountable pieces for the Wendao-side server.

This crate does not own a server process.

### `wendao-nexus-core`

Core contracts and domain model:

- source identity, kind, capability, cursor, and checkpoint types;
- document, section, citation, license, and raw source payload contracts;
- provenance and authority-level records;
- extracted document resource contracts for attachment/document extraction
  adapters;
- connector trait;
- agent-facing query request and evidence response contracts;
- sync job status and job record types.

This crate intentionally avoids Arrow, storage, network clients, and Wendao
server runtime orchestration. It also does not run Docling or schedule
attachment parsing; those belong to the Rust Wendao server side.

### `wendao-nexus-flight`

Public Arrow Flight protocol:

- canonical route constants;
- metadata header constants;
- Arrow schemas for search, open, sync, status, and compare batches;
- typed command envelopes for `FlightDescriptor::cmd`;
- Arrow batch builders and a thin command handler provider for Wendao-side
  routing.

This crate is safe for external consumers because it carries protocol contracts,
not xiuxian internals.

### `wendao-nexus-runtime`

Sync orchestration:

- job registry facade;
- checkpoint registry facade;
- content hash dedup registry;
- raw-to-normalized document normalization boundary;
- local mirror query facade for normalized documents;
- source sync runtime;
- normalized-document handoff for Wendao-side parsed evidence, with
  provenance/hash consistency checks;
- deterministic in-memory registry and knowledge store for tests and early
  embedding.

Durable backends such as DuckDB, Arrow, LanceDB, Valkey, and graph indexes should
plug in here or in future storage crates, not into the core contract crate.

### `wendao-nexus-connectors`

Source adapters:

- source-specific capability declarations;
- source-specific fetch/discover/delta behavior;
- connector configs and test fixtures.

The first skeleton includes Wikipedia, PubMed, customer private corpus, and a
deterministic static connector. Live API clients are deliberately left as
explicit unsupported paths until source-specific rate limit, auth, and contract
tests are added.

## Flight Routes

```text
/knowledge/external/search
/knowledge/external/open
/knowledge/external/sync
/knowledge/external/status
/knowledge/external/compare
```

## Evidence Boundary

Nexus records should preserve:

- where a claim came from;
- when it was fetched;
- source update or revision metadata when available;
- identifiers such as DOI, PMID, jurisdiction, or revision id;
- license and authority level;
- content hash for dedup and recovery.

LLM output remains subordinate to governed sources. Authority-sensitive flows
should return evidence records and provenance bundles rather than prose-only
answers.

## Validation

```shell
direnv exec . cargo fmt --all -- --check
direnv exec . cargo test --workspace
direnv exec . cargo clippy --workspace --all-targets --all-features -- -D warnings
direnv exec . git diff --check
```

When the Wendao CLI is available, also run the Johnny.Decimal documentation
audit described in `docs/90_operations/90.01_validation_and_governance.md`.
