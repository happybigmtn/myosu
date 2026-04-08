# Specification: Game Engine Framework

## Objective

Describe the multi-game trait abstraction layer (`myosu-games`), the concrete game implementations (poker, Liar's Dice, Kuhn), the robopoker fork dependency, and the enum-based dispatch architecture that enables multiple games on one chain.

## Evidence Status

### Verified (code-grounded)

- Trait crate: `crates/myosu-games/src/` with:
  - `lib.rs` (18 lines): Re-exports `registry` and `traits` modules.
  - `traits.rs` (500 lines): Core game types and CFR trait re-exports.
  - `registry.rs`: `GameDescriptor` and `GameRegistry` types.
- Re-exported CFR traits from robopoker (`rbp_core`, `rbp_mccfr`):
  - `Probability` (f64 alias), `Utility` (f64 alias)
  - `CfrEdge`, `CfrGame`, `CfrInfo`, `CfrTurn`, `Encoder`, `Profile`
- Myosu-specific types defined in `traits.rs`:
  - `GameConfig`: `{ game_type: GameType, num_players: u8, params: GameParams }`.
  - `GameType` enum (`traits.rs:63-77`): `NlheHeadsUp`, `NlheSixMax`, `KuhnPoker`, `LiarsDice`, `Custom(String)`. Marked `#[non_exhaustive]`.
  - `GameParams` enum (`traits.rs:158-181`): `NlheHeadsUp { stack_bb, ante_bb }`, `LiarsDice { num_dice, num_faces }`, `KuhnPoker`, `Custom(serde_json::Value)`. Marked `#[non_exhaustive]`.
  - `StrategyQuery<I>`: Generic over the info type `I: Serialize`. Contains `info: I`.
  - `StrategyResponse<E>`: Generic over the edge/action type `E: Serialize`. Contains `actions: Vec<(E, Probability)>`. Validity check: probabilities sum to ~1.0 within epsilon 0.001.
- Canonical byte encoding for on-chain storage (`GameType::to_bytes()`/`from_bytes()`):
  - `NlheHeadsUp` → `b"nlhe_hu"`, `NlheSixMax` → `b"nlhe_6max"`, `KuhnPoker` → `b"kuhn_poker"`, `LiarsDice` → `b"liars_dice"`.
  - Unknown bytes → `Custom(String)` via UTF-8 parse.
- Default player counts: NLHE HU=2, 6max=6, Kuhn=2, Liar's Dice=2, Custom=2.
- Game implementation crates (workspace members in `Cargo.toml:3-7`):
  - `crates/myosu-games-poker`: Full NLHE implementation using `rbp-nlhe` from robopoker fork.
  - `crates/myosu-games-liars-dice`: Second-game proof (different subnet).
  - `crates/myosu-games-kuhn`: Simplified game for testing.
- Robopoker fork: `happybigmtn/robopoker` pinned by revision in workspace `Cargo.toml`.
- INV-006 (`INVARIANTS.md:75-88`): Fork tracks v1.0.0 baseline. Every downstream change documented in `docs/robopoker-fork-changelog.md`. Core MCCFR algorithm changes require review.
- Tests in `traits.rs` include:
  - Serde roundtrip for `GameConfig`, `GameType`, `StrategyResponse`.
  - `GameType::from_bytes` / `to_bytes` roundtrip for all known types + custom.
  - Unicode custom game type roundtrip (`"liars_dice_묘수"`).
  - `StrategyResponse::is_valid()` boundary tests (valid, empty, near-one, invalid).
  - Proptest strategies for `GameType`, `GameConfig`, `StrategyResponse` fuzzing.
- Proof command: `cargo test -p myosu-games-liars-dice --quiet` (`README.md:123`).

### Recommendations (intended future direction)

- The `Custom(String)` / `Custom(serde_json::Value)` variants provide extensibility for future games without code changes.
- ADR records (referenced in `docs/adr/`) document the enum dispatch pattern decision.
- No additional game implementations are planned for stage-0 — three games prove the architecture.

### Hypotheses / Unresolved

- Whether `GameRegistry` enforces uniqueness of `GameDescriptor` identifiers at the type level or at runtime.
- Whether `NlheSixMax` has any concrete implementation or is only a variant definition — the workspace only lists `myosu-games-poker` (heads-up focus).
- The exact robopoker fork revision is pinned in `Cargo.toml` but the specific commit hash was not extracted in this pass.

## Acceptance Criteria

- `cargo check -p myosu-games` succeeds
- `cargo test -p myosu-games --quiet` passes (including proptest fuzzing)
- `cargo test -p myosu-games-poker --quiet` passes
- `cargo test -p myosu-games-liars-dice --quiet` passes
- `cargo test -p myosu-games-kuhn --quiet` passes
- `GameType::from_bytes(gt.to_bytes()) == Some(gt)` for all known variants
- `StrategyResponse::is_valid()` returns true when probabilities sum to 1.0 ± 0.001
- `StrategyResponse::is_valid()` returns true for empty action lists (terminal states)
- `GameConfig` serializes and deserializes via serde_json without loss
- The `#[non_exhaustive]` attribute is present on both `GameType` and `GameParams`

## Verification

```bash
# All game crate tests
cargo test -p myosu-games --quiet
cargo test -p myosu-games-poker --quiet
cargo test -p myosu-games-liars-dice --quiet
cargo test -p myosu-games-kuhn --quiet

# Verify non_exhaustive on enums
grep -n 'non_exhaustive' crates/myosu-games/src/traits.rs

# Verify robopoker fork pin
grep 'happybigmtn/robopoker' Cargo.toml

# Verify canonical byte encodings exist
grep 'fn to_bytes' crates/myosu-games/src/traits.rs
grep 'fn from_bytes' crates/myosu-games/src/traits.rs
```

## Open Questions

- Does `NlheSixMax` have a concrete solver/game implementation, or is it only a type placeholder?
- Is `GameRegistry` used at runtime for dynamic game discovery, or is game dispatch hardcoded per-subnet?
- What is the stability contract for `GameType::to_bytes()` encoding — if the on-chain storage uses these bytes as keys, changing them would be a breaking migration.
- Should `StrategyResponse::is_valid()` epsilon (0.001) be tighter for validator use, or is it intentionally loose for network transport tolerance?
