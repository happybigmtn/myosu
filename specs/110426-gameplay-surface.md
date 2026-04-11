# Specification: Gameplay Surface — Play Binary, TUI Shell, and Renderers

## Objective

Define the current state and intended direction of the human-and-agent-facing gameplay surface: the `myosu-play` binary, the `myosu-tui` shared shell, per-game renderers, and the three interaction modes (TUI, pipe, smoke-test). This spec establishes the contract for how users and agents experience solver-backed gameplay.

## Evidence Status

### Verified facts (code-grounded)

#### Play Binary (`myosu-play`)

- CLI arguments: `--game` (default poker), `--mode` (tui/pipe), `--smoke-test`, `--require-artifact`, `--require-discovery`, `--require-live-query`, `--chain`, `--subnet` — `crates/myosu-play/src/cli.rs`
- `GameSelection` enum matches miner/validator: Poker, Kuhn, LiarsDice + 20 portfolio games — `crates/myosu-play/src/cli.rs`
- Three modes: TUI (interactive), pipe (agent protocol via stdin/stdout), smoke-test (non-interactive validation) — `crates/myosu-play/src/main.rs`
- Smoke test validates preflop → flop progression for poker — CI test `smoke_report_proves_preflop_to_flop_progression` at `.github/workflows/ci.yml:122`
- Kuhn smoke test also runs in CI: `cargo run -p myosu-play --quiet -- --game kuhn --smoke-test` — `.github/workflows/ci.yml:124`
- Blueprint bot module provides trained offline play (no mining) — `crates/myosu-play/src/blueprint.rs`
- Discovery module enables miner discovery on chain — `crates/myosu-play/src/discovery.rs`
- Live module enables live strategy queries from remote miner — `crates/myosu-play/src/live.rs`
- Only poker supports on-chain miner discovery via `--chain`/`--subnet`; non-poker games reject discovery flags — `crates/myosu-play/src/cli.rs:76-79`, `crates/myosu-play/src/cli.rs:208-214`, `crates/myosu-play/src/cli.rs:328-389`
- INV-004: `myosu-play` has NO dependency on `myosu-miner` — CI enforced at `.github/workflows/ci.yml:145-158`
- `AdviceStartupState` covers Success, Empty, Partial states for solver availability — play source

#### TUI Shell (`myosu-tui`)

- Five-panel layout: header, transcript, state, declaration, input — `crates/myosu-tui/src/lib.rs:1-11`
- Game-agnostic: games implement `GameRenderer` trait — `crates/myosu-tui/src/renderer.rs`
- `InteractionState` enum: Neutral, Loading, Empty, Partial, Error, Success — `crates/myosu-tui/src/events.rs`
- Modules: events, input, pipe, renderer, schema, screens, shell, theme — `crates/myosu-tui/src/lib.rs`
- Async event loop using Tokio + ratatui rendering — `crates/myosu-tui/`
- `PipeMode` for non-interactive agent protocol — `crates/myosu-tui/src/pipe.rs`
- Terminal resize handled via `Event::Resize` — `crates/myosu-tui/src/events.rs`
- Game renderers express layout needs via `desired_height()` — `crates/myosu-tui/src/renderer.rs`
- Shell state tests run in CI: `cargo test -p myosu-tui shell_state` — `.github/workflows/ci.yml:121`

#### Game Renderers

- `NlheRenderer` for poker — `crates/myosu-games-poker/src/renderer.rs`
- `KuhnRenderer` for Kuhn poker — `crates/myosu-games-kuhn/src/renderer.rs`
- `LiarsDiceRenderer` for Liar's Dice — `crates/myosu-games-liars-dice/src/renderer.rs`
- `PortfolioRenderer` for all 20 portfolio games — `crates/myosu-games-portfolio/`
- Each renderer wraps a game-specific snapshot type (NlheSnapshot, KuhnSnapshot, LiarsDiceSnapshot, PortfolioSnapshot)

#### E2E Harnesses

- `tests/e2e/canonical_ten_play_harness.sh` — validates canonical 10 games play through — `.github/workflows/ci.yml:136`
- `tests/e2e/research_play_harness.sh` — validates research game play paths — `.github/workflows/ci.yml:137`
- `tests/e2e/research_games_harness.sh` — exercises all 22 research games — `.github/workflows/ci.yml:140`
- `tests/e2e/research_strength_harness.sh` — validates strength profiles — `.github/workflows/ci.yml:143`

### Recommendations (intended system)

- Promotion provenance should become visible in TUI/CLI once policy bundle work lands
- Explicit empty/partial/error-state treatment should be protected in plans
- Pipe mode should be the primary agent interface; it shares the text protocol with humans — AGENTS.md design principle
- Any future solver-backed table UI should define its information architecture before implementation: lobby/ready-room placement, bundle verification status, benchmark/provenance labels, empty/loading/error/success states, and keyboard-only operation

### Hypotheses / unresolved questions

- Whether screen-reader, color/contrast, and copy/paste ergonomics are sufficient is unverified
- Whether pipe mode protocol is stable enough for external agent consumers is unspecified
- Whether portfolio games need game-specific renderers or the generic PortfolioRenderer is sufficient long-term

## Acceptance Criteria

- `cargo run -p myosu-play -- --smoke-test` exits 0 and validates preflop → flop progression
- `cargo run -p myosu-play -- --game kuhn --smoke-test` exits 0
- Pipe mode accepts `quit\n` on stdin and exits cleanly: `printf 'quit\n' | cargo run -p myosu-play -- pipe`
- Every `GameSelection` variant compiles and resolves to a valid renderer
- TUI shell renders five-panel layout without panic on standard terminal sizes
- `InteractionState` transitions from `Loading` to `Success` or `Error` (never hangs in `Loading`)
- `GameRenderer` trait is implemented by NlheRenderer, KuhnRenderer, LiarsDiceRenderer, and PortfolioRenderer
- Canonical ten play harness passes: `bash tests/e2e/canonical_ten_play_harness.sh`
- Research play harness passes: `bash tests/e2e/research_play_harness.sh`
- Research games harness exercises all 22 games: `bash tests/e2e/research_games_harness.sh`
- Research strength harness validates strength profiles: `bash tests/e2e/research_strength_harness.sh`
- `myosu-play` has zero dependency on `myosu-miner` (INV-004)

## Verification

```bash
# Smoke tests
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test

# Unit tests
SKIP_WASM_BUILD=1 cargo test -p myosu-play --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-tui --quiet

# Specific shell state tests
SKIP_WASM_BUILD=1 cargo test -p myosu-tui shell_state

# Progression proof
SKIP_WASM_BUILD=1 cargo test -p myosu-play \
  smoke_report_proves_preflop_to_flop_progression --quiet

# E2E harnesses
bash tests/e2e/canonical_ten_play_harness.sh
bash tests/e2e/research_play_harness.sh
bash tests/e2e/research_games_harness.sh
bash tests/e2e/research_strength_harness.sh

# INV-004
play_tree="$(SKIP_WASM_BUILD=1 cargo tree -p myosu-play --edges normal)"
echo "$play_tree" | grep -q 'myosu-miner' && echo "FAIL" || echo "PASS"
```

## Open Questions

1. **Pipe mode protocol stability:** Is the pipe mode text protocol versioned? External agents consuming it need a stability guarantee. What constitutes a breaking change?
2. **Portfolio renderer adequacy:** The generic `PortfolioRenderer` handles all 20 portfolio games. As games deepen (e.g., cribbage per plan 009), will game-specific renderers be needed?
3. **Live query scope:** Only poker supports `--require-live-query` with chain miner discovery. Should Kuhn and Liar's Dice also support live queries, or is this poker-only by design?
4. **Accessibility verification:** Screen-reader compatibility, color contrast, and keyboard navigation have not been audited. Is this in scope for stage-0?
5. **Agent experience parity:** The pipe mode gives agents the same text protocol as humans. Is this sufficient, or should there be structured (JSON) input/output for richer agent interaction?
