# Specification: Unified Observability

Source: Genesis Plan 005 M1 (Unified tracing), ASSESSMENT.md observability gaps
Status: Draft
Depends-on: none

## Purpose

The three myosu binaries (miner, validator, play) use inconsistent logging
approaches: miner and validator use the `tracing` crate with structured output,
while play and the game crates use print-based logging. Operators running the
full stack cannot correlate events across components, filter log levels
uniformly, or integrate with standard log aggregation tools. Unifying all
binaries on a single structured logging surface gives operators consistent
diagnostics across the entire local loop.

## Whole-System Goal

Current state: `myosu-miner` and `myosu-validator` initialize a tracing
subscriber and emit structured log events. `myosu-play` and the game library
crates use `println!`/`eprintln!` for diagnostic output. There is no shared log
format, no environment-based level filtering in play, and no way to get
structured output from the gameplay surface.

This spec adds: A unified tracing subscriber across all three binaries with
environment-based log level filtering and structured output format.

If all ACs land: An operator can set `RUST_LOG=myosu=debug` and get consistent,
structured, filterable log output from every myosu binary in the stack.

Still not solved here: Metrics collection, distributed tracing across network
boundaries, alerting, dashboards, and log aggregation infrastructure.

## Scope

In scope:
- Adding tracing subscriber initialization to myosu-play
- Replacing print-based logging in game library crates with tracing macros
- Ensuring all three binaries respect `RUST_LOG` environment variable filtering
- Consistent structured log output format across binaries

Out of scope:
- Metrics or prometheus endpoints
- Distributed tracing (trace IDs across network calls)
- Log aggregation infrastructure or dashboards
- Alerting or monitoring systems
- Changing the tracing setup in miner or validator beyond format consistency

## Current State

`crates/myosu-miner/src/main.rs` and `crates/myosu-validator/src/main.rs` both
initialize a tracing subscriber with `tracing_subscriber::fmt`. The miner and
validator crates use `tracing::info!`, `tracing::warn!`, etc. throughout.

`crates/myosu-play/src/main.rs` does not initialize a tracing subscriber.
Diagnostic output in `myosu-play`, `myosu-games-poker`, and
`myosu-games-liars-dice` uses `println!` and `eprintln!`.

`crates/myosu-tui/` manages terminal state via ratatui/crossterm, which requires
care when mixing with stderr-based log output in TUI mode.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Miner tracing setup | `crates/myosu-miner/src/main.rs` subscriber init | Reuse as reference | Working pattern to replicate |
| Validator tracing setup | `crates/myosu-validator/src/main.rs` subscriber init | Reuse as reference | Working pattern to replicate |
| Play entry point | `crates/myosu-play/src/main.rs` | Extend | Add subscriber initialization |
| Game diagnostic output | `println!` calls in game crates | Replace | Switch to tracing macros |
| TUI terminal management | `crates/myosu-tui/src/` | Extend | Must coexist with tracing output |

## Non-goals

- Extracting a shared tracing initialization library across binaries. The setup
  is small enough to duplicate until a third pattern emerges.
- Adding tracing to the chain binary (Substrate has its own logging framework).
- Structured JSON log output — human-readable format is sufficient for stage-0.
- Performance benchmarking of tracing overhead.

## Behaviors

All three binaries (myosu-miner, myosu-validator, myosu-play) initialize a
tracing subscriber at startup that reads the `RUST_LOG` environment variable for
level filtering. When `RUST_LOG` is not set, a sensible default applies (info
level for myosu crates, warn for dependencies).

Game library crates emit diagnostic events through tracing macros rather than
print statements. In TUI mode, tracing output is directed to stderr or
suppressed to avoid corrupting the terminal display. In pipe mode and smoke-test
mode, tracing output appears on stderr alongside the structured protocol output
on stdout.

Log events include at minimum: timestamp, level, target module, and message.
The format is consistent across all three binaries so that interleaved log
output from concurrent processes is parseable.

## Acceptance Criteria

- All three binaries (myosu-miner, myosu-validator, myosu-play) produce
  structured log output via tracing when started.
- Setting `RUST_LOG=myosu_play=debug` produces debug-level output from the play
  binary.
- No `println!` or `eprintln!` calls remain in active game library crates for
  diagnostic purposes (user-facing TUI rendering excluded).
- TUI mode does not display raw log lines in the terminal interface.
- Log output format (timestamp, level, target, message) is consistent across
  all three binaries.
