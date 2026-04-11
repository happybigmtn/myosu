# Specification: Game Solver Core — Traits, Registry, and Solver Implementations

## Objective

Define the current state and intended direction of the myosu game-solving core: the shared trait surface in `myosu-games`, the game registry, the three dedicated MCCFR/CFR solvers, and the 20-game portfolio engine. This spec establishes the contract that miners, validators, gameplay, and the promotion pipeline depend on.

## Evidence Status

### Verified facts (code-grounded)

- `GameType` enum has 23 named variants plus `Custom(String)` — `crates/myosu-games/src/traits.rs:63-248`
- `GameRegistry::supported()` returns 23 `GameDescriptor` entries — `crates/myosu-games/src/registry.rs:43-68`
- CFR traits (`CfrGame`, `CfrInfo`, `CfrEdge`, `CfrTurn`, `Encoder`, `Profile`) are re-exported from `rbp_core` and `rbp_mccfr` — `crates/myosu-games/src/traits.rs:8-9`
- CFR traits are not object-safe (require `Copy + Sized`); all dispatch is enum-based, not `dyn` — documented in AGENTS.md engineering decisions
- `StrategyQuery<I>` and `StrategyResponse<E>` are generic over game info/edge types — `crates/myosu-games/src/traits.rs:283-337`
- `StrategyResponse::is_valid()` checks probabilities sum to ~1.0 — `crates/myosu-games/src/traits.rs:316-322`
- `GameConfig` bundles `GameType`, `num_players`, and `GameParams` — `crates/myosu-games/src/traits.rs:17-57`
- `GameParams` has variants for NlheHeadsUp (stack_bb, ante_bb), LiarsDice (num_dice, num_faces), KuhnPoker, and Custom — `crates/myosu-games/src/traits.rs:250-277`
- Robopoker fork pinned at rev `047163...` in `Cargo.toml` with `serde` feature — `Cargo.toml` workspace dependencies
- Poker checkpoint format: 4-byte magic `"MYOS"` + 4-byte version (currently `1`) + bincode payload — `crates/myosu-games-poker/src/solver.rs:20-21`
- Liar's Dice checkpoint format also uses `"MYOS"` + version `1` + bincode payload, while Kuhn's exact-solver checkpoint uses `"MYOK"` + version `1` and no bincode payload — `crates/myosu-games-liars-dice/src/solver.rs:21-22`, `crates/myosu-games-kuhn/src/solver.rs:10-12`
- `NlheInfoKey` provides wire-safe information set key (subgame: u64, bucket: i16, choices: u64) — `crates/myosu-games-poker/src/robopoker.rs:42-76`
- Kuhn solver is an exact CFR solver with no MCCFR training — `crates/myosu-games-kuhn/src/solver.rs`
- Liar's Dice solver uses CFR with tree size `1 << 10` — `crates/myosu-miner/src/training.rs` (`LIARS_DICE_SOLVER_TREES`)
- Each dedicated solver crate has additive-constraint tests verifying no cross-game imports — `crates/myosu-games-kuhn/src/lib.rs:38-54`, `crates/myosu-games-liars-dice/src/lib.rs:42-53`
- Portfolio crate defines `ResearchGame` enum with 22 entries (20 portfolio-routed + 2 dedicated) — `crates/myosu-games-portfolio/src/lib.rs`
- Portfolio solver answers via `answer_typed_challenge()` and `answer_game()` — `crates/myosu-games-portfolio/src/lib.rs`
- `EngineTier` and `EngineAnswer` types govern portfolio quality reporting — `crates/myosu-games-portfolio/`
- All portfolio engines are `rule-aware` tier (not trained CFR) — verified by canonical binding test asserting `engine_tier == "rule-aware"` — `crates/myosu-games-canonical/src/lib.rs:249`
- Serialization roundtrip tests exist for `myosu-games` — CI job `active-crates` runs `cargo test -p myosu-games --quiet serialization_roundtrip`
- `CanonicalGameSpec`, `CanonicalActionSpec`, `CanonicalStateSnapshot`, `CanonicalStrategyBinding` types exist in `myosu-games` (re-exported by canonical crate) — `crates/myosu-games/src/lib.rs`
- `CANONICAL_TEN` is a fixed 10-game subset of the 22 research games — `crates/myosu-games-canonical/src/lib.rs:22-33`

### Recommendations (intended system)

- Policy bundle types (`CanonicalPolicyBundle`, `PolicyPromotionTier`, `CanonicalPolicySamplingProof`, `CanonicalPolicyProvenance`) should be added to `crates/myosu-games-canonical/src/policy.rs` per plan 001
- Probability in policy bundles should use PPM (parts per million, `u32`) not `f64` for determinism and cross-platform stability
- Bundle hash should be SHA-256 over canonical byte representation (not JSON serialization order)
- Dedicated solvers for NLHE and Liar's Dice should reach `promotable_local` tier before portfolio games attempt same

### Hypotheses / unresolved questions

- Whether bincode 1.3.3 in checkpoint/wire paths should be migrated to bincode 2.x or postcard (SEC-001 / RUSTSEC-2025-0141) is unresolved
- Whether `NlheInfoKey` wire format will need versioning beyond the checkpoint magic bytes is open
- Whether portfolio games will ever use trained CFR (not just rule-aware engines) is undecided

## Acceptance Criteria

- `GameRegistry::supported()` returns exactly 23 descriptors covering all named `GameType` variants
- `GameType::from_bytes()` and `GameType::to_bytes()` roundtrip for every supported game type
- `StrategyResponse::is_valid()` returns false when action probabilities do not sum to ~1.0
- Each dedicated solver crate (`myosu-games-poker`, `myosu-games-kuhn`, `myosu-games-liars-dice`) compiles without importing any other game crate
- Poker checkpoint loading rejects payloads without `"MYOS"` magic bytes
- Kuhn solver produces exact Nash equilibrium (exploitability = 0.0 within float precision)
- Liar's Dice solver trains and checkpoints within bounded iteration count
- Portfolio solver answers typed challenges for all 20 portfolio-routed games
- `canonical_bootstrap_snapshot()` produces valid snapshot for each of the 10 canonical games
- `canonical_bootstrap_strategy_binding()` reports `engine_tier = "rule-aware"` for all canonical games
- Snapshot and binding hashes are deterministic (stable across repeated calls in same process)
- `cargo test -p myosu-games --quiet serialization_roundtrip` passes

## Verification

```bash
# Trait and registry tests
cargo test -p myosu-games --quiet

# Dedicated solver tests (additive constraint + solver logic)
cargo test -p myosu-games-kuhn --quiet
cargo test -p myosu-games-poker --quiet
cargo test -p myosu-games-liars-dice --quiet

# Portfolio solver tests
cargo test -p myosu-games-portfolio --quiet core

# Canonical registry tests
cargo test -p myosu-games-canonical --quiet

# Canonical manifest (10 games, 10 snapshot=ok)
manifest="$(cargo run --quiet -p myosu-games-canonical --example canonical_manifest)"
grep -c '^CANONICAL_GAME ' <<<"$manifest"  # expect 10
grep -c 'snapshot=ok' <<<"$manifest"       # expect 10

# Additive constraint (no cross-game imports)
cargo tree -p myosu-games-kuhn --edges normal | grep -v 'myosu-games-poker\|myosu-games-liars-dice'
cargo tree -p myosu-games-liars-dice --edges normal | grep -v 'myosu-games-poker\|myosu-games-kuhn'
```

## Open Questions

1. **Bincode migration path:** Direct bincode 1.3.3 usage in wire/checkpoint paths is flagged under SEC-001 (RUSTSEC-2025-0141). Should the migration target bincode 2.x, postcard, or another codec? Decision affects all three dedicated solver wire formats, the portfolio wire format, poker checkpoints, Liar's Dice checkpoints, and poker artifact encoding.
2. **Checkpoint versioning beyond magic bytes:** Current checkpoint formats use version `1`, but Kuhn's exact-solver checkpoint is header-only while poker and Liar's Dice carry bincode payloads. What triggers version `2`? Is there a migration reader for old checkpoints?
3. **Portfolio engine tier advancement:** Can a portfolio-routed game ever reach `benchmarked` or `promotable_local` without a dedicated solver crate? Plan 009 (cribbage) attempts `benchmarked` using the existing portfolio engine.
4. **`GameType::Custom` governance:** Custom games bypass the registry's `builtin` flag. How are custom games validated on-chain? The pallet currently accepts arbitrary byte strings.
5. **Robopoker fork lifecycle:** INV-006 requires fork documentation in `docs/robopoker-fork-changelog.md`. What triggers a rebase against upstream? The fork coherence CI job is `continue-on-error: true` (advisory only).
