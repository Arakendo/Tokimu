use network_tools::{
    decode_payload, encode_payload, JsonEnvelopeCodec, LoopbackTransport,
    ObservationSequenceTracker, ReplicationEnvelope, SchemaIdentity, Transport,
    DEFAULT_PAYLOAD_LIMIT, PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};

const SCHEMA_ID: &str = "tokimu.example.player-observation";
const SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct PlayerObservation {
    frame: u32,
    position: [f32; 3],
    score: u32,
    status: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = SchemaIdentity::new(SCHEMA_ID, SCHEMA_VERSION);
    let codec = JsonEnvelopeCodec::new(schema.clone());
    let observations = [
        (
            12,
            PlayerObservation {
                frame: 12,
                position: [4.0, 1.5, -2.0],
                score: 900,
                status: "observing".to_owned(),
            },
        ),
        (
            13,
            PlayerObservation {
                frame: 13,
                position: [4.5, 1.5, -2.0],
                score: 920,
                status: "observing".to_owned(),
            },
        ),
        (
            15,
            PlayerObservation {
                frame: 15,
                position: [5.0, 1.5, -2.0],
                score: 960,
                status: "gap demonstrated".to_owned(),
            },
        ),
        (
            15,
            PlayerObservation {
                frame: 15,
                position: [5.0, 1.5, -2.0],
                score: 960,
                status: "duplicate ignored".to_owned(),
            },
        ),
        (
            14,
            PlayerObservation {
                frame: 14,
                position: [4.75, 1.5, -2.0],
                score: 940,
                status: "late observation ignored".to_owned(),
            },
        ),
    ];

    let mut transport = LoopbackTransport::new(observations.len());
    let provider = transport.provider_name();
    for (sequence, observation) in &observations {
        let payload = encode_payload(observation, DEFAULT_PAYLOAD_LIMIT)?;
        let envelope = ReplicationEnvelope::observation(schema.clone(), *sequence, payload);
        transport.send(&codec.encode(&envelope)?)?;
    }

    let mut tracker = ObservationSequenceTracker::new();
    while let Some(received_frame) = transport.try_receive()? {
        let received_envelope = codec.decode(&received_frame)?;
        let received: PlayerObservation =
            decode_payload(&received_envelope.payload, DEFAULT_PAYLOAD_LIMIT)?;
        let decision = tracker.observe(&received_envelope);

        println!(
            "provider={provider} protocol={} schema={}@{} sequence={} bytes={} decision={decision:?}",
            received_envelope.protocol_version,
            decision.schema_id(),
            decision.schema_version(),
            received_envelope.sequence,
            received_frame.len(),
        );

        if decision.is_accepted() && received.frame != received_envelope.sequence as u32 {
            return Err(format!(
                "application validation failed: observation frame {} did not match sequence {}",
                received.frame, received_envelope.sequence
            )
            .into());
        }
    }

    match codec.decode(b"{malformed") {
        Ok(_) => return Err("malformed frame unexpectedly decoded".into()),
        Err(error) => println!("stage=decode diagnostic={error}"),
    }

    assert_eq!(tracker.last_accepted(), Some(15));
    assert_eq!(PROTOCOL_VERSION, 1);
    Ok(())
}
