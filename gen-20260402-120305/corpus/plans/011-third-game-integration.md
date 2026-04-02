# 011 - Third Game Integration

## Purpose / Big Picture

Multi-game architecture was proven with Liar's Dice (verified). Adding a third
game further validates extensibility with a structurally different game. This
plan adds a third game without modifying poker or Liar's Dice code.

## Context and Orientation

Current multi-game surface:
- `myosu-games/src/traits.rs`: `GameType` enum with `NlheHeadsUp`, `NlheSixMax`,
  `LiarsDice`, `Custom(String)` variants
- Adding a game requires: new crate, implement traits, add renderer, wire
  protocol, register in `myosu-play`
- Research reports cover 15+ games. Kuhn Poker or Leduc are smallest candidates.

## Architecture

```
crates/myosu-games-kuhn/     # New crate
├── src/
│   ├── lib.rs               # Module exports
│   ├── game.rs              # Game state machine
│   ├── renderer.rs          # TUI renderer
│   ├── solver.rs            # Solver (exact for Kuhn)
│   └── wire.rs              # Wire protocol
└── Cargo.toml
```

## Progress

- [x] (pre-satisfied) M1. Multi-game architecture proven
  - Surfaces: `crates/myosu-games-liars-dice/`
Proof command: `cargo test -p myosu-games-liars-dice --quiet`

### Milestone 2: Scaffold third game crate

- [ ] M2. Create crate with game state machine
  - Surfaces: `crates/myosu-games-kuhn/` (new)
  - What exists after: Crate with state machine, `GameType` variant added.
  - Why now: Validates architecture scales.
Proof command: `cargo test -p myosu-games-kuhn --quiet`
  - Tests: State machine tests pass

### Milestone 3: Renderer and wire protocol

- [ ] M3. TUI renderer and wire protocol for third game
  - Surfaces: `crates/myosu-games-kuhn/src/renderer.rs`, `wire.rs`
  - What exists after: Renders in TUI, wire round-trips.
  - Why now: Full integration requires both.
Proof command: `cargo test -p myosu-games-kuhn --quiet -- wire`
  - Tests: Wire round-trip, renderer construction

### Milestone 4: Register in play and CI

- [ ] M4. Add to myosu-play game selection and CI
  - Surfaces: `crates/myosu-play/src/main.rs`, `.github/workflows/ci.yml`
  - What exists after: `myosu-play --game kuhn` works. CI tests it.
  - Why now: Proves integration path is clean.
Proof command: `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test`
  - Tests: Smoke test passes

## Surprises & Discoveries

- Kuhn Poker has 12 information sets, solves exactly. No MCCFR needed.
- `Custom(String)` variant exists but third game should use named variant.

## Decision Log

- Decision: Kuhn Poker over larger games (Mahjong, Bridge).
  - Why: Goal is architecture validation, not production game. Smallest game
    maximizes signal.
  - Failure mode: Too trivial to reveal issues.
  - Mitigation: Still exercises all traits.
  - Reversible: yes

## Validation and Acceptance

1. Third game crate with passing tests.
2. Zero modifications to poker or Liar's Dice.
3. Smoke test in play.
4. CI gates third game.

## Outcomes & Retrospective
_Updated after milestones complete._
