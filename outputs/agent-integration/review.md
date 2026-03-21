# `agent-integration` Review

## Judgment Summary

**KEEP the product decision surface. REOPEN stale review details. RESET
nothing.**

The product frontier is ready to move into an implementation family. The
remaining blockers are product-owned missing implementation surfaces, not
another upstream restart. What does need reopening is the honesty of a few
review details inside the current `play:tui` and `agent:experience` artifacts.

## Keep

### 1. `fabro/programs/myosu-product.yaml`

Keep the current product frontier boundary. It already cleanly separates
product work from bootstrap, platform, services, and recurring oversight.

### 2. `outputs/play/tui/spec.md` and `outputs/play/tui/review.md`

Keep the lane boundary and slice ordering. The `myosu-play` binary, poker
renderer, training table, blueprint backend, and solver advisor are still the
correct first product implementation path.

### 3. `outputs/agent/experience/spec.md` and `outputs/agent/experience/review.md`

Keep the agent-facing scope:

- persistent agent context
- reflection channel
- narration mode
- append-only journal
- lobby/game selection
- spectator relay and spectator screen

The boundary is still sound even where some proof details have drifted.

### 4. Trusted upstream code surfaces

Keep these as the proven base for product delivery:

- `crates/myosu-games/`
- `crates/myosu-tui/src/schema.rs`
- `crates/myosu-tui/src/pipe.rs`
- `crates/myosu-tui/src/renderer.rs`

## Reopen

### 1. Stale robopoker blocker language in product artifacts

The current tree contradicts the blocker text in reviewed product artifacts.

Current code truth:

- `crates/myosu-games/Cargo.toml` already uses git+rev robopoker dependencies.
- `cargo test -p myosu-games` passes in this worktree.

Artifacts that should be reopened on their next edit:

- `outputs/agent/experience/review.md`
- `outputs/agent/experience/spec.md`
- `outputs/play/tui/review.md`
- `outputs/play/tui/spec.md`

These files still talk about absolute path dependencies as if they were
unresolved. That is now stale.

### 2. Stale test-count claims in `agent:experience`

Current proof in this worktree:

- `cargo test -p myosu-tui schema::tests` passed with 12 tests.
- `cargo test -p myosu-tui pipe::tests` passed with 5 tests.

The `agent:experience` artifacts still claim 16 schema tests and 6 pipe tests.
That mismatch is small, but it matters because Raspberry treats outputs as
durable supervisory truth.

### 3. Mistyped proof commands in `outputs/agent/experience/spec.md`

The proof block currently contains `cargo test -p myyosu-tui ...` for journal
tests. That typo should be corrected before the lane becomes an implementation
source of truth.

## Reset

Nothing in the frontier needs a reset.

- No reviewed product lane has been invalidated.
- No upstream reviewed lane lost proof.
- No restart-style re-bootstrap is justified by the current evidence.

## Evidence Gathered

### Commands Run

- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-games`
  Result: pass; 10 unit tests and 4 doctests.
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui schema::tests`
  Result: pass; 12 tests.
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui pipe::tests`
  Result: pass; 5 tests.

### Existence Checks

- `crates/myosu-play/`: missing
- `crates/myosu-games-poker/`: missing
- `docs/api/game-state.json`: present
- `crates/myosu-tui/src/schema.rs`: present

### Program-Family Check

There is no product implementation family today. The repo has:

- a product bootstrap program: `fabro/programs/myosu-product.yaml`
- a traits implementation family: `fabro/programs/myosu-games-traits-implementation.yaml`

It does not yet have the product-side equivalent.

## Decision

**Product needs an implementation family next. It does not need another
upstream unblock first.**

The true current dependency picture is:

1. `tui:shell` is already reviewed and trusted.
2. `games:traits` is not only reviewed; it has already completed its first
   implementation and verification slice.
3. `play:tui` and `agent:experience` are both reviewed and both point toward
   missing product crates and modules.
4. The main blocking surface for deeper agent work is the missing
   `myosu-play` binary, which is a product implementation slice, not an
   upstream bootstrap problem.

## Concrete Next Actions

1. Create a product implementation-family program that mirrors the existing
   `games:traits` implementation family.
2. Start the critical path with `play:tui` Slice 1:
   scaffold `crates/myosu-play/` and prove `cargo build -p myosu-play`.
3. Allow `agent:experience` Slices 1-2 to run as parallel product work if
   capacity exists:
   `agent_context.rs` and `journal.rs` inside `myosu-tui`.
4. Refresh stale blocker and proof text in the reviewed product artifacts when
   the first implementation-family pass touches them.

## Final Call

The honest reviewed slice says “go implement product.” The frontier is no
longer waiting on another upstream unblock; it is waiting on a product
implementation family to exist.
