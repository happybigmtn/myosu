# `sdk:core` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP**

The lane is correctly scoped. The source spec (031626-19-game-engine-sdk.md) is coherent with the existing `games:traits` and `games:multi-game` lanes. The SDK surface is a meta-crate wrapping already-trusted `myosu-games` traits — no new trust assumptions beyond what `games:traits` already established. No spec reopening is needed.

The lane is entirely greenfield (`myosu-sdk` crate does not exist), but this is normal for a bootstrap lane. The spec is specific enough that implementation ambiguity is low: the scaffold template stubs, the compliance test assertions, and the registration CLI are all well-specified in 031626-19.

---

## Proof Expectations

The following commands must all exit 0 before the lane is considered complete:

```bash
# Slice 1 — SDK crate (SDK-01)
cargo build -p myosu-sdk
cargo test -p myosu-sdk
cargo test -p myosu-sdk -- --include-ignored  # no tests yet, but exits 0

# Slice 2 — test harness (SDK-03)
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
cargo test -p myosu-sdk testing::tests::broken_game_fails_zero_sum_check
cargo test -p myosu-sdk testing::tests::convergence_test_detects_non_convergence

# Slice 3 — scaffold (SDK-02)
cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
cargo test -p myosu-sdk scaffold::tests::generated_tests_fail_with_todo
cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory

# Slice 4 — registration CLI (SDK-04)
cargo test -p myosu-sdk register::tests::register_help_output
cargo test -p myosu-sdk register::tests::connection_timeout_error

# Slice 5 — documentation (SDK-05)
# docs/sdk/quickstart.md exists and is non-empty
# docs/sdk/trait-reference.md exists and is non-empty
# docs/sdk/registration.md exists and is non-empty
```

---

## Remaining Blockers

### Blocker 1: `myosu-sdk` Crate Is Entirely Greenfield (Critical)

**Location**: `crates/myosu-sdk/` does not exist.

**What must happen**: The implementation lane must create all source files. This is normal for a bootstrap lane — not a design problem.

**Risk if ignored**: Lane cannot progress at all.

---

### Blocker 2: `CfrGame: Copy` Constraint Affects Scaffold Templates (High)

**Location**: Scaffold-generated `src/game.rs` stub; `crates/myosu-sdk/src/scaffold/templates.rs`

The scaffold template must generate `CfrGame` impls that satisfy `CfrGame: Copy`. The bid history for games like Liar's Dice must use fixed-size arrays with sentinel, not `Vec`. This constraint propagates to the scaffold template generation.

**Risk if ignored**: Generated games may compile but fail at runtime with unexpected `Copy` constraint violations.

---

### Blocker 3: `games:traits` Dependency on Absolute Robopoker Paths (High)

**Location**: `crates/myosu-games/Cargo.toml` lines 16–17

`myosu-sdk` depends on `myosu-games`, which still uses absolute path dependencies on robopoker at `/home/r/coding/robopoker/`. Until `games:traits` completes Slice 1 (path-to-git migration), `myosu-sdk` will also carry this portability constraint.

**What must happen**: `myosu-sdk` can be built with the current path dependencies, but Slice 1 of `games:traits` (git rev migration) is a prerequisite for clean CI and publishing.

**Risk if ignored**: `cargo test -p myosu-sdk` will only work for developers with robopoker checked out at the exact same path. This is acceptable for internal development but blocks community contribution.

---

### Blocker 4: On-Chain Registration Extrinsic Not Yet Implemented (Medium)

**Location**: `crates/myosu-chain/pallets/game-solver/`

The `register_game_type` extrinsic referenced in AC-SDK-04 is not yet implemented. The SDK CLI side can be written (Slice 4), but it cannot be fully integration-tested without the chain pallet.

**What must happen**: SDK registration CLI can be written with a mock or skip integration test. Chain-side extrinsic is owned by `chain:pallet` lane.

**Risk if ignored**: SDK registration CLI is incomplete from a full-stack perspective, but the CLI argument parsing and error handling can still be verified.

---

## Risks the Implementation Lane Must Preserve

1. **Scaffold-generated crate compiles**: The generated `myosu-games-<name>/Cargo.toml` must correctly express `myosu-sdk` dependency with `features = ["tui"]`. The generated stub `todo!()` bodies must not break compilation.

2. **Test harness uses existing RPS reference**: The compliance harness tests against `rbp-mccfr::rps::RpsSolver`, not a new mock. This is the existing reference implementation. Preserving this avoids duplicating a working test target.

3. **`CfrGame: Copy` is a hard constraint**: The scaffold template for game state must use fixed-size arrays for variable-length data (bid history). This constraint comes from robopoker's `CfrGame` trait bound. The implementation lane must enforce this in the template.

4. **`tui` feature gates `GameRenderer`**: The scaffold generates `renderer.rs` only when `tui` feature is enabled. Developers building headless miners should be able to omit TUI rendering.

---

## Risks the Implementation Lane Should Reduce

1. **Scaffold must refuse to overwrite**: `myosu init --game existing-name` on an existing directory must fail with a clear error, not silently overwrite. This is a correctness requirement.

2. **Test harness should be reusable across n-player games**: The `assert_nplayer_game_valid` variant (for `NPlayerGame` trait) should also be implemented, even though `NPlayerGame` is not yet in `myosu-games`. Design the harness to be extensible to n-player.

3. **Registration CLI should have a 5-second connection timeout**: Long-running or hanging connection attempts block the developer experience. The timeout must be enforced.

---

## Is the Lane Ready for an Implementation-Family Workflow Next?

**Yes — unconditionally.**

The specification is stable. The implementation lane can begin with Slice 1 (SDK crate skeleton) immediately. The source spec (031626-19) has low ambiguity — the scaffold template structure, the test harness assertions, and the registration CLI flags are all explicitly specified.

The conditions for proceeding are minimal:

1. Slice 1 (SDK crate) must precede all other slices
2. Slice 2 (test harness) and Slice 3 (scaffold) are independent and can run in parallel
3. Slice 4 (registration CLI) is independent of slices 2 and 3
4. Slice 5 (docs) is independent of all code slices

No design reopening is needed. The `myosu-sdk` crate is a meta-crate — it adds no new game logic, only wraps existing `myosu-games` traits. The trust model is inherited directly from `games:traits`.

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | `sdk:core` depends on `myosu-games` (a kept leaf). The dependency is additive — sdk:core re-exports `myosu-games` types, doesn't modify them. |
| `games:multi-game` | Independent. Multi-game implements a concrete game engine; sdk:core provides the SDK template for that game and future games. |
| `games:poker-engine` | The poker-engine will eventually be re-implemented using the SDK scaffolding (or at least depend on `myosu-sdk`). No direct dependency for bootstrap. |
| `chain:pallet` | SDK-04 (registration CLI) calls `register_game_type` extrinsic. The extrinsic signature must be coordinated with `chain:pallet` lane before full integration testing. |
| `services:miner` | Miners will depend on `myosu-sdk` for trait types. Not needed for SDK bootstrap. |
| `services:validator-oracle` | Validators use the exploitability API validated by the SDK test harness. Not needed for SDK bootstrap. |
| `product:play-tui` | The scaffold generates `GameRenderer` implementations. `sdk:core` and `product:play-tui` coordinate on the `GameRenderer` trait interface. |

The `games:traits` lane (kept, unblocked) is the only hard prerequisite. All other lanes are independent for the purposes of SDK bootstrap.
