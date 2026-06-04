# Changelog

All notable operator-facing changes to myosu are documented in this file.

The format is based on Keep a Changelog, and the project follows Semantic
Versioning during stage-0 and stage-1. The repo already carries incremental
`0.0.x` checkpoint tags for task-completion snapshots on `trunk`; this file
tracks supported operator-facing releases starting at `0.1.0`.

## Maintenance Process

- Update `Unreleased` in the same change that alters operator-facing behavior,
  proofs, docs, or bundle contents.
- Write entries from the operator perspective. Prefer concrete impact, proof
  surfaces, and manual actions over internal refactor detail.
- During `0.x`, minor releases may contain breaking operator changes and patch
  releases should remain operator-compatible.
- When cutting a release, move the relevant items from `Unreleased` into a
  dated version section and keep required operator action items explicit.

## [Unreleased]

### Added

- `ops/release.sh` now provides a single release wrapper for validating a
  `vX.Y.Z` tag, generating changelog-derived release notes, and materializing a
  versioned operator bundle before a real tag is created.
- `docs/operator-guide/upgrading.md` now documents the current `0.x`
  release contract, the minimum operator notice windows for patch vs breaking
  minor releases, the manual upgrade path, and the repo-owned rollback
  procedure.
- NEM-001B independent poker quality reference test (mix-ladder substitute).
  `crates/myosu-validator/src/validation.rs` gains
  `PokerQualityBenchmarkPoint` + `POKER_REFERENCE_LADDER = [0.0, 0.25, 0.5,
  0.75, 1.0]` + `POKER_USEFUL_REFERENCE_MATCH_RATIO = 0.95` +
  `PokerQualityBenchmarkReport` + `poker_quality_benchmark_points`;
  `crates/myosu-games-poker/src/benchmark.rs` gains
  `mixed_bootstrap_reference_solver` / `mixed_bootstrap_reference_profile` and
  the `POKER_REFERENCE_SCENARIO_COUNT` / `POKER_REFERENCE_SELF_MATCH_COUNT` /
  `POKER_REFERENCE_SELF_MATCH_L1` anchors. The new
  `crates/myosu-validator/examples/poker_quality_benchmark.rs` example prints
  the ladder on demand, `tests/e2e/poker_quality_benchmark.sh` is the
  CI-grade proof (6 sub-checks, all green), and
  `tests/e2e/miner_convergence_doc_keeps_truthful_thresholds.sh` now carries
  a 7th sub-check that pins the live `poker_quality_benchmark` configuration
  against the operator doc. The mix-ladder is the truthful F-003 poker
  substitute against the checked-in sparse bootstrap encoder (positive-iteration
  MCCFR training still fails upstream with `isomorphism not found`; the
  PROMOTE-001 external artifact supply is the unblock for a real
  positive-iteration poker exploitability ladder).
- F-016 Stratego benchmark dossier + rule-aware scenario pack (the 10th
  portfolio-game-promotion slice, first `state-aware belief-scout heuristic`
  engine family — the previous nine portfolio slices all shared a prior engine
  family; Stratego is the first dossier slice to open a new family).
  `crates/myosu-games-portfolio/src/core/stratego.rs` gains `StrategoScenario`
  (22 rows; coverage buckets `scout×8` / `advance-piece×8` / `place-safe×6` with
  the tightest margin pinned at 0.65 on `place-safe-no-targets-mid-bomb`) and
  the const `STRATEGO_SCENARIO_PACK` exposed through `stratego_scenario_pack()`.
  The new crate module `crates/myosu-games-portfolio/src/stratego_benchmark.rs`
  defines `StrategoBenchmarkDossier` with the full promotion surface
  (`benchmark_id`, `benchmark_method`, `metric_name`, `metric_value`,
  `threshold`, `passing`, `scenario_count`, `recommendation_count`,
  `engine_family`, `engine_tier`, `rule_file`, `scenario_hash`,
  `recommendations: BTreeMap<String, String>`) and a deterministic SHA-256 over
  the canonical scenario/answer table sorted by `scenario_id`. The new example
  binary `crates/myosu-games-portfolio/examples/stratego_benchmark.rs` runs the
  live rule-aware engine against the pack and writes the JSON dossier to
  `outputs/solver-promotion/stratego/stratego-benchmark-dossier.json`
  (overridable via `MYOSU_STRATEGO_BENCHMARK_OUTPUT`). The new e2e proof
  harness `tests/e2e/stratego_benchmark_dossier.sh` (5 sub-checks) is wired
  into `.github/workflows/ci.yml` as a new `Verify F-016 Stratego benchmark
  dossier` step in the `active-crates` job. `ops/solver_promotion.yaml`
  advances the `stratego` row from `tier: routed` to `tier: benchmarked`
  (matching the F-015 HwatuGoStop promotion pattern); the live promotion
  manifest now shows `slug=stratego tier=benchmarked
  code_bundle_support=benchmarked benchmark_surface=rule_aware_scenario_pack`.
  Promotion tier remains `benchmarked` (not `promotable_local`) — the policy
  bundle builder is designed for dedicated games, the same scope boundary
  every prior portfolio-game-promotion slice has landed under.

## [0.1.0] - 2026-04-02

### Added

- A runnable stage-0 local loop proving chain, miner, validator, and gameplay
  integration on one machine, including end-to-end emission and validator
  determinism checks.
- Operator onboarding documentation covering quickstart, architecture,
  troubleshooting, and named-network bundle/bootstrap preparation.
- Named-network bundle and bootnode preparation surfaces through
  `.github/scripts/prepare_operator_network_bundle.sh`,
  `.github/scripts/check_operator_network_bootstrap.sh`, and
  `ops/deploy-bootnode.sh --dry-run`.
- Security disclosure guidance and an upstream CVE tracking process for the
  inherited chain and solver dependencies.

### Changed

- Reduced the default chain runtime to the stage-0 pallet surface and stripped
  Frontier/EVM service dependencies from the node binary.
- Reduced the default `pallet-game-solver` dispatch surface to the extrinsics
  exercised by the live stage-0 loop.
- Established `0.1.0` as the first supported operator-facing release baseline;
  earlier `0.0.x` tags remain internal task-checkpoint markers on `trunk`.

### Fixed

- Removed legacy root-network stake weighting from stage-0 emission
  distribution so subnet-local emissions follow the single-token model.
- Added deterministic Yuma and local-devnet proofs so repeated validator runs
  converge on the same emission outputs and submitted weights.
- Added tracing subscriber initialization to `myosu-play` so operator
  diagnostics respect `RUST_LOG` without changing the pipe protocol surface.

### Security

- Added a `cargo audit` CI gate with the current inherited-chain advisory
  ignore set documented in repo policy.
- Documented mmap checkpoint safety boundaries in the poker engine and pinned
  the audited robopoker fork delta.
