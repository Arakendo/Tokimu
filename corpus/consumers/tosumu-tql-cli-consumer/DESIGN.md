# Tosumu TQL CLI Consumer Corpus

## Purpose

Prove that a downstream Rust tool can consume Tosumu's bounded, provisional
TQL JSON contract through an external process without parsing TQL, opening a
Tosumu store, or recreating Tosumu inspection semantics.

## Consumer Claim

```text
fixture database
        ↓
tosumu executable
        ↓
TQL command text
        ↓
versioned JSON envelope
        ↓
Rust consumer observation
```

At no point does this consumer:

- parse TQL grammar;
- open or inspect Tosumu storage directly;
- render or scrape human-facing CLI text;
- reinterpret integrity, WAL, or value metadata facts.

## Ownership

- **Tosumu CLI** owns command parsing, database opening, public inspection,
  dispatch, errors, and JSON-envelope production.
- **This consumer** owns process invocation, fixture orchestration, schema
  version checks, and evidence reporting.
- **Tokimu corpus** owns the independent-consumer evidence only. It does not
  promote TQL into Tokimu or define its semantics.

## Scope

The first fixture invokes:

- `STATUS`
- `CHECK`
- `DESCRIBE asset/manifest`
- `DESCRIBE missing/key`
- `WAL STATUS`
- a malformed `STATUS trailing` command to preserve typed error evidence.

The consumer expects `schema_version = 1` and records field-level outcomes. It
does not claim a stable public TQL ABI; the Tosumu TQL architectural review
remains responsible for that decision.

## Running

Build Tosumu's CLI first, then run the corpus consumer from the Tokimu root:

```powershell
cargo build --manifest-path .\third-party\tosumu\Cargo.toml -p tosumu-cli
cargo run -p tosumu-tql-cli-consumer
```

Set `TOSUMU_CLI_BIN` when the executable lives elsewhere. The consumer uses
only the executable boundary and does not link against Tosumu crates.

## Success Criteria

- Each admitted command returns a versioned JSON envelope.
- Successful and typed-invalid inputs remain distinguishable by command,
  outcome, error code, and process result.
- No human-readable output is parsed.
- The consumer can be rerun without changing Tosumu's command semantics.
