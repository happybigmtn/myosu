# Specification: Miner Process

Source: Reverse-engineered from crates/myosu-miner (main.rs, cli.rs, lib.rs)
Status: Draft
Depends-on: 001-game-trait-framework, 002-poker-solver-engine, 006-chain-rpc-client

## Purpose

The miner process trains game strategies via MCCFR and serves them to validators
and the gameplay surface over HTTP. It is the off-chain compute component of the
Myosu system: it registers on-chain, trains locally, checkpoints progress, and
exposes a strategy query endpoint that validators probe for quality scoring. A
miner's economic reward is proportional to the quality of strategies it
produces.

The primary consumer is a miner operator running the binary against a live chain
node.

## Whole-System Goal

Current state: The miner binary is fully implemented with a sequential startup
pipeline covering key resolution, chain probing, optional registration, optional
axon publication, MCCFR training, single-shot strategy serving, and persistent
HTTP axon serving.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: An operator can start a miner that registers on-chain, trains
a game strategy, and serves it over HTTP for validators to score.

Still not solved here: Validator scoring, emission distribution, gameplay
consumption of strategies, and multi-miner coordination are separate concerns.

## Scope

In scope:
- Sequential startup pipeline with 9 ordered phases
- Key resolution from URI or encrypted config directory
- Chain connectivity probing with health and neuron state
- Optional on-chain registration and axon endpoint publication
- MCCFR training with configurable iteration count and checkpoint resume
- Single-shot strategy serving from wire-encoded query/response files
- Persistent HTTP axon server for live strategy queries
- Structured reporting at each phase (6 report formats)
- Game selection (Poker or LiarsDice)

Out of scope:
- The MCCFR algorithm internals (handled by game crates)
- Chain runtime or pallet behavior
- Validator scoring or weight submission
- Gameplay rendering or user interaction
- Key generation or management (handled by myosu-keys)

## Current State

The binary exists at crates/myosu-miner with approximately 1,400 lines of code.
It depends on myosu-games-poker and myosu-games-liars-dice for game-specific
solvers, myosu-chain-client for chain interaction, and tracing for structured
logging.

The startup pipeline executes sequentially: (1) initialize tracing from RUST_LOG
or default to myosu_miner=info, (2) parse CLI with mutual exclusion between
--key URI and --key-config-dir, (3) resolve key URI (direct or decrypted from
config directory using password from environment variable), (4) probe chain via
WebSocket for health, peers, sync state, and neuron data, (5) optionally
register hotkey on subnet if --register flag is set, (6) optionally publish axon
endpoint on-chain if --serve_axon flag is set, (7) optionally run MCCFR training
batch if train_iterations > 0 or --checkpoint is provided, (8) optionally serve
a single strategy response from wire-encoded query/response files, (9) optionally
start persistent HTTP axon server.

Each phase produces a structured report printed to stdout. The miner either
exits after completing its configured phases or blocks indefinitely serving HTTP
strategy requests.

The HTTP axon server loads a checkpoint (from training output or explicit path)
and serves strategy queries on the configured port (default 8080). The /health
endpoint returns a status response. The /strategy endpoint accepts wire-encoded
queries and returns wire-encoded responses.

Training produces checkpoint files in the configured data directory. Each
training report includes game type, checkpoint path, epoch count, and
exploitability score.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| CLI parsing | Clap-based with mutual exclusion groups | Reuse | Clean argument validation |
| Key resolution | URI or encrypted config via myosu-keys | Reuse | Shared with validator |
| Chain probing | probe_chain with startup_report | Reuse | Common startup phase |
| Registration | ensure_registered via chain client | Reuse | Shared with validator |
| MCCFR training | training_plan_from_cli + run_training_batch | Reuse | Game-agnostic training |
| HTTP serving | Axon server with /health and /strategy | Reuse | Validator-compatible endpoint |

## Non-goals

- Running multiple training jobs concurrently.
- Automatically selecting the optimal game or subnet.
- Interacting with the gameplay surface directly.
- Implementing validator scoring or weight submission.
- Managing key lifecycle beyond resolution.

## Behaviors

The miner executes phases sequentially. Each phase depends on the previous:
training uses the key and chain state from earlier phases, and the HTTP server
uses the checkpoint from training.

Key resolution enforces mutual exclusion: either a raw key URI or a config
directory path, never both. Config directory resolution reads the encrypted
keyfile using a password from the environment variable specified by
--key_password_env (default MYOSU_KEY_PASSWORD).

Chain probing connects to the WebSocket endpoint and reports peer count, sync
status, available RPC methods, and neuron data. Probe failure terminates the
process.

Registration (when --register is set) calls ensure_registered, which submits a
burned_register transaction and polls for the UID to appear. The registration
report includes whether the hotkey was already registered.

Axon publication (when --serve_axon is set) calls ensure_serving, which submits
a serve_axon transaction with the configured port and polls for the axon info
to appear on-chain.

Training (when train_iterations > 0 or --checkpoint exists) builds a training
plan from CLI arguments including game type, encoder directory, checkpoint path,
and iteration count. The training batch runs the configured number of MCCFR
iterations and produces a checkpoint file.

Strategy serving (when --query_file and --response_file are provided) reads a
wire-encoded query, loads the latest checkpoint, and writes a wire-encoded
response. This supports deterministic single-shot validation.

HTTP axon serving (when configured) loads the checkpoint, binds to the
configured port, and serves indefinitely. The server is the last phase: once
started, the process does not exit until terminated.

All failures in any phase produce an error and terminate the process. There is
no retry logic or recovery within the startup pipeline.

## Acceptance Criteria

- The miner starts and probes the chain successfully, printing a startup report.
- Key resolution from a config directory decrypts the active key using the
  configured password environment variable.
- Registration succeeds and the miner's UID appears on the specified subnet.
- Axon publication succeeds and the endpoint appears in on-chain axon info.
- MCCFR training produces a checkpoint file with decreasing exploitability.
- Single-shot strategy serving produces a wire-encoded response that round-trips
  through the codec.
- The HTTP axon server responds to /health with a status response.
- The HTTP axon server responds to /strategy with a wire-encoded strategy
  response.
- Each startup phase prints a structured report to stdout.
- Failure in any phase terminates the process with a descriptive error.
