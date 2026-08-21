use hello_browser_terminal_observer::{
    classify_terminal_outcome, SubjectProcessState, SubjectTerminalEvent, TerminalOutcome,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_REQUEST_BYTES: usize = 16 * 1024;
const MAX_FIELD_BYTES: usize = 4096;

#[derive(Debug)]
struct Config {
    browser: PathBuf,
    url: String,
    profile: Option<PathBuf>,
    log: Option<PathBuf>,
    result: Option<PathBuf>,
    observer_port: u16,
    startup_timeout: Duration,
    heartbeat_timeout: Duration,
    overall_timeout: Duration,
    close_browser_on_terminal: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageObservation {
    schema_version: u32,
    run_id: String,
    subject_id: String,
    sequence: u64,
    event: String,
    operation: String,
    #[serde(default)]
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalRecord<'a> {
    schema_version: u32,
    run_id: &'a str,
    classification: &'a str,
    elapsed_milliseconds: f64,
    subject_started: bool,
    last_sequence: Option<u64>,
    reason: &'a str,
    subject_detail: &'a str,
    browser_log: String,
    profile: String,
    physical_cause: &'static str,
    browser_disposition: &'a str,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("browser-terminal-observer rejected: {error}");
            ExitCode::from(64)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let config = parse_config()?;
    let run_id = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    );
    let profile = config.profile.clone().unwrap_or_else(|| {
        PathBuf::from(format!("target/browser-terminal-observer-profile-{run_id}"))
    });
    let log = config
        .log
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("target/browser-terminal-observer-{run_id}.log")));
    let result = config.result.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/browser-terminal-observer-{run_id}.terminal.json"
        ))
    });
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, config.observer_port))
        .map_err(|error| format!("failed to bind loopback observer: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("failed to configure loopback observer: {error}"))?;
    let observer_address = listener.local_addr().map_err(|error| error.to_string())?;
    let observer_url = format!("http://{observer_address}/__tokimu_terminal");
    let target_url = append_query(
        &config.url,
        &[("tokimu_observer", &observer_url), ("tokimu_run", &run_id)],
    );

    if let Some(parent) = profile.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if let Some(parent) = log.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if let Some(parent) = result.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if result.exists() {
        return Err(format!(
            "terminal result path already exists: {}",
            result.display()
        ));
    }
    fs::create_dir_all(&profile).map_err(|error| error.to_string())?;
    let profile = absolute_path(&profile)?;
    let log = absolute_path(&log)?;
    let result = absolute_path(&result)?;

    let (sender, receiver) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let server_stop = Arc::clone(&stop);
    let expected_run = run_id.clone();
    let server =
        thread::spawn(move || serve_observations(listener, expected_run, sender, server_stop));

    let mut browser = launch_browser(&config, &profile, &log, &target_url)?;
    println!(
        "browser-terminal-observer started: run-id={run_id}; launcher-pid={}; ownership=pending-page-acknowledgement; observer={observer_url}; url={}; startup-timeout-ms={}; heartbeat-timeout-ms={}; overall-timeout-ms={}; cause=unassigned",
        browser.id(),
        config.url,
        config.startup_timeout.as_millis(),
        config.heartbeat_timeout.as_millis(),
        config.overall_timeout.as_millis(),
    );

    let started_at = Instant::now();
    let mut subject_started = false;
    let mut subject_id: Option<String> = None;
    let mut last_heartbeat = started_at;
    let mut last_sequence = None;
    let mut terminal_event = None;
    let mut terminal_detail = String::new();
    let mut launcher_handed_off = false;
    let outcome = 'observe: loop {
        while let Ok(observation) = receiver.try_recv() {
            if subject_id
                .as_ref()
                .is_some_and(|current| current != &observation.subject_id)
            {
                break 'observe TerminalOutcome::UnresolvedDisappearance {
                    reason: "page-subject-identity-changed-before-terminal-outcome",
                };
            }
            subject_id.get_or_insert_with(|| observation.subject_id.clone());
            last_sequence = Some(last_sequence.unwrap_or(0).max(observation.sequence));
            match observation.event.as_str() {
                "subject-started" => {
                    subject_started = true;
                    last_heartbeat = Instant::now();
                }
                "heartbeat" | "operation-started" => {
                    subject_started = true;
                    last_heartbeat = Instant::now();
                }
                "operation-completed" | "operator-completed" => {
                    terminal_detail = observation.detail;
                    terminal_event = Some(SubjectTerminalEvent::Completed {
                        operation: observation.operation,
                    });
                }
                "structured-rejection" => {
                    terminal_detail = observation.detail.clone();
                    terminal_event = Some(SubjectTerminalEvent::StructuredFailure {
                        operation: observation.operation,
                        detail: observation.detail,
                    });
                }
                "page-error" => {
                    eprintln!(
                        "browser-terminal-observer page evidence: run-id={run_id}; sequence={}; event=page-error; operation={}; detail={}",
                        observation.sequence,
                        observation.operation,
                        bounded(&observation.detail),
                    );
                }
                _ => {}
            }
        }

        let process_state = if launcher_handed_off {
            SubjectProcessState::LauncherHandedOff
        } else {
            match browser.try_wait().map_err(|error| error.to_string())? {
                Some(_status) if !subject_started => {
                    launcher_handed_off = true;
                    SubjectProcessState::LauncherHandedOff
                }
                Some(status) => SubjectProcessState::from(status),
                None => SubjectProcessState::Running,
            }
        };
        let now = Instant::now();
        let heartbeat_expired = if subject_started {
            now.duration_since(last_heartbeat) >= config.heartbeat_timeout
        } else {
            now.duration_since(started_at) >= config.startup_timeout
        };
        let classified = classify_terminal_outcome(
            terminal_event.clone(),
            process_state,
            heartbeat_expired,
            subject_started,
        );
        if classified != TerminalOutcome::Running {
            break classified;
        }
        if now.duration_since(started_at) >= config.overall_timeout {
            break TerminalOutcome::UnresolvedDisappearance {
                reason: "overall-observation-deadline-expired-without-terminal-outcome",
            };
        }
        thread::sleep(Duration::from_millis(50));
    };

    stop.store(true, Ordering::Release);
    let _ = server.join();
    let browser_disposition = match &outcome {
        TerminalOutcome::Completed { .. } | TerminalOutcome::StructuredFailure { .. }
            if launcher_handed_off =>
        {
            "launcher-handed-off-browser-not-process-owned-after-terminal-record"
        }
        TerminalOutcome::Completed { .. } | TerminalOutcome::StructuredFailure { .. }
            if config.close_browser_on_terminal =>
        {
            terminate_owned_browser(&mut browser);
            "terminated-by-observer-on-request-after-terminal-record"
        }
        TerminalOutcome::Completed { .. } | TerminalOutcome::StructuredFailure { .. } => {
            "left-running-after-terminal-record"
        }
        TerminalOutcome::ExternallyTerminated { .. } => "already-exited-before-terminal-record",
        TerminalOutcome::UnresolvedDisappearance { .. } if launcher_handed_off => {
            "launcher-handed-off-browser-not-process-owned-after-unresolved-observation"
        }
        TerminalOutcome::UnresolvedDisappearance { .. } => {
            terminate_owned_browser(&mut browser);
            "terminated-by-observer-after-unresolved-observation"
        }
        TerminalOutcome::Running => unreachable!(),
    };
    let elapsed = started_at.elapsed();
    let (classification, exit_code, reason) = match outcome {
        TerminalOutcome::Completed { operation } => ("completed", 0, operation),
        TerminalOutcome::StructuredFailure { operation, detail } => (
            "structured-failure",
            2,
            format!("operation={operation}; detail={}", bounded(&detail)),
        ),
        TerminalOutcome::ExternallyTerminated { exit_code } => (
            "externally-terminated",
            3,
            format!("browser-exit-code={exit_code:?}; cause=unknown"),
        ),
        TerminalOutcome::UnresolvedDisappearance { reason } => {
            ("unresolved-disappearance", 4, reason.to_owned())
        }
        TerminalOutcome::Running => unreachable!(),
    };
    let terminal_record = TerminalRecord {
        schema_version: 1,
        run_id: &run_id,
        classification,
        elapsed_milliseconds: elapsed.as_secs_f64() * 1000.0,
        subject_started,
        last_sequence,
        reason: &reason,
        subject_detail: &terminal_detail,
        browser_log: log.display().to_string(),
        profile: profile.display().to_string(),
        physical_cause: "unknown-unless-explicitly-reported",
        browser_disposition,
    };
    let result_bytes = serde_json::to_vec_pretty(&terminal_record)
        .map_err(|error| format!("failed to encode terminal result: {error}"))?;
    let temporary_result = result.with_extension("terminal.json.tmp");
    fs::write(&temporary_result, result_bytes)
        .map_err(|error| format!("failed to write terminal result: {error}"))?;
    fs::rename(&temporary_result, &result)
        .map_err(|error| format!("failed to install terminal result: {error}"))?;
    println!(
        "browser-terminal-observer terminal: run-id={run_id}; classification={classification}; elapsed-ms={:.3}; subject-started={subject_started}; last-sequence={last_sequence:?}; reason={}; subject-detail={}; browser-disposition={browser_disposition}; browser-log={}; profile={}; result={}; physical-cause=unknown-unless-explicitly-reported",
        elapsed.as_secs_f64() * 1000.0,
        bounded(&reason),
        bounded(&terminal_detail),
        log.display(),
        profile.display(),
        result.display(),
    );
    Ok(ExitCode::from(exit_code))
}

fn parse_config() -> Result<Config, String> {
    let mut browser = None;
    let mut url = None;
    let mut profile = None;
    let mut log = None;
    let mut result = None;
    let mut observer_port = 0_u16;
    let mut startup_timeout = Duration::from_secs(30);
    let mut heartbeat_timeout = Duration::from_secs(30);
    let mut overall_timeout = Duration::from_secs(15 * 60);
    let mut close_browser_on_terminal = false;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        let value = |arguments: &mut std::iter::Skip<std::env::Args>| {
            arguments
                .next()
                .ok_or_else(|| format!("{argument} requires a value"))
        };
        match argument.as_str() {
            "--browser" => browser = Some(PathBuf::from(value(&mut arguments)?)),
            "--url" => url = Some(value(&mut arguments)?),
            "--profile" => profile = Some(PathBuf::from(value(&mut arguments)?)),
            "--log" => log = Some(PathBuf::from(value(&mut arguments)?)),
            "--result" => result = Some(PathBuf::from(value(&mut arguments)?)),
            "--observer-port" => {
                observer_port = value(&mut arguments)?
                    .parse()
                    .map_err(|_| "--observer-port must be a u16".to_owned())?
            }
            "--startup-timeout-seconds" => {
                startup_timeout = parse_duration(&argument, &mut arguments)?
            }
            "--heartbeat-timeout-seconds" => {
                heartbeat_timeout = parse_duration(&argument, &mut arguments)?
            }
            "--overall-timeout-seconds" => {
                overall_timeout = parse_duration(&argument, &mut arguments)?
            }
            "--close-browser-on-terminal" => close_browser_on_terminal = true,
            "--help" | "-h" => return Err(usage().to_owned()),
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }

    let browser = browser.ok_or_else(|| format!("--browser is required\n{}", usage()))?;
    if !browser.is_file() {
        return Err(format!(
            "browser executable does not exist: {}",
            browser.display()
        ));
    }
    let url = url.ok_or_else(|| format!("--url is required\n{}", usage()))?;
    if !url.starts_with("http://127.0.0.1:") && !url.starts_with("http://localhost:") {
        return Err("--url must be a loopback HTTP URL for this corpus harness".into());
    }
    Ok(Config {
        browser,
        url,
        profile,
        log,
        result,
        observer_port,
        startup_timeout,
        heartbeat_timeout,
        overall_timeout,
        close_browser_on_terminal,
    })
}

fn parse_duration(
    argument: &str,
    arguments: &mut std::iter::Skip<std::env::Args>,
) -> Result<Duration, String> {
    let seconds: u64 = arguments
        .next()
        .ok_or_else(|| format!("{argument} requires a value"))?
        .parse()
        .map_err(|_| format!("{argument} must be an integer number of seconds"))?;
    if seconds == 0 {
        return Err(format!("{argument} must be greater than zero"));
    }
    Ok(Duration::from_secs(seconds))
}

fn usage() -> &'static str {
    "usage: hello-browser-terminal-observer --browser <path> --url <loopback-url> [--profile <path>] [--log <path>] [--result <path>] [--observer-port <u16>] [--startup-timeout-seconds <n>] [--heartbeat-timeout-seconds <n>] [--overall-timeout-seconds <n>] [--close-browser-on-terminal]"
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        env::current_dir()
            .map(|directory| directory.join(path))
            .map_err(|error| format!("failed to resolve absolute observer path: {error}"))
    }
}

fn launch_browser(
    config: &Config,
    profile: &Path,
    log: &Path,
    target_url: &str,
) -> Result<Child, String> {
    Command::new(&config.browser)
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--new-window")
        .arg("--disable-background-mode")
        .arg("--disable-features=msEdgeStartupBoost")
        .arg("--enable-logging")
        .arg("--v=1")
        .arg(format!("--log-file={}", log.display()))
        .arg(target_url)
        .spawn()
        .map_err(|error| format!("failed to launch owned browser process: {error}"))
}

fn terminate_owned_browser(browser: &mut Child) {
    if matches!(browser.try_wait(), Ok(None)) {
        let _ = browser.kill();
        let _ = browser.wait();
    }
}

fn append_query(url: &str, pairs: &[(&str, &str)]) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    let mut result = format!("{url}{separator}");
    for (index, (key, value)) in pairs.iter().enumerate() {
        if index != 0 {
            result.push('&');
        }
        result.push_str(key);
        result.push('=');
        result.push_str(&percent_encode(value));
    }
    result
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn serve_observations(
    listener: TcpListener,
    expected_run: String,
    sender: mpsc::Sender<PageObservation>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let response = match read_observation(&mut stream, &expected_run) {
                    Ok(observation) => {
                        let _ = sender.send(observation);
                        "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    }
                    Err(_) => "HTTP/1.1 400 Bad Request\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                };
                let _ = stream.write_all(response.as_bytes());
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn read_observation(stream: &mut TcpStream, expected_run: &str) -> Result<PageObservation, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 2048];
    loop {
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err("request exceeded observer bound".into());
        }
        if let Some(header_end) = find_header_end(&bytes) {
            let headers =
                std::str::from_utf8(&bytes[..header_end]).map_err(|error| error.to_string())?;
            let length = content_length(headers)?;
            if bytes.len() >= header_end + 4 + length {
                break;
            }
        }
    }
    let header_end = find_header_end(&bytes).ok_or_else(|| "missing HTTP headers".to_owned())?;
    let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|error| error.to_string())?;
    if !headers.starts_with("POST /__tokimu_terminal HTTP/1.1") {
        return Err("unexpected observer request target".into());
    }
    let length = content_length(headers)?;
    let body_start = header_end + 4;
    let body_end = body_start
        .checked_add(length)
        .ok_or_else(|| "body length overflow".to_owned())?;
    let body = bytes
        .get(body_start..body_end)
        .ok_or_else(|| "incomplete request body".to_owned())?;
    let observation: PageObservation =
        serde_json::from_slice(body).map_err(|error| error.to_string())?;
    if observation.schema_version != 1 || observation.run_id != expected_run {
        return Err("observer schema or run correlation mismatch".into());
    }
    for field in [
        observation.subject_id.as_str(),
        observation.event.as_str(),
        observation.operation.as_str(),
        observation.detail.as_str(),
    ] {
        if field.len() > MAX_FIELD_BYTES {
            return Err("observer field exceeded bound".into());
        }
    }
    Ok(observation)
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(headers: &str) -> Result<usize, String> {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>())
        })
        .ok_or_else(|| "missing content length".to_owned())?
        .map_err(|_| "invalid content length".to_owned())
}

fn bounded(value: &str) -> String {
    value.chars().take(512).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_values_are_encoded_without_rewriting_the_target_url() {
        assert_eq!(
            append_query(
                "http://127.0.0.1:4177/?fixture=identity",
                &[("tokimu_observer", "http://127.0.0.1:49152/__tokimu_terminal")]
            ),
            "http://127.0.0.1:4177/?fixture=identity&tokimu_observer=http%3A%2F%2F127.0.0.1%3A49152%2F__tokimu_terminal"
        );
    }

    #[test]
    fn content_length_is_case_insensitive_and_bounded_by_the_caller() {
        assert_eq!(
            content_length("POST / HTTP/1.1\r\ncOnTeNt-LeNgTh: 42\r\n"),
            Ok(42)
        );
        assert!(content_length("POST / HTTP/1.1\r\n").is_err());
    }
}
