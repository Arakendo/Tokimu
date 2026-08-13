use network_tools::{
    decode_payload, encode_payload, JsonEnvelopeCodec, LoopbackTransport, MessageKind,
    ObservationSequenceTracker, ReplicationEnvelope, SchemaIdentity, SequenceDecision, Transport,
    DEFAULT_PAYLOAD_LIMIT,
};
use serde::{Deserialize, Serialize};

const CLIENT_COUNT: usize = 2;
const INPUT_SCHEMA_ID: &str = "tokimu.example.client-input";
const SNAPSHOT_SCHEMA_ID: &str = "tokimu.example.server-observation";
const SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct ClientInput {
    client_id: u8,
    tick: u32,
    move_x: i8,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct PlayerObservation {
    client_id: u8,
    position_x: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct ServerObservation {
    tick: u32,
    players: Vec<PlayerObservation>,
}

struct SimulatedClient {
    id: u8,
    input_transport: LoopbackTransport,
    observation_transport: LoopbackTransport,
    observation_tracker: ObservationSequenceTracker,
    latest_observation: Option<ServerObservation>,
}

impl SimulatedClient {
    fn new(id: u8) -> Self {
        Self {
            id,
            input_transport: LoopbackTransport::new(8),
            observation_transport: LoopbackTransport::new(8),
            observation_tracker: ObservationSequenceTracker::new(),
            latest_observation: None,
        }
    }

    fn send_input(
        &mut self,
        codec: &JsonEnvelopeCodec,
        sequence: u64,
        tick: u32,
        move_x: i8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = encode_payload(
            &ClientInput {
                client_id: self.id,
                tick,
                move_x,
            },
            DEFAULT_PAYLOAD_LIMIT,
        )?;
        let envelope = ReplicationEnvelope::client_input(
            SchemaIdentity::new(INPUT_SCHEMA_ID, SCHEMA_VERSION),
            sequence,
            payload,
        );
        self.input_transport.send(&codec.encode(&envelope)?)?;
        Ok(())
    }

    fn receive_observation(
        &mut self,
        codec: &JsonEnvelopeCodec,
    ) -> Result<Option<SequenceDecision>, Box<dyn std::error::Error>> {
        let Some(frame) = self.observation_transport.try_receive()? else {
            return Ok(None);
        };
        let envelope = codec.decode(&frame)?;
        if envelope.message_kind != MessageKind::ObservationSnapshot {
            return Err("client rejected non-observation server message".into());
        }
        let observation: ServerObservation =
            decode_payload(&envelope.payload, DEFAULT_PAYLOAD_LIMIT)?;
        if observation.tick != envelope.sequence as u32 {
            return Err("client rejected observation tick that did not match its sequence".into());
        }
        let decision = self.observation_tracker.observe(&envelope);
        if decision.is_accepted() {
            self.latest_observation = Some(observation);
        }
        Ok(Some(decision))
    }
}

struct AuthoritativeServer {
    tick: u32,
    positions: [i32; CLIENT_COUNT],
    last_input_sequence: [Option<u64>; CLIENT_COUNT],
}

impl AuthoritativeServer {
    fn receive_input(
        &mut self,
        client: &mut SimulatedClient,
        codec: &JsonEnvelopeCodec,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(frame) = client.input_transport.try_receive()? {
            let envelope = codec.decode(&frame)?;
            if envelope.message_kind != MessageKind::ClientInput {
                return Err("server rejected non-input client message".into());
            }
            let input: ClientInput = decode_payload(&envelope.payload, DEFAULT_PAYLOAD_LIMIT)?;
            let index = usize::from(client.id);
            if input.client_id != client.id || input.tick != self.tick + 1 {
                return Err("server rejected input with an invalid client or tick".into());
            }
            if !(-1..=1).contains(&input.move_x) {
                return Err("server rejected input outside the allowed movement range".into());
            }
            if self.last_input_sequence[index].is_some_and(|last| envelope.sequence <= last) {
                return Err("server rejected duplicate or stale client input".into());
            }
            self.last_input_sequence[index] = Some(envelope.sequence);
            self.positions[index] += i32::from(input.move_x);
        }
        Ok(())
    }

    fn observation(&mut self) -> ServerObservation {
        self.tick += 1;
        ServerObservation {
            tick: self.tick,
            players: self
                .positions
                .iter()
                .enumerate()
                .map(|(index, &position_x)| PlayerObservation {
                    client_id: index as u8,
                    position_x,
                })
                .collect(),
        }
    }

    fn publish_observation(
        &self,
        clients: &mut [SimulatedClient],
        codec: &JsonEnvelopeCodec,
        observation: &ServerObservation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = encode_payload(observation, DEFAULT_PAYLOAD_LIMIT)?;
        let envelope = ReplicationEnvelope::observation(
            SchemaIdentity::new(SNAPSHOT_SCHEMA_ID, SCHEMA_VERSION),
            u64::from(observation.tick),
            payload,
        );
        let frame = codec.encode(&envelope)?;
        for client in clients {
            client.observation_transport.send(&frame)?;
        }
        Ok(())
    }
}

fn run_simulation() -> Result<[i32; CLIENT_COUNT], Box<dyn std::error::Error>> {
    let input_codec = JsonEnvelopeCodec::new(SchemaIdentity::new(INPUT_SCHEMA_ID, SCHEMA_VERSION));
    let snapshot_codec =
        JsonEnvelopeCodec::new(SchemaIdentity::new(SNAPSHOT_SCHEMA_ID, SCHEMA_VERSION));
    let mut server = AuthoritativeServer {
        tick: 0,
        positions: [0; CLIENT_COUNT],
        last_input_sequence: [None; CLIENT_COUNT],
    };
    let mut clients = [SimulatedClient::new(0), SimulatedClient::new(1)];

    let input_sets = [[1, 1, 1], [-1, 0, -1]];
    for tick in 1..=3 {
        for (index, client) in clients.iter_mut().enumerate() {
            client.send_input(
                &input_codec,
                u64::from(tick),
                tick,
                input_sets[index][tick as usize - 1],
            )?;
        }
        for client in &mut clients {
            server.receive_input(client, &input_codec)?;
        }

        let observation = server.observation();
        server.publish_observation(&mut clients, &snapshot_codec, &observation)?;
        for client in &mut clients {
            let decision = client
                .receive_observation(&snapshot_codec)?
                .expect("server published one observation per tick");
            if !decision.is_accepted() {
                return Err("client rejected a newly published server observation".into());
            }
        }
    }

    for client in &clients {
        assert_eq!(
            client.latest_observation.as_ref().map(|value| value.tick),
            Some(3)
        );
    }
    Ok(server.positions)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let positions = run_simulation()?;
    println!(
        "authoritative-server ticks=3 clients={CLIENT_COUNT} positions={positions:?} input_provider=in-memory-loopback observation_provider=in-memory-loopback"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_remains_authoritative_over_two_client_inputs() {
        assert_eq!(
            run_simulation().expect("simulation should succeed"),
            [3, -2]
        );
    }

    #[test]
    fn invalid_client_input_is_rejected_before_server_state_changes() {
        let input_codec =
            JsonEnvelopeCodec::new(SchemaIdentity::new(INPUT_SCHEMA_ID, SCHEMA_VERSION));
        let mut server = AuthoritativeServer {
            tick: 0,
            positions: [0; CLIENT_COUNT],
            last_input_sequence: [None; CLIENT_COUNT],
        };
        let mut client = SimulatedClient::new(0);

        client
            .send_input(&input_codec, 1, 1, 2)
            .expect("fixture input should encode and send");
        let error = server
            .receive_input(&mut client, &input_codec)
            .expect_err("server must reject movement outside the application range");

        assert!(error
            .to_string()
            .contains("outside the allowed movement range"));
        assert_eq!(server.positions, [0, 0]);
    }
}
