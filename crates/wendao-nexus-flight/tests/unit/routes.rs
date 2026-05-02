use wendao_nexus_flight::{NexusFlightRoute, EXTERNAL_KNOWLEDGE_SEARCH_ROUTE};

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
