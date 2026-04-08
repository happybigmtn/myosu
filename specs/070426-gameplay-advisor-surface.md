# Specification: Gameplay & Advisor Surface

## Objective

Describe the gameplay binary (`myosu-play`), its TUI and pipe protocol modes, the blueprint bot (local deterministic opponent), live miner connection, miner discovery, and smoke test infrastructure.

## Evidence Status

### Verified (code-grounded)

- Gameplay crate: `crates/myosu-play/` with source files in `src/`:
  - `main.rs` (58.0K): TUI rendering, pipe protocol, advisor integration, game loop.
  - `blueprint.rs` (27.0K): Local blueprint bot (deterministic opponent using MCCFR).
  - `live.rs` (7.1K): Live miner connection via HTTP.
  - `discovery.rs` (3.3K): Miner endpoint discovery from on-chain axon data.
  - `cli.rs` (5.7K): Command-line interface.
- TUI framework: `crates/myosu-tui/` using `ratatui` 0.29 and `crossterm` 0.28 (`Cargo.toml:45-46`).
- Gameplay modes:
  - TUI mode: Interactive poker hand play against a trained strategy.
  - Pipe mode: Machine-readable advisor feed (`printf 'quit\n' | cargo run -p myosu-play -- pipe`).
  - Smoke test: `cargo run -p myosu-play -- --smoke-test` (`README.md:81`).
- INV-004 (`INVARIANTS.md:43-57`): Solver and gameplay layers share game engine code (`myosu-games`, `myosu-games-poker`) but never share runtime state or trust boundaries. `myosu-play` must not import from `myosu-miner` and vice versa.
- The E2E smoke test (`stage0_local_loop.rs:5-18`) captures per-game:
  - `gameplay_advice_source`: Which advisor served the gameplay session.
  - `gameplay_final_state`: Terminal state of the gameplay session.
  - `gameplay_discovered_miner_uid`: Miner UID discovered from on-chain axon data (optional).
  - `gameplay_discovered_miner_endpoint`: HTTP endpoint discovered for the miner (optional).
  - `gameplay_live_miner_connect_endpoint`: Endpoint used for live connection (optional).
- Proof commands:
  - `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test` (`README.md:81`)
  - `printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe` (`README.md:82`)
- Workspace member: `crates/myosu-play` (`Cargo.toml:8`).
- Two-game proof: The E2E loop exercises gameplay for both poker and Liar's Dice subnets (`stage0_local_loop.rs` parses both game prefixes).

### Recommendations (intended future direction)

- Plan 012 (README overhaul) recommends documenting the gameplay commands more clearly.
- The gameplay surface is TUI + pipe only for stage-0. Web/mobile surfaces are explicitly not in scope.

### Hypotheses / Unresolved

- Whether the pipe protocol has a documented wire format (delimiters, encoding) or is ad-hoc.
- Whether `discovery.rs` works across multi-node networks or only local devnet (single-node).
- The blueprint bot's MCCFR iteration count and whether it uses pre-trained or JIT-computed strategies.

## Acceptance Criteria

- `cargo check -p myosu-play` succeeds (with `SKIP_WASM_BUILD=1`)
- `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test` exits successfully
- Pipe mode accepts `quit\n` on stdin and exits cleanly
- The gameplay binary has no direct dependency on `myosu-miner` (INV-004)
- The blueprint bot produces valid `StrategyResponse` payloads (probabilities sum to ~1.0)
- Live miner connection successfully queries a running miner axon and receives strategy
- In the E2E smoke test, `gameplay_final_state` is a recognized terminal state for each game

## Verification

```bash
# Compile check
SKIP_WASM_BUILD=1 cargo check -p myosu-play

# Smoke test
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test

# Pipe mode exits cleanly
printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe

# INV-004: no dependency on myosu-miner
cargo tree -p myosu-play 2>/dev/null | grep -c myosu-miner
# Expected: 0

# E2E proof
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
```

## Open Questions

- Is the pipe protocol format documented anywhere? What are the message delimiters and encoding?
- Does the blueprint bot train on startup or load a pre-computed checkpoint?
- How does miner discovery behave when multiple miners are registered on the same subnet?
- Is there a spectator mode (observing a game without playing)?
