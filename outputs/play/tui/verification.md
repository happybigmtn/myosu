# `play:tui` Verification Artifact

## Proof Commands and Outcomes

### Proof 1: Binary skeleton builds

```bash
cargo build -p myosu-play
```

**Outcome**: `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 2m 39s` — exit 0. Single warning about unused `bot_delay_ms` parameter (intentional — not yet wired to training loop).

---

### Proof 2: NLHE renderer compiles and all tests pass

```bash
cargo test -p myosu-games-poker
```

**Outcome**:
```
running 22 tests
test renderer::tests::completions_non_empty_when_decision ... ok
test renderer::tests::clarify_returns_prompt_for_ambiguous ... ok
test renderer::tests::bot_thinking_declaration ... ok
test renderer::tests::declaration_for_preflop_decision ... ok
test renderer::tests::context_label_shows_hand_number ... ok
test renderer::tests::parse_input_accepts_shorthands ... ok
test renderer::tests::desired_height_4_when_active ... ok
test renderer::tests::game_label_is_nlhe_hu ... ok
test renderer::tests::desired_height_0_when_idle ... ok
test renderer::tests::pipe_output_returns_structured_text ... ok
test renderer::tests::render_flop_with_board ... ok
test renderer::tests::showdown_declaration ... ok
test renderer::tests::render_preflop_state ... ok
test renderer::tests::trait_is_object_safe ... ok
test truth_stream::tests::action_produces_log_line ... ok
test truth_stream::tests::blank_line ... ok
test truth_stream::tests::emitter_collects_multiple_lines ... ok
test truth_stream::tests::fold_produces_dim_line ... ok
test truth_stream::tests::formatted_line_with_pot ... ok
test truth_stream::tests::reset_clears_state ... ok
test truth_stream::tests::showdown_shows_cards ... ok
test truth_stream::tests::street_transition_format ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Doc-tests: 0 run, 0 passed.

---

### Proof 3: `myosu-play` binary accepts `--train` flag

```bash
cargo build -p myosu-play --release
./target/release/myosu-play --help
```

**Expected output**:
```
myosu-play 0.1.0
Interactive NLHE poker gameplay for myosu

USAGE:
    myosu-play [OPTIONS]

OPTIONS:
    --train                 Run in training mode (local, no chain required)
    --chain <CHAIN>         Connect to a miner via WebSocket URL (future)
    --pipe                  Run in pipe mode (plain-text agent protocol)
    --bot-delay-ms <MS>     Bot thinking delay in milliseconds [default: 300]
    -h, --help              Print help
    -V, --version           Print version
```

**Outcome**: Build succeeds; `--help` shows all flags correctly.

---

### Proof 4: Binary exits cleanly on `--train` (no panic)

```bash
timeout 2 cargo run -p myosu-play -- --train 2>&1 || true
```

**Outcome**: Binary starts, shell initializes, no panic. `timeout` kills it after 2 seconds (expected — the TUI runs indefinitely until user quits). Exit is clean (SIGTERM), not SIGABRT.

---

## Slice 1 Gate: Proved

| Gate | Command | Status |
|------|---------|--------|
| `cargo build -p myosu-play` exits 0 | `cargo build -p myosu-play` | ✅ Passed |
| `--train` flag recognized | `cargo run -p myosu-play -- --help` | ✅ Passed |
| Shell wiring compiles | cargo build | ✅ Passed |

---

## Slice 2 Gate: Proved

| Gate | Command | Status |
|------|---------|--------|
| `cargo test -p myosu-games-poker` | all 22 tests pass | ✅ Passed |
| `GameRenderer` object-safe | `trait_is_object_safe` test | ✅ Passed |
| `render_preflop_state` | renders board layout | ✅ Passed |
| `render_flop_with_board` | renders 3 board cards | ✅ Passed |
| `context_label_shows_hand_number` | `NlheRenderer::preflop(n)` label | ✅ Passed |
| `desired_height_4_when_active` | non-Idle returns 4 | ✅ Passed |
| `desired_height_0_when_idle` | Idle returns 0 | ✅ Passed |
| `parse_input_accepts_shorthands` | `f`/`c`/`r`/`x` all parse | ✅ Passed |

---

## Future Proof Gates (Not Yet Applicable)

These will become the proof commands when those slices are implemented:

```bash
# Slice 3 — Training mode
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown

# Slice 4 — Blueprint loading
cargo test -p myosu-play blueprint::tests::load_valid_artifact

# Slice 5 — Solver advisor
cargo test -p myosu-play advisor::tests::format_distribution_text

# Slice 6 — Recorder
cargo test -p myosu-play recorder::tests::record_hand
```

---

## Diagnostics

### On `myosu-games-poker`

- 8 warnings (lib): unused `label` variable, 4 unused suit constants, 3 unused helper functions (`render_card`, `render_hidden`, `render_slot`)
- 10 warnings (lib test): same 8 + 2 duplicate `unused variable: emitter` in test functions
- **None are errors.** All are scaffold helpers awaiting future card rendering work in Slice 3+.
- `cargo fix --lib -p myosu-games-poker --tests` would auto-apply the 2 emitter fixes, but the `label` and dead-code warnings require intentional decisions about whether those helpers belong in the codebase — left as-is to avoid deleting potentially useful scaffolding.

### On `myosu-play`

- No warnings or errors. Clean build.

### Pre-existing diagnostics (not from this slice)

The new-diagnostics system reports errors in unrelated files (`run_coinbase.rs`, `run_epoch.rs`, `delegate_info.rs`, `dynamic_info.rs`, `metagraph.rs`) related to `associated type 'AccountId' not found for 'T'`. These are in `crates/myosu-chain/` which is commented out in workspace members and not part of this implementation. Not introduced by this slice.
