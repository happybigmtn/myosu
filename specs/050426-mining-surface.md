# Specification: Mining Surface

## Objective

Define the contract for `myosu-miner`, the operator-facing binary that bootstraps a miner node: probes the chain, optionally registers and publishes an axon, runs bounded MCCFR training, saves checkpoints, and serves strategy via file or persistent HTTP.

## Evidence Status

All claims below are grounded in source at `crates/myosu-miner/src/`.

- **Verified**: CLI flags, lifecycle ordering, report formats, error types, HTTP routes, request size limit, game dispatch, checkpoint I/O, wire codec round-trip.
- **Unverified**: minimum useful training iteration count, production suitability of unauthenticated HTTP axon under adversarial load.

## Acceptance Criteria

### Binary and CLI

- Binary name is `myosu-miner`, crate root at `crates/myosu-miner/`.
- CLI is clap-derive (`cli.rs`), with `--chain`, `--subnet`, `--key` | `--key-config-dir` (mutually exclusive, one required via `ArgGroup`), `--key-password-env`, `--port`, `--register`, `--serve-axon`, `--serve-http`, `--data-dir`, `--game`, `--encoder-dir`, `--checkpoint`, `--train-iterations`, `--query-file`, `--response-file`.
- `--chain` defaults to `ws://127.0.0.1:9944`. `--port` defaults to `8080`. `--data-dir` defaults to `./miner-data`. `--game` defaults to `poker`. `--train-iterations` defaults to `0`.
- Key resolution: `--key` accepts a raw seed/URI; `--key-config-dir` loads via `myosu_keys::load_active_secret_uri_from_env` using the env var named by `--key-password-env` (default `MYOSU_KEY_PASSWORD`).
- Supported games: `Poker`, `LiarsDice` (via `GameSelection` enum).

### Lifecycle (main.rs)

- Step 1: Initialize tracing (`myosu_miner=info` default filter), parse CLI, resolve key URI.
- Step 2: `probe_chain` connects via WebSocket, fetches `system_health`, `rpc_methods`, `neuron_info_get_neurons_lite`. Prints `startup_report`.
- Step 3: If `--register`, calls `ensure_registered` (burned registration, 20s timeout). Prints `registration_report`.
- Step 4: If `--serve-axon`, calls `ensure_serving` (publishes axon IP/port on-chain, 20s timeout). Prints `axon_report`.
- Step 5: Build `TrainingPlan` from CLI. If training is requested (iterations > 0, or query-file without checkpoint triggers bootstrap), run `run_training_batch`. Prints `training_report`.
- Step 6: Build `StrategyServePlan` from CLI (accepts checkpoint hint from step 5). If query-file provided, run `serve_strategy_once`. Prints `strategy_report`.
- Step 7: Build `AxonServePlan` from CLI (accepts checkpoint hint from step 5). If `--serve-http`, load solver and bind TCP listener, then `serve()` in an infinite accept loop. Prints `http_axon_report` before entering loop.
- Each step is sequential; later steps depend on earlier outputs (checkpoint path flows forward).

### Chain Operations (chain.rs)

- `probe_chain(endpoint, subnet)` returns `ChainProbeReport { endpoint, subnet, health, rpc_methods, neuron_lite_bytes }`.
- `ensure_registered(endpoint, key_uri, subnet)` returns `RegistrationReport { hotkey, subnet, uid, already_registered, extrinsic_hash }`.
- `ensure_serving(endpoint, key_uri, subnet, port)` returns `AxonServeReport { hotkey, subnet, version, ip, port, already_published, extrinsic_hash }`. Version is hardcoded to `1`, IP to `0`.

### Training (training.rs)

- `TrainingPlan { game, encoder_dir, checkpoint, checkpoint_output, iterations }`.
- Poker training requires `--encoder-dir`; Liar's Dice does not.
- Poker: loads `PokerSolver` from checkpoint or creates new, calls `solver.train(iterations)`, saves to `checkpoint_output`.
- Liar's Dice: uses `LiarsDiceSolver<1024>` (const `LIARS_DICE_SOLVER_TREES = 1 << 10`).
- Default checkpoint output: `{data_dir}/checkpoints/latest.bin`.
- `TrainingRunReport` includes `exploitability` -- Poker may return `"unavailable: {error}"` for sparse encoders; Liar's Dice always returns exact exploitability.
- Bootstrap mode: when `--query-file` is set without `--checkpoint`, a zero-iteration training run is triggered to produce a checkpoint for the strategy step.

### Strategy Serving (strategy.rs)

- File-based one-shot: `--query-file` + `--response-file` (default `{data_dir}/responses/latest.bin`).
- Reads wire-encoded query from disk, loads solver from checkpoint, writes wire-encoded response.
- `StrategyServeReport { game, response_path, action_count, recommended_action }`.
- Poker queries decoded via `myosu_games_poker::decode_strategy_query`; Liar's Dice via `myosu_games_liars_dice::decode_strategy_query`.
- Invalid query bytes produce `StrategyServeError::DecodeQuery` with the file path in context.

### HTTP Axon (axon.rs)

- Activated by `--serve-http`. Requires `--encoder-dir` and a checkpoint (from `--checkpoint` or training output). Poker only; Liar's Dice returns `AxonServeError::UnsupportedGame`.
- Custom HTTP/1.1 server over raw `TcpListener`/`TcpStream` (no framework).
- Request size limit: 64 KiB (`REQUEST_LIMIT_BYTES`).
- Routes: `GET /health` returns JSON `{"status":"ok","epochs":<n>}`; `POST /strategy` accepts wire-encoded body, returns wire-encoded response as `application/octet-stream`.
- Unknown routes return 404. Invalid requests return 400.
- Connection handling: one connection at a time in the accept loop (sequential, not concurrent).
- `AxonServeReport { game, bind_endpoint, connect_endpoint, checkpoint_path, epochs }`.
- Bind on `0.0.0.0:{port}` by default; connect endpoint rewrites unspecified to `127.0.0.1`.

### Reports (lib.rs)

- Six report formatters: `startup_report`, `registration_report`, `axon_report`, `http_axon_report`, `training_report`, `strategy_report`.
- All produce multi-line `key=value` plain text prefixed with a section tag (`MINER`, `REGISTRATION`, `AXON`, `HTTP`, `TRAINING`, `STRATEGY`).
- Reports are printed to stdout via `print!`, not logged via tracing.

### Error Handling

- Top-level error type: `MinerBootstrapError` (in `training.rs`) with variants for Key, Chain, ChainAction, Training, Strategy, Axon.
- All error types use `thiserror` with `#[source]` chains.
- Errors include paths and operation context for operator diagnosis.

### Testing

- CI: `SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet` (in `.github/workflows/ci.yml`).
- E2E: `tests/e2e/local_loop.sh` exercises the full miner path.
- Unit tests cover: CLI parsing (both key sources, all flags), training plan validation (missing encoder, bootstrap mode), training batch execution (zero-iteration save, sparse encoder failure), strategy plan validation (missing checkpoint), strategy serving (round-trip wire codec, bad query bytes), axon plan validation (missing checkpoint, unsupported game), axon server (health + strategy endpoints via TCP).

## Verification

- `SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet` -- all unit tests pass.
- `cargo clippy -p myosu-miner --all-targets -- -D warnings` -- zero warnings.
- Manual: `tests/e2e/local_loop.sh` with a running devnet exercises the full lifecycle.

## Open Questions

- What is the minimum `--train-iterations` count that produces a strategy meaningfully better than uniform random? No convergence quality gate exists today.
- The HTTP axon has no authentication and no rate limiting. Is this acceptable for multi-node devnet, or should a shared secret or connection limit be added before operators expose axons to untrusted validators?
- HTTP axon handles one connection at a time (sequential accept loop). Under concurrent validator queries, this becomes a bottleneck. Should it spawn per-connection tasks?
- Liar's Dice has no HTTP axon path (`AxonServeError::UnsupportedGame`). Is this planned, or will validators only query Liar's Dice miners via file-based strategy?
- `ensure_serving` hardcodes IP to `0` and version to `1`. Should these be configurable or derived from the operator's network environment?
