use std::io::{self, BufRead, Write};

use tokimu_resource_workbench_bridge::{BridgeRequest, ResourceBridge};

fn main() {
    let stdin = io::stdin();
    let mut bridge = ResourceBridge::default();
    let mut stdout = io::BufWriter::new(io::stdout().lock());

    for line in stdin.lock().lines() {
        let response = match line {
            Ok(line) => match serde_json::from_str::<BridgeRequest>(&line) {
                Ok(request) => bridge.execute(request),
                Err(error) => ResourceBridge::invalid_request(error.to_string()),
            },
            Err(error) => {
                ResourceBridge::invalid_request(format!("could not read request: {error}"))
            }
        };
        if serde_json::to_writer(&mut stdout, &response).is_err()
            || writeln!(stdout).is_err()
            || stdout.flush().is_err()
        {
            break;
        }
    }
}
