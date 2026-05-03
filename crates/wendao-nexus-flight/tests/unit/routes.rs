use wendao_nexus_flight::{
    EXTERNAL_KNOWLEDGE_COMPARE_ROUTE, EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
    EXTERNAL_KNOWLEDGE_SEARCH_ROUTE, EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
    EXTERNAL_KNOWLEDGE_SYNC_ROUTE, NEXUS_AUTHORITY_MIN_HEADER, NEXUS_PROVENANCE_SUMMARY_HEADER,
    NEXUS_ROUTE_HEADER, NEXUS_SOURCE_ID_HEADER, NexusFlightRoute,
};

#[test]
fn route_round_trips_from_canonical_path() {
    let route = NexusFlightRoute::try_from(EXTERNAL_KNOWLEDGE_SEARCH_ROUTE).unwrap();

    assert_eq!(route, NexusFlightRoute::Search);
    assert_eq!(route.as_str(), EXTERNAL_KNOWLEDGE_SEARCH_ROUTE);
}

#[test]
fn route_ticket_uses_canonical_path_bytes() {
    let ticket = NexusFlightRoute::Search.ticket();

    assert_eq!(
        ticket.ticket.as_ref(),
        EXTERNAL_KNOWLEDGE_SEARCH_ROUTE.as_bytes()
    );
}

#[test]
fn routes_match_snapshot() {
    let routes = NexusFlightRoute::all()
        .into_iter()
        .map(NexusFlightRoute::as_str)
        .collect::<Vec<_>>();

    assert_eq!(
        routes,
        vec![
            EXTERNAL_KNOWLEDGE_SEARCH_ROUTE,
            EXTERNAL_KNOWLEDGE_OPEN_ROUTE,
            EXTERNAL_KNOWLEDGE_SYNC_ROUTE,
            EXTERNAL_KNOWLEDGE_STATUS_ROUTE,
            EXTERNAL_KNOWLEDGE_COMPARE_ROUTE,
        ]
    );
}

#[test]
fn metadata_headers_match_snapshot() {
    assert_eq!(
        [
            NEXUS_ROUTE_HEADER,
            NEXUS_SOURCE_ID_HEADER,
            NEXUS_AUTHORITY_MIN_HEADER,
            NEXUS_PROVENANCE_SUMMARY_HEADER,
        ],
        [
            "x-wendao-nexus-route",
            "x-wendao-nexus-source-id",
            "x-wendao-nexus-authority-min",
            "x-wendao-nexus-provenance-summary",
        ]
    );
}
