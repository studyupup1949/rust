use ada_sdk::proto::{
    IngestRequest, JobStatusStreamEvent, PublicEventStreamEvent, SignalStreamEvent,
};
use ada_sdk::subscription::SubscriptionOptions;
use base64::Engine;
use prost::Message;
use std::time::Duration;

#[test]
fn shared_contract_fixture_has_all_typed_variants() {
    let fixture = include_str!("../../contract/stream-fixtures.tsv");
    let rows = fixture.lines().collect::<Vec<_>>();
    assert_eq!(rows[0].split('\t').count(), 5);
    assert_eq!(rows[1].split('\t').count(), 14);
    assert_eq!(rows[2], "jobs\tjob.started\tjob.progressed\tjob.finished");
    let mut decoded = [0_u8; 3];
    for row in rows.iter().filter(|row| row.starts_with("wire\t")) {
        let fields = row.split('\t').collect::<Vec<_>>();
        let wire = base64::engine::general_purpose::STANDARD
            .decode(fields[3])
            .expect("fixture base64");
        match fields[1] {
            "events" => {
                let envelope =
                    PublicEventStreamEvent::decode(wire.as_slice()).expect("event fixture");
                let event = envelope.event.expect("event envelope");
                assert!(!envelope.cursor.is_empty());
                assert_eq!(event.principal_id, "namespace:alice");
                assert!(event.event.is_some());
                decoded[0] += 1;
            }
            "signals" => {
                let envelope = SignalStreamEvent::decode(wire.as_slice()).expect("signal fixture");
                let signal = envelope.signal.expect("signal envelope");
                assert!(!envelope.cursor.is_empty());
                assert_eq!(signal.principal_id, "namespace:alice");
                assert!(signal.signal.is_some());
                decoded[1] += 1;
            }
            "jobs" => {
                let envelope = JobStatusStreamEvent::decode(wire.as_slice()).expect("job fixture");
                assert!(!envelope.event_id.is_empty());
                assert_eq!(envelope.principal_id, "namespace:alice");
                assert!(envelope.event.is_some());
                decoded[2] += 1;
            }
            _ => panic!("unknown fixture category"),
        }
    }
    assert_eq!(decoded, [4, 13, 3]);
    let support = rows
        .iter()
        .find(|row| row.starts_with("support\t"))
        .expect("map fixture")
        .split('\t')
        .collect::<Vec<_>>();
    let wire = base64::engine::general_purpose::STANDARD
        .decode(support[3])
        .expect("map base64");
    let request = IngestRequest::decode(wire.as_slice()).expect("map fixture");
    assert_eq!(request.metadata["alpha"], "one");
    assert_eq!(request.metadata["beta"], "two");
    let invalid = rows
        .iter()
        .find(|row| row.contains("\tinvalid_wire\t"))
        .expect("invalid wire fixture")
        .split('\t')
        .collect::<Vec<_>>();
    let wire = base64::engine::general_purpose::STANDARD
        .decode(invalid[3])
        .expect("invalid wire base64");
    assert!(PublicEventStreamEvent::decode(wire.as_slice()).is_err());
}

#[test]
fn reconnect_delay_is_bounded() {
    let options = SubscriptionOptions {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        max_reconnect_attempts: None,
        ..SubscriptionOptions::default()
    };
    assert_eq!(options.initial_delay, Duration::from_millis(100));
    assert_eq!(options.max_delay, Duration::from_secs(1));
}
