use serde::{Deserialize, Serialize};

use crate::{
    decode_payload, encode_payload, CodecError, InjectedFailure, JsonEnvelopeCodec,
    LoopbackTransport, MessageKind, ObservationSequenceTracker, ReplicationEnvelope,
    SchemaIdentity, SequenceDecision, Transport, TransportError, PROTOCOL_VERSION,
};

const PAYLOAD_LIMIT: usize = 256;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Observation {
    frame: u32,
    position: [f32; 3],
    score: u32,
}

fn schema() -> SchemaIdentity {
    SchemaIdentity::new("tokimu.example.observation", 1)
}

fn envelope(sequence: u64) -> ReplicationEnvelope {
    let payload = encode_payload(
        &Observation {
            frame: sequence as u32,
            position: [1.0, 2.0, 3.0],
            score: 7,
        },
        PAYLOAD_LIMIT,
    )
    .expect("fixture payload should encode");
    ReplicationEnvelope::observation(schema(), sequence, payload)
}

#[test]
fn envelope_and_payload_round_trip_exactly() {
    let codec = JsonEnvelopeCodec::new(schema());
    let sent = envelope(42);
    let frame = codec.encode(&sent).expect("envelope should encode");
    let received = codec.decode(&frame).expect("envelope should decode");
    let payload: Observation =
        decode_payload(&received.payload, PAYLOAD_LIMIT).expect("payload should decode");

    assert_eq!(received, sent);
    assert_eq!(payload.frame, 42);
    assert_eq!(payload.position, [1.0, 2.0, 3.0]);
    assert_eq!(payload.score, 7);
}

#[test]
fn codec_rejects_protocol_schema_kind_and_malformed_frames() {
    let codec = JsonEnvelopeCodec::new(schema());
    let mut wrong_protocol = envelope(1);
    wrong_protocol.protocol_version = PROTOCOL_VERSION + 1;
    assert!(matches!(
        codec.encode(&wrong_protocol),
        Err(CodecError::UnsupportedProtocolVersion { .. })
    ));

    let wrong_schema = JsonEnvelopeCodec::new(SchemaIdentity::new("other", 1));
    let frame = codec.encode(&envelope(1)).expect("fixture should encode");
    assert!(matches!(
        wrong_schema.decode(&frame),
        Err(CodecError::UnsupportedSchema { .. })
    ));

    let wrong_schema_version =
        JsonEnvelopeCodec::new(SchemaIdentity::new("tokimu.example.observation", 2));
    assert!(matches!(
        wrong_schema_version.decode(&frame),
        Err(CodecError::UnsupportedSchema {
            expected_version: 2,
            found_version: 1,
            ..
        })
    ));

    let unknown_kind = frame
        .windows(b"observation_snapshot".len())
        .position(|window| window == b"observation_snapshot")
        .map(|start| {
            let mut changed = frame.clone();
            changed.splice(
                start..start + b"observation_snapshot".len(),
                b"unknown_kind".iter().copied(),
            );
            changed
        })
        .expect("wire fixture should contain message kind");
    assert!(matches!(
        codec.decode(&unknown_kind),
        Err(CodecError::UnknownMessageKind(kind)) if kind == "unknown_kind"
    ));

    assert!(matches!(
        codec.decode(b"{not-json"),
        Err(CodecError::MalformedEnvelope(_))
    ));
}

#[test]
fn codec_enforces_payload_and_frame_limits() {
    let strict_payload = JsonEnvelopeCodec::new(schema()).with_limits(1024, 2);
    assert!(matches!(
        strict_payload.encode(&envelope(1)),
        Err(CodecError::PayloadTooLarge { .. })
    ));

    let strict_frame = JsonEnvelopeCodec::new(schema()).with_limits(8, 1024);
    assert!(matches!(
        strict_frame.encode(&envelope(1)),
        Err(CodecError::FrameTooLarge { .. })
    ));
    assert!(matches!(
        strict_frame.decode(&[0; 9]),
        Err(CodecError::FrameTooLarge { .. })
    ));
}

#[test]
fn loopback_preserves_frames_and_fifo_order() {
    let mut transport = LoopbackTransport::new(2);
    transport.send(b"first").expect("first frame should send");
    transport.send(b"second").expect("second frame should send");

    assert_eq!(transport.pending_frames(), 2);
    assert_eq!(
        transport.try_receive().expect("receive should work"),
        Some(b"first".to_vec())
    );
    assert_eq!(
        transport.try_receive().expect("receive should work"),
        Some(b"second".to_vec())
    );
    assert_eq!(
        transport.try_receive().expect("empty receive should work"),
        None
    );
}

#[test]
fn loopback_reports_limits_closure_and_injected_failures() {
    let mut full = LoopbackTransport::new(1);
    full.send(b"one").expect("first frame should send");
    assert_eq!(
        full.send(b"two"),
        Err(TransportError::QueueFull { capacity: 1 })
    );

    let mut limited = LoopbackTransport::new(1).with_frame_limit(3);
    assert!(matches!(
        limited.send(b"four"),
        Err(TransportError::FrameTooLarge { .. })
    ));

    let mut injected = LoopbackTransport::new(1);
    injected.inject_failure(InjectedFailure::Send);
    assert_eq!(
        injected.send(b"frame"),
        Err(TransportError::Injected { operation: "send" })
    );
    injected.inject_failure(InjectedFailure::Receive);
    assert_eq!(
        injected.try_receive(),
        Err(TransportError::Injected {
            operation: "receive"
        })
    );

    let mut closed = LoopbackTransport::new(1);
    closed.close();
    assert_eq!(closed.send(b"frame"), Err(TransportError::Closed));
    assert_eq!(closed.try_receive(), Err(TransportError::Closed));
}

#[test]
fn loopback_instances_do_not_share_state() {
    let mut first = LoopbackTransport::new(1);
    let mut second = LoopbackTransport::new(1);

    first.send(b"first-only").expect("frame should send");

    assert_eq!(
        second
            .try_receive()
            .expect("independent empty receive should work"),
        None
    );
    assert_eq!(
        first.try_receive().expect("first receive should work"),
        Some(b"first-only".to_vec())
    );

    first.close();
    assert_eq!(
        second
            .try_receive()
            .expect("closing first must not close second"),
        None
    );
}

#[test]
fn message_kind_is_application_neutral() {
    assert_eq!(envelope(1).message_kind, MessageKind::ObservationSnapshot);

    let input = ReplicationEnvelope::client_input(schema(), 2, b"input".to_vec());
    assert_eq!(input.message_kind, MessageKind::ClientInput);
}

#[test]
fn sequence_tracker_accepts_first_and_ordered_observations() {
    let mut tracker = ObservationSequenceTracker::new();

    assert!(matches!(
        tracker.observe(&envelope(7)),
        SequenceDecision::AcceptedFirst { received: 7, .. }
    ));
    assert!(matches!(
        tracker.observe(&envelope(8)),
        SequenceDecision::AcceptedInOrder { received: 8, .. }
    ));
    assert_eq!(tracker.last_accepted(), Some(8));
}

#[test]
fn sequence_tracker_reports_gaps_and_ignores_duplicate_or_late_frames() {
    let mut tracker = ObservationSequenceTracker::new();
    tracker.observe(&envelope(10));

    assert_eq!(
        tracker.observe(&envelope(12)),
        SequenceDecision::AcceptedWithGap {
            schema_id: "tokimu.example.observation".to_owned(),
            schema_version: 1,
            expected: 11,
            received: 12,
        }
    );
    assert_eq!(
        tracker.observe(&envelope(12)),
        SequenceDecision::IgnoredDuplicate {
            schema_id: "tokimu.example.observation".to_owned(),
            schema_version: 1,
            received: 12,
            last_accepted: 12,
        }
    );
    assert_eq!(
        tracker.observe(&envelope(11)),
        SequenceDecision::IgnoredStaleOrOutOfOrder {
            schema_id: "tokimu.example.observation".to_owned(),
            schema_version: 1,
            received: 11,
            last_accepted: 12,
        }
    );
    assert_eq!(tracker.last_accepted(), Some(12));
}

#[test]
fn sequence_tracker_keeps_the_maximum_value_monotonic() {
    let mut tracker = ObservationSequenceTracker::new();
    tracker.observe(&envelope(u64::MAX));

    assert!(matches!(
        tracker.observe(&envelope(0)),
        SequenceDecision::IgnoredStaleOrOutOfOrder {
            received: 0,
            last_accepted: u64::MAX,
            ..
        }
    ));
    assert!(matches!(
        tracker.observe(&envelope(u64::MAX)),
        SequenceDecision::IgnoredDuplicate {
            received: u64::MAX,
            ..
        }
    ));
}
