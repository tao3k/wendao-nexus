# Wendao Nexus

Wendao Nexus is the independent external knowledge source ingestion and trust
registry service for Wendao.

It is not a generic web crawler, not an LLM answer authority, not a document
parser, and not a `xiuxian-*` internal crate. Its job is to define the
external-knowledge contract, preserve source identity and provenance, and
expose a clean Arrow Flight boundary that Wendao or other consumers can call.

## Parser-Free Contract Boundary

Nexus does not parse PDFs, HTML, Markdown, Docling output, database rows, or
binary attachments into knowledge. Wendao owns parser execution, Docling
orchestration, scheduling, agent routing, and backend indexing. Wendao-side code
can turn external databases or documents into normalized Markdown, sections,
metadata, and provenance, then hand those records to Nexus through
`RawSourceDocument`, `ExternalKnowledgeDocument`, source-pack manifests, and
Arrow/Flight batches.

The Nexus contract intentionally includes metadata such as authors, license,
DOI, PMID, publication/update time, version, revision id, source kind, authority
level, provenance, and evidence kind. It also names the first vertical business
fields that source packs need, including legal jurisdiction/statute/article and
agriculture region/crop/market-signal fields. That lets agents and LLM-facing
tools receive an evidence boundary without making Nexus a parser framework.

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

Facade crate:

- serverless fixture Flight harness tests for source-pack runtime and Arrow
  batch validation;
- package-level harness coverage for composing runtime and protocol crates.

This crate does not own a server process, local knowledge store, memory engine,
or search/open implementation. Wendao-side code mounts `wendao-nexus-flight`
providers by implementing `NexusFlightCommandHandler`.

### `wendao-nexus-core`

Core contracts and domain model:

- source identity, kind, capability, cursor, and checkpoint types;
- document, section, citation, license, and raw source payload contracts;
- provenance and authority-level records;
- extracted document resource contracts for attachment/document extraction
  adapters;
- connector trait;
- agent-facing command request and evidence response contracts;
- sync job status and job record types.

This crate intentionally avoids Arrow, storage, network clients, and Wendao
server runtime orchestration. It also does not run Docling, parse documents, or
schedule attachment parsing; those belong to the Rust Wendao server side.

### `wendao-nexus-flight`

Public Arrow Flight protocol:

- canonical route constants;
- metadata header constants;
- Arrow schemas for search, open, sync, status, and compare batches;
- typed command envelopes for `FlightDescriptor::cmd`, including required
  command `schema_version = 1`;
- Arrow batch builders and a thin command handler provider for Wendao-side
  routing.

This crate is safe for external consumers because it carries protocol contracts,
not xiuxian internals.

### `wendao-nexus-runtime`

Sync orchestration:

- job registry facade;
- source registry facade;
- checkpoint registry facade;
- content hash dedup registry;
- artifact store facade plus local filesystem backend for raw payload and
  normalized document sidecars;
- raw-to-normalized document contract boundary for deterministic fixtures;
- source sync runtime;
- normalized-document handoff for Wendao-side parsed evidence, with
  provenance/hash consistency checks;
- deterministic in-memory registry for tests and early embedding.

Wendao-side persistent databases, indexing, vector search, lexical search, graph
expansion, hot cache, and rerank engines remain outside this library. Nexus
runtime state is only the trait surface, deterministic fixture validation,
replay artifacts, and normalized evidence handoff. `PlainTextNormalizer` is a
fixture/test shim over already-text payloads, not a replacement for Wendao
parsers or Docling.

### `wendao-nexus-connectors`

Source adapters:

- source-specific capability declarations;
- external database/API-feed source identity, endpoint, auth-mode, and
  planned access-mode metadata;
- source-specific fetch/discover/delta behavior;
- connector configs and test fixtures.

The first skeleton includes Wikipedia, PubMed, customer private corpus, generic
external database/API-feed, and deterministic static connector boundaries. It
also includes a file-backed `LocalCorpusConnector` for JSONL and Markdown
fixture corpora so source-pack behavior can be validated before live APIs. That
connector is fixture-only: its minimal frontmatter handling is not a production
Markdown parser and must not grow into document extraction ownership.
`SourcePack` loads TOML/JSON manifests that group multiple local corpus sources
with source kind, authority, schema version, producer, version, display name,
and license metadata, and can emit source catalog records for the Wendao-side
registry boundary. Live API clients are deliberately left as explicit
unsupported paths until source-specific rate limit, auth, and contract tests are
added.
Unsupported live stubs do not advertise executable discover/fetch/delta or live
query capabilities. Current phase guards reject direct live-client dependencies
and connector-source imports for live clients, while source-pack manifests
reject unsupported schema versions, empty or whitespace-padded registry
identities, and duplicate source ids. Disabled source-pack entries stay visible
as source catalog records but do not create local corpus connectors.

## Flight Routes

```text
/knowledge/external/search
/knowledge/external/open
/knowledge/external/sync
/knowledge/external/status
/knowledge/external/compare
```

Arrow and Arrow Flight are the priority data boundary. Nexus schemas carry
route and schema-version metadata, and search batches reserve nullable extension
columns for section identity, heading hierarchy, source kind, publication and
source update times, downstream score fields, license, metadata, DOI, PMID,
jurisdiction, and evidence kind. Nexus does not compute vector, lexical, graph,
or rerank results; Wendao-side backends may populate those fields when they
mount the provider.

## Current Engineering Sequence

Nexus should prove its own deterministic protocol runtime before real external
sources or Wendao integration are added. The current sequence is:

1. Contract snapshots for routes, headers, command envelopes, Arrow schemas, and
   batch builders.
2. Static and local corpus connectors over deterministic JSONL/Markdown
   fixtures.
3. Source pack TOML/JSON manifests and source catalog records for governed
   fixture packs.
4. Registry traits, normalized-document handoff, and artifact store.
5. Serverless fixture Flight harness/client validation.
6. External database/API connector contracts.
7. Wendao-side adapter or mount wiring.

Live PubMed, Wikipedia, legal, news, paid database, and Wendao adapter work
should wait until the deterministic connector/runtime/schema path is repeatably
validated. Current crates intentionally carry no HTTP client dependency.

## Business Scenario First

The first commercial fixtures are vertical source packs:

- medical baseline evidence with PubMed-style article metadata, a clinical
  guideline fixture, license metadata, and non-generic `evidence_kind` values;
- customer private SOP evidence with tenant, department, ACL tags, version,
  customer-confidential license policy, and `CustomerInternal` authority;
- legal compliance clauses with jurisdiction, statute, article, effective date,
  amendment version, and `law_clause` evidence kind;
- agriculture market signals with region, crop, price date, weather window,
  supply signal, and `market_signal` evidence kind.

Nexus keeps these as deterministic local corpus input so business-critical
knowledge paths can be validated before any live customer database, CRM,
document system, external market API, CocoIndex exporter, or Wendao adapter is
added.
Positive business packs use directory-first fixtures with `source_pack.toml`,
local JSONL documents, and golden text snapshots for search/open/status-shaped
contract rows. Flat manifests remain for negative validation cases.

## CocoIndex Positioning

CocoIndex is a useful reference for general incremental dataflow and can become
a future optional SourcePack exporter. It is not a Nexus core dependency in this
repository. Any future bridge should output Nexus-owned source-pack fixtures or
artifacts and leave authority, provenance, evidence kind, and Arrow Flight
evidence boundaries under Nexus contracts.

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
