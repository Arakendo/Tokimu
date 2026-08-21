# Hello Browser Terminal Observer

Windows/Edge-focused, corpus-private evidence for ADR-0017. The observer runs
outside the page, owns an isolated Edge process, receives bounded loopback
heartbeats and terminal events, and classifies exactly one run as:

- `completed`;
- `structured-failure`;
- `externally-terminated`; or
- `unresolved-disappearance`.

It does not diagnose OOM, WGPU, browser, driver, or hardware failure from
missing evidence. It is not a general process supervisor or a stable Tokimu
API.

First serve the desired fixture. For the independent renderer-lifetime page:

```powershell
python -m http.server 4177 --bind 127.0.0.1 --directory corpus/campaigns/renderer-reliability/hello-render-resource-identity-web/web
```

In another terminal, launch an isolated Edge process under observation:

```powershell
cargo run -p hello-browser-terminal-observer -- `
  --browser "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" `
  --url "http://127.0.0.1:4177/" `
  --startup-timeout-seconds 60 `
  --heartbeat-timeout-seconds 60 `
  --overall-timeout-seconds 900
```

For the Doom browser workbench, serve its built web directory and point the
same observer at port 4176:

```powershell
python -m http.server 4176 --bind 127.0.0.1 --directory corpus/consumers/doom-ts-boundary-workbench/web

cargo run -p hello-browser-terminal-observer -- `
  --browser "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" `
  --url "http://127.0.0.1:4176/" `
  --startup-timeout-seconds 60 `
  --heartbeat-timeout-seconds 60 `
  --overall-timeout-seconds 1800
```

The observer creates a unique profile, browser log, and structured terminal
JSON record below `target/`. After a completed or structured-failure record it
leaves the isolated browser open for inspection by default. Pass
`--close-browser-on-terminal` for automated cleanup. It still terminates the
browser it owns after an unresolved liveness/identity outcome. A page reload or
replacement before a terminal outcome changes the page-subject identity and is
retained as an unresolved disappearance rather than silently becoming a new
run.

The owned Chromium-family launch disables background mode and Edge startup
boost. If the host still hands the launch to another process before page
acknowledgement, the observer waits for the bounded page acknowledgement rather
than calling the launcher exit a browser termination. After handoff, page
identity, heartbeat, and terminal events remain authoritative; missing
acknowledgement or heartbeat still closes as an unresolved disappearance.

The launch PID is not treated as an owned browser until the instrumented page
acknowledges the observer. A browser launcher that prints `Opening in existing
browser session` and exits before that acknowledgement is an unresolved
startup/ownership failure, not an externally observed browser termination.

For the historical Alternative B fixture, run the retained-session sequence,
then the stale-aliasing probe, then the destructive atomicity probe. The
atomicity probe closes that observed workflow. For the Doom workbench, a
completed three-round ADR-0018 rotation closes the workflow automatically and
its bounded success detail is retained in the terminal record. A manual
walkabout remains open until the operator presses **Complete observed
walkabout**.

Exit codes are stable fixture evidence:

| Code | Classification |
| ---: | --- |
| 0 | Completed; browser remains open unless cleanup was requested |
| 2 | Structured failure |
| 3 | Browser process terminated before a page terminal record |
| 4 | Unresolved disappearance or liveness timeout |
| 64 | Harness configuration/startup rejection |

The browser log and retained terminal JSON must be reviewed together. An exit
code alone does not establish physical cause. The harness observes the owned
browser process and page subject; it does not yet independently identify the
browser's renderer or GPU subprocesses.
