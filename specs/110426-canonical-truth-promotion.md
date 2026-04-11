# Specification: Canonical Truth Registry and Promotion Pipeline

## Objective

Define the current state of the canonical truth registry (`myosu-games-canonical`) and the intended promotion pipeline that will advance research games from `routed` through `benchmarked` to `promotable_local`. This spec separates what exists today from what plans 001-007 intend to build.

## Evidence Status

### Verified facts (code-grounded)

#### Current canonical crate (`myosu-games-canonical`)

- Crate exports: `CanonicalActionSpec`, `CanonicalGameSpec`, `CanonicalStateSnapshot`, `CanonicalStrategyBinding`, `canonical_hash`, `validate_unique_action_ids` — `crates/myosu-games-canonical/src/lib.rs:6-9`
- Re-exports playtrace types: `PlaytraceError`, `PlaytracePolicy`, `PlaytraceReport`, `PlaytraceRequest`, `canonical_ten_playtrace_requests`, `research_playtrace_requests`, `run_playtrace`, `validate_transition_trace` — `crates/myosu-games-canonical/src/lib.rs:15-18`
- `CANONICAL_TEN` is a fixed 10-game subset: NlheSixMax, HanafudaKoiKoi, RiichiMahjong, Bridge, GinRummy, Stratego, OfcChinesePoker, DouDiZhu, Backgammon, Cribbage — `crates/myosu-games-canonical/src/lib.rs:22-33`
- Canonical ten does NOT include NlheHeadsUp, LiarsDice, or KuhnPoker (those are dedicated solvers) — `crates/myosu-games-canonical/src/lib.rs:217-218`
- `canonical_game_spec()` produces spec with game_type, slug, chain_id, ruleset_version (1), display_name, default_players, rule_file — `crates/myosu-games-canonical/src/lib.rs:44-58`
- `canonical_action_specs()` derives action specs from portfolio engine typed challenge — `crates/myosu-games-canonical/src/lib.rs:61-77`
- `canonical_bootstrap_snapshot()` produces `CanonicalStateSnapshot` with public_state, private_state_commitments, legal_actions, terminal flag — `crates/myosu-games-canonical/src/lib.rs:80-120`
- `canonical_bootstrap_strategy_binding()` produces binding with query_hash, response_hash, engine_tier ("rule-aware"), engine_family, quality_summary — `crates/myosu-games-canonical/src/lib.rs:123-156`
- Action IDs use format `{game_slug}.{family}.{action_label}` — `crates/myosu-games-canonical/src/lib.rs:180`
- Game families defined per-game: poker_like, hanafuda, mahjong, trick_taking, gin_rummy, stratego, ofc, shedding, backgammon, cribbage — `crates/myosu-games-canonical/src/lib.rs:188-201`
- CI validates canonical manifest: 10 games, 10 `snapshot=ok` — `.github/workflows/ci.yml:128-135`
- Playtrace tests run in CI: `cargo test -p myosu-games-canonical --quiet playtrace` — `.github/workflows/ci.yml:120`
- Snapshot and binding hashes are stable (deterministic) within same process — test at `crates/myosu-games-canonical/src/lib.rs:312-335`

#### What does NOT exist yet

- No `policy.rs` file in `myosu-games-canonical` — `find crates/myosu-games-canonical/src -maxdepth 1 -type f` lists `lib.rs` and `playtrace.rs` but no policy module
- No `PolicyPromotionTier` enum
- No `CanonicalPolicyBundle` struct
- No `CanonicalPolicySamplingProof` struct
- No `CanonicalPolicyProvenance` struct
- No `ops/solver_promotion.yaml` promotion ledger
- No `tests/e2e/promotion_manifest.sh` harness
- No `outputs/solver-promotion/` directory
- No `NlheArtifactDossier` or `NlheBenchmarkDossier` types
- No promotion manifest example binary

### Recommendations (intended system, from plans 001-007)

- **Plan 001:** Add `crates/myosu-games-canonical/src/policy.rs` with `PolicyPromotionTier` (Routed, Benchmarked, PromotableLocal, PromotableFunded), `CanonicalPolicyBundle`, `CanonicalPolicySamplingProof`, `CanonicalPolicyProvenance`
- **Plan 001:** Probability in bundles as PPM (u32, parts per million) not f64 — determinism and cross-platform stability
- **Plan 001:** Bundle hash via SHA-256 over canonical byte representation
- **Plan 001:** Functions: `verify_policy_bundle()`, `sample_policy_action()`, `compute_bundle_hash()`
- **Plan 002:** `ops/solver_promotion.yaml` with 22 game entries (route, tier, benchmark_surface, benchmark_threshold, artifact_requirement, bundle_support, bitino_target_phase, notes)
- **Plan 002:** `crates/myosu-games-canonical/examples/promotion_manifest.rs` joins ledger with code-reported support
- **Plan 002:** `tests/e2e/promotion_manifest.sh` CI gate — fails when declared tier unsupported by code
- **Plan 002:** Initial tiering: NLHE and Liar's Dice at `benchmarked`, all others at `routed`
- **Plan 004:** `NlheArtifactDossier` and `NlheBenchmarkDossier` types in `crates/myosu-games-poker/src/artifacts.rs`
- **Plan 004:** External artifacts referenced by SHA-256 hash, not checked into repo (7-11GB infeasible)
- **Plan 004:** Reuse existing `benchmark_scenario_pack.rs` reference surface
- **Plan 005:** NLHE promotion to `promotable_local` requires pinned artifact + benchmark dossier provenance
- **Plan 005:** Sparse bootstrap artifacts explicitly remain negative fixtures (cannot produce `promotable_local` claims)
- **Plan 006:** Liar's Dice promotion uses exact exploitability (not scenario pack), tree size `1 << 10`
- **Plan 009:** Cribbage deepening to `benchmarked` (not `promotable_local`) using portfolio engine

### Hypotheses / unresolved questions

- Whether policy bundle generalization to portfolio games is feasible without dedicated solver crates is open
- Whether the 22-game ledger count is correct (plans say 22, canonical ten is 10, dedicated are 3; plans exclude Kuhn from 22-entry set)
- Whether a promotion-grade external NLHE artifact already exists is unproven in this repository; NLHE promotion must fail closed until a pinned artifact hash, manifest, and benchmark dossier are verified

## Acceptance Criteria

### Current state (must hold now)

- `CANONICAL_TEN` contains exactly 10 games
- `canonical_ten_specs()` returns 10 specs with unique slugs
- Every canonical game builds a snapshot and a strategy binding without error
- Binding engine_tier is "rule-aware" for all canonical games
- Snapshot and binding hashes are deterministic (two calls produce identical hashes)
- Strategy response actions map 1:1 to canonical action specs
- Renderer completion labels map 1:1 to canonical action specs
- Canonical manifest CI gate: `CANONICAL_GAME` count = 10, `snapshot=ok` count = 10
- Playtrace tests pass

### Future state (after plans 001-007 land)

- `crates/myosu-games-canonical/src/policy.rs` exists with all specified types
- At least 6 policy unit tests passing (plan 003 gate)
- `ops/solver_promotion.yaml` has 22 entries (plan 002)
- `bash tests/e2e/promotion_manifest.sh` passes (plan 002)
- NLHE and Liar's Dice both at `promotable_local` in YAML (plan 007 gate)
- Both have verified bundles under `outputs/solver-promotion/` (plan 007 gate)
- Sparse bootstrap artifacts are rejected as promotion inputs (plan 005 negative test)

## Verification

```bash
# Current state tests
SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet

# Playtrace tests specifically
SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet playtrace

# Canonical manifest gate
manifest="$(SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
  --example canonical_manifest)"
echo "$manifest" | grep -c '^CANONICAL_GAME '   # expect 10
echo "$manifest" | grep -c 'snapshot=ok'          # expect 10

# E2E harnesses
bash tests/e2e/canonical_ten_play_harness.sh
bash tests/e2e/research_play_harness.sh

# Future (after plans 001-003 land):
# test -f crates/myosu-games-canonical/src/policy.rs
# test -f ops/solver_promotion.yaml
# bash tests/e2e/promotion_manifest.sh
```

## Open Questions

1. **Canonical ten vs. 22 research games:** The canonical ten is a subset of the 22 research games. Plans 002 tracks all 22 in the promotion ledger. Does the canonical ten set expand, or does it remain a fixed migration batch distinct from the promotion ledger?
2. **Policy bundle generalization:** Plans 001-005 build policy bundles for dedicated solvers (NLHE, Liar's Dice). Plan 009 reaches `benchmarked` for cribbage but not `promotable_local`. What's the path from portfolio-routed game to policy bundle?
3. **Kuhn's role in promotion:** Kuhn is explicitly excluded from the 22-game research set in plans. It has an exact solver but no promotion path. Is it a test fixture only, or does it have product value?
4. **PPM precision:** Plan 001 recommends PPM (u32) for probability representation. Is 6 decimal places (1 part per million) sufficient for all game action distributions? Some games may have very small probabilities.
5. **Bundle versioning:** The policy bundle contract is intended to be durable and consumed by sibling repos (Bitino). What's the versioning/evolution strategy?
6. **Promotion ledger drift prevention:** Plan 002 says the ledger "must reflect shipped evidence, not aspirational reset to routed." What CI mechanism prevents manual YAML edits that overstate tier?
