# Build the Promotion Ledger and Benchmark Gate Surface

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, every research game in Myosu has a single-source-of-truth promotion entry in `ops/solver_promotion.yaml`. A machine-checked manifest binary (`crates/myosu-games-canonical/examples/promotion_manifest.rs`) joins the static ledger with live code-reported bundle support and prints a table showing which games are only routed, which are benchmarked, and which are promoted. A shell harness (`tests/e2e/promotion_manifest.sh`) fails if any declared promotion tier is not actually supported by code and proofs. This prevents the promotion ledger from becoming a wish list.

## Requirements Trace

- R1: `ops/solver_promotion.yaml` exists with one entry per research game (22 games)
- R2: Each entry has required fields: route, tier, benchmark_surface, benchmark_threshold, artifact_requirement, bundle_support, bitino_target_phase, notes
- R3: `promotion_manifest.rs` example binary prints table joining ledger with code-reported support
- R4: `tests/e2e/promotion_manifest.sh` fails when a declared tier is unsupported by code
- R5: The initial ledger reflects truthful current shipped evidence instead of
  flattening every game back to `routed`
- R6: The manifest binary exits non-zero if the YAML is malformed or missing required fields

## Scope Boundaries

This plan creates the promotion ledger and manifest surface. It does not invent
`promotable_*` claims. It may mark games `benchmarked` when a code-backed
independent benchmark surface already exists. Actual promotion to
`promotable_local` or `promotable_funded` happens in plans 005, 006, and 009.
It does not modify any game solver, miner, validator, or chain code.

## Progress

- [ ] Create `ops/solver_promotion.yaml` with 22 game entries and truthful initial tiers
- [ ] Add serde types for the YAML schema in `promotion_manifest.rs`
- [ ] Implement manifest binary that reads YAML, queries code support, prints table
- [ ] Create `tests/e2e/promotion_manifest.sh` harness
- [ ] Verify `cargo run -p myosu-games-canonical --example promotion_manifest -- --format table` works
- [ ] Verify `bash tests/e2e/promotion_manifest.sh` passes
- [ ] Add to CI `active-crates` job

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: Ledger lives in `ops/` rather than `crates/` because it is an operational artifact, not a code dependency.
  Rationale: The YAML is read at example/test time, not compiled into any crate. Keeping it in `ops/` aligns with the repo's convention for operational state.
  Date/Author: 2026-04-11 / genesis corpus

- Decision: The initial ledger reflects current shipped evidence, not a blanket
  reset to `routed`.
  Rationale: The ledger is supposed to be a truthful live inventory. Dedicated
  games with real independent benchmark surfaces should start at
  `benchmarked`, while games without that evidence remain `routed`. The
  aspirational boundary is `promotable_*`, not `benchmarked`.
  Date/Author: 2026-04-11 / reconciliation pass

## Outcomes & Retrospective

None yet.

## Context and Orientation

The 22 research games are defined in `crates/myosu-games-portfolio/src/game.rs`
as the `ResearchGame` enum. In this repo, that set currently means 20
portfolio-routed games plus the two dedicated promotion targets
`nlhe-heads-up` and `liars-dice`. Kuhn is a dedicated engine in the repo, but
it is not part of the 22-entry research-game promotion ledger. The existing
`crates/myosu-games-portfolio/examples/bootstrap_manifest.rs` already prints a
manifest table with game identity, route, player count, rule file, chain id,
and solver family. The promotion manifest extends that pattern with
promotion-specific fields.

The existing E2E harness pattern uses shell scripts under `tests/e2e/` that run Cargo binaries and assert output patterns. The promotion harness follows the same pattern.

## Plan of Work

1. Create `ops/solver_promotion.yaml` with 22 entries. Each entry uses the
   game slug as key and has the required fields. Initial tiers must follow
   current shipped evidence. For games with existing independent benchmark
   surfaces (poker via `benchmark_scenario_pack.rs`, Liar's Dice via exact
   exploitability), set tier to `benchmarked`. For games without that evidence,
   keep tier at `routed`. No game should claim `promotable_*` yet.

2. Add `crates/myosu-games-canonical/examples/promotion_manifest.rs`:
   - Read and parse `ops/solver_promotion.yaml`
   - For each game, query the canonical crate for snapshot support, strategy binding support, and policy bundle support (once policy.rs from plan 001 exists)
   - Print a table with columns: game, route, tier, benchmark, artifact, bundle_support, code_support
   - Accept `--format table` (default) and `--format yaml` flags
   - Exit non-zero if YAML is malformed

3. Create `tests/e2e/promotion_manifest.sh`:
   - Run the manifest binary
   - Assert all 22 games appear
   - Assert no game claims a tier higher than `benchmarked` initially
   - Assert exit code 0

## Implementation Units

### Unit 1: YAML ledger

Goal: Create the promotion ledger file.
Requirements advanced: R1, R2, R5.
Dependencies: None.
Files to create: `ops/solver_promotion.yaml`.
Tests to add: None (tested by manifest binary).
Approach: Hand-write YAML with 22 entries derived from the `ResearchGame` enum
and current shipped benchmark surfaces.
Test expectation: none -- this is a data file validated by Unit 2.

### Unit 2: Manifest binary and harness

Goal: Implement the machine-checked manifest and E2E harness.
Requirements advanced: R3, R4, R6.
Dependencies: Plan 001 (policy.rs) for bundle_support query. Can stub this field as `false` if plan 001 is not yet complete.
Files to create: `crates/myosu-games-canonical/examples/promotion_manifest.rs`, `tests/e2e/promotion_manifest.sh`.
Files to modify: `crates/myosu-games-canonical/Cargo.toml` (add example entry if needed).
Tests to add: The E2E harness is the test.
Approach: Use `serde_yaml` to parse the ledger, query canonical crate APIs, format table output.
Test scenarios: (a) All 22 games listed, (b) No tier above `benchmarked`, (c)
Malformed YAML → non-zero exit.

## Concrete Steps

Use the repo-root command sequence below to build and check the ledger surface.

## Verification

Use the commands below to prove that the ledger and manifest stay aligned. The
manifest must fail loudly on malformed or overstated YAML claims.

From the repository root:

    cargo run -p myosu-games-canonical --example promotion_manifest -- --format table

Expected output (abbreviated):

    PROMOTION game=nlhe-heads-up route=dedicated tier=benchmarked benchmark=scenario_pack artifact=bootstrap_sparse bundle_support=false
    PROMOTION game=liars-dice route=dedicated tier=benchmarked benchmark=exact_exploitability artifact=checkpoint bundle_support=false
    PROMOTION game=bridge route=portfolio tier=routed benchmark=none artifact=none bundle_support=false
    ...

Then:

    bash tests/e2e/promotion_manifest.sh

Expected: exit 0, all 22 games present.

## Acceptance Criteria

- `ops/solver_promotion.yaml` exists and contains 22 research-game entries.
- The manifest example reads the ledger and prints one row per research game.
- The harness fails if the YAML claims a promotion tier the code cannot support.
- The ledger schema uses repo-local rollout fields, not sibling-repo assumptions.

## Validation and Acceptance

1. `ops/solver_promotion.yaml` exists with exactly 22 top-level game entries.
2. `cargo run -p myosu-games-canonical --example promotion_manifest -- --format table` prints 22 `PROMOTION` lines and exits 0.
3. `bash tests/e2e/promotion_manifest.sh` passes.
4. If any game in the YAML claims `tier: benchmarked` or higher, the manifest
   binary verifies that the declared benchmark surface exists in code.
   Initially, no game should claim above `benchmarked`.

## Idempotence and Recovery

Creating the YAML and example binary is additive. The YAML can be overwritten safely on rerun. The example binary is a standalone Cargo example that does not affect library code.

## Artifacts and Notes

Expected new files:
    ops/solver_promotion.yaml
    crates/myosu-games-canonical/examples/promotion_manifest.rs
    tests/e2e/promotion_manifest.sh

## Interfaces and Dependencies

The manifest binary reads:
- `ops/solver_promotion.yaml` (YAML, serde_yaml)
- `myosu_games_canonical::is_canonical_ten()` for canonical status
- `myosu_games_canonical::policy::PolicyPromotionTier` for tier validation (from plan 001)
- `myosu_games_portfolio::ResearchGame` for game enumeration

New dependency for the example: `serde_yaml` (add to workspace if not present, or use `toml_edit` if YAML is not preferred -- decision to record in Decision Log).

Revision note (2026-04-11 / Codex review): corrected the research-game ledger
scope (Kuhn is not in the 22-entry research set), restored the rollout field to
`bitino_target_phase` to match the active root master plan, and added
compatibility verification/acceptance headings.
