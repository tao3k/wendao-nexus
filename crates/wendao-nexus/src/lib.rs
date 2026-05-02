//! Facade adapters that compose `Wendao Nexus` runtime and Flight contracts.
//!
//! This crate does not own a server. It provides small composition pieces that
//! Wendao-side server code can mount.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_source_gate!(
    "../../../tests/support/rust_harness.rs"
);

mod local_mirror_handler;

pub use local_mirror_handler::LocalMirrorFlightHandler;
