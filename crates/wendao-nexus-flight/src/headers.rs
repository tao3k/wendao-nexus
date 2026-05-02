//! Metadata header constants for `Wendao Nexus` Flight calls.

/// Header that carries the requested Nexus route.
pub const NEXUS_ROUTE_HEADER: &str = "x-wendao-nexus-route";

/// Header that carries an optional source id filter.
pub const NEXUS_SOURCE_ID_HEADER: &str = "x-wendao-nexus-source-id";

/// Header that carries the minimum requested authority level.
pub const NEXUS_AUTHORITY_MIN_HEADER: &str = "x-wendao-nexus-authority-min";

/// Header that carries a compact provenance summary for agent-facing consumers.
pub const NEXUS_PROVENANCE_SUMMARY_HEADER: &str = "x-wendao-nexus-provenance-summary";
