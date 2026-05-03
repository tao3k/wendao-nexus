//! Facade adapters that compose `Wendao Nexus` runtime and Flight contracts.
//!
//! This crate does not own a server. It provides small composition pieces that
//! Wendao-side server code can mount.

mod fixture_harness;

pub use fixture_harness::{FixtureIngestReport, FixtureSourceIngestReport, NexusFixtureHarness};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);
