# Myosu Design System

Date: 2026-04-02
Status: Active
Scope: All user-facing surfaces -- TUI gameplay, CLI outputs, pipe protocol, and
operator-facing reports.

## Design Principles

1. **Terminal-first.** All primary interactions happen in the terminal. No web UI
   exists today. Design for 80x24 minimum, 120x40 comfortable.
2. **Structured output for agents.** Every interactive surface has a pipe mode
   counterpart that emits structured text (JSON or line protocol) for non-human
   consumers.
3. **Operator reports, not dashboards.** Miner and validator produce deterministic
   text reports at each phase. Reports are the primary observability surface.
4. **Progressive disclosure.** Default output is minimal. Verbose output is
   opt-in via flags or environment variables.

## User-Facing Surfaces

### 1. myosu-play (Interactive Gameplay)

**Entry point:** `crates/myosu-play/src/main.rs`

**Modes:**
- TUI mode (default): Full ratatui terminal interface
- Pipe mode (`--pipe`): Structured line protocol for agent integration
- Smoke test (`--smoke-test`): Headless validation

**TUI Layout (Five-Panel Shell):**
```
+-------------------------------------------+
|              Header Panel                 |
+-------------------------------------------+
|  Transcript  |  Game State  |  Declaration|
|   Panel      |   Panel      |   Panel     |
|              |              |             |
+-------------------------------------------+
|              Input Panel                  |
+-------------------------------------------+
```

**Screen State Machine:**
- Screens defined in `crates/myosu-tui/src/screens.rs`
- Event loop in `crates/myosu-tui/src/events.rs`
- Input processing in `crates/myosu-tui/src/input.rs`
- Theme in `crates/myosu-tui/src/theme.rs`

**Game Renderers:**
- `NlheRenderer` in `crates/myosu-games-poker/src/renderer.rs` (1,551 lines)
- `LiarsDiceRenderer` in `crates/myosu-games-liars-dice/src/renderer.rs`
- Both implement `GameRenderer` trait from `crates/myosu-tui/src/renderer.rs`

**Interaction States:**
```
InteractionState {
    WaitingForInput,
    ProcessingAction,
    ShowingResult,
    GameOver,
}
```

**Advice Display:**
- Blueprint-backed strategy advice with freshness tracking
- Live miner query with connectivity status (Fresh/Stale/Offline)
- 4-tier artifact discovery with graceful fallback

### 2. myosu-miner (Mining Service)

**Entry point:** `crates/myosu-miner/src/main.rs`

**Operator Flow:**
```
probe_chain → ensure_registered → ensure_serving
           → run_training_batch → serve_strategy_once
           → load_axon_server → server.serve()
```

**Report Functions:**
- `startup_report()` -- chain probe results
- `registration_report()` -- neuron registration status
- `axon_report()` -- axon serving status
- `training_report()` -- MCCFR training results
- `strategy_report()` -- strategy serving results
- `http_axon_report()` -- HTTP server status

### 3. myosu-validator (Validation Oracle)

**Entry point:** `crates/myosu-validator/src/main.rs`

**Operator Flow:**
```
probe_chain → ensure_registered → ensure_subtoken_enabled
           → ensure_subnet_tempo → ensure_weights_set_rate_limit
           → ensure_commit_reveal_enabled → ensure_validator_permit_ready
           → ensure_weights_set → score_response → submit_weights
```

**Report Functions:**
- `startup_report()`, `registration_report()`, `subtoken_bootstrap_report()`
- `subnet_tempo_bootstrap_report()`, `weights_rate_limit_bootstrap_report()`
- `commit_reveal_bootstrap_report()`, `permit_bootstrap_report()`
- `validation_report()`, `weight_submission_report()`

### 4. myosu-keys (Key Management)

**Entry point:** `crates/myosu-keys/src/main.rs`

**Commands:** `create`, `import`, `list`, `export`, `switch`, `print-bootstrap`

**Security properties:**
- XSalsa20Poly1305 AEAD encryption
- scrypt KDF for password-based key derivation
- 0o600 Unix file permissions on encrypted key files
- Password via environment variable (no interactive prompt in production)

### 5. Operator Bundle

**Entry point:** `.github/scripts/prepare_operator_network_bundle.sh`

**Contents:**
- `start-miner.sh`, `start-validator.sh` -- startup scripts
- `build-devnet-spec.sh`, `build-test-finney-spec.sh` -- chain spec generators
- `verify-bundle.sh` -- self-verification
- `bundle-manifest.toml` -- metadata
- `README.md` -- operator instructions
- `devnet-spec.json`, `test-finney-spec.json` -- pre-built chain specs

## Wire Protocols

### Game State Wire Format

**Location:** `crates/myosu-games-poker/src/wire.rs`, `crates/myosu-games-liars-dice/src/wire.rs`

- Binary encoding with versioned magic bytes (4-byte header)
- Round-trip tested via property-based tests
- Used for checkpoint persistence and network communication

### Pipe Protocol

**Location:** `crates/myosu-play/src/main.rs` (`PipeResponse` enum)

Responses:
- `Action(String)` -- game action
- `Clarify(String)` -- request for clarification
- `Error(String)` -- error message
- `Quit(String)` -- session end

### JSON API Schema

**Location:** `docs/api/game-state.json`

JSON schema for game state representation used in agent integration and
spectator protocol.

## Color and Theme

**Location:** `crates/myosu-tui/src/theme.rs`

Terminal color palette designed for readability on both dark and light terminal
backgrounds. Uses ratatui's `Style` system with named semantic colors rather
than hardcoded ANSI codes.

## Accessibility

**Current state:** Minimal. Terminal-based interaction limits accessibility to
screen-reader-compatible terminal emulators. Pipe mode provides a structured
text alternative that could be consumed by assistive technology.

**Gaps:**
- No high-contrast theme option
- No configurable key bindings
- No screen reader announcements beyond terminal emulator support

## Design Drift Risks

1. **Renderer size.** `NlheRenderer` at 1,551 lines is the largest single file
   in active crates. Complex rendering logic could drift from the shell
   abstraction.
2. **Report format consistency.** Miner and validator reports use similar but not
   identical formatting. No shared report trait or template.
3. **Pipe protocol versioning.** No version header in pipe mode output. Changes
   to the protocol could break agent consumers silently.
