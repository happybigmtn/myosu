# Unblock the NLHE Benchmark Surface for Promotion

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, the `nlhe-heads-up` game has a truthful benchmark surface that can produce quality evidence suitable for promotion. Currently, the repo-owned bootstrap artifacts are postflop-sampled and the miner rejects positive-iteration training on them. The independent benchmark surface (`crates/myosu-games-poker/examples/benchmark_scenario_pack.rs`) exists but operates on a fixed 80-scenario reference pack, and its relationship to promotion quality thresholds has not been established.

This plan resolves the F-007 / MINER-QUAL-001 blocker by enabling the promotion pipeline to use external pinned artifacts by hash. It does not require the repo to check in large encoder artifacts. Instead, it adds a dossier reader that can point at an external artifact directory, verify its hash, run the benchmark, and produce a benchmark summary that the promotion ledger can consume.

## Requirements Trace

- R1: `crates/myosu-games-poker/src/artifacts.rs` supports loading external artifacts by manifest hash
- R2: A benchmark dossier can be produced from external artifacts using the existing `benchmark_scenario_pack.rs` surface
- R3: The benchmark summary is a structured type that the promotion manifest can read
- R4: The repo-owned sparse artifacts remain unchanged and continue to work for zero-iteration proofs
- R5: The dossier workflow is documented and runnable from the repo root

## Scope Boundaries

This plan adds a parallel "dossier" reader to the poker artifact surface and a benchmark summary writer. It does not change the existing bootstrap artifact shape, does not change the miner training path, does not change the validator scoring path, and does not promote any game. It does not require external artifacts to be checked into the repo.

## Progress

- [ ] Add `NlheArtifactDossier` type to `crates/myosu-games-poker/src/artifacts.rs`
- [ ] Add `load_external_artifacts_by_hash()` function that verifies a manifest hash
- [ ] Add `NlheBenchmarkDossier` type to capture benchmark results
- [ ] Add `write_benchmark_dossier()` function to `crates/myosu-games-poker/src/benchmark.rs`
- [ ] Add unit test: dossier from repo-owned sparse artifacts produces benchmark with quality caveat
- [ ] Add unit test: dossier hash verification rejects tampered manifests
- [ ] Document the dossier workflow in a short prose block in this plan
- [ ] Verify `cargo test -p myosu-games-poker --quiet dossier` passes
- [ ] Verify existing tests still pass: `cargo test -p myosu-games-poker --quiet`

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: External artifacts are referenced by hash, not checked into the repo.
  Rationale: The master plan says "repo-owned artifacts are allowed to remain proof bundles as long as promoted games use pinned promotion dossiers that can point at stronger external artifacts." Checking in 7-11 GB of NLHE encoder data is infeasible.
  Date/Author: 2026-04-11 / master plan

- Decision: The benchmark surface for promotion reuses `benchmark_scenario_pack.rs` rather than building a new exploitability oracle.
  Rationale: The scenario pack already provides an independent reference (not a same-checkpoint self-check). It compares solver answers against a fixed reference pack, which is sufficient for measuring quality improvement across training iterations. A full exploitability oracle would be ideal but is out of scope for this unblock plan.
  Date/Author: 2026-04-11 / genesis corpus

## Outcomes & Retrospective

None yet.

## Context and Orientation

The relevant files are:

- `crates/myosu-games-poker/src/artifacts.rs`: Currently handles `NlheAbstractionArtifactBundle` loading from the repo-owned bootstrap path. The `load_encoder_bundle()` function reads a directory with `manifest.json` and per-street encoder files.
- `crates/myosu-games-poker/examples/bootstrap_artifacts.rs`: Generates the repo-owned sparse artifacts (169 preflop entries, 24 per postflop street, `postflop_complete=false`).
- `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs`: Runs 80 fixed scenarios against a solver checkpoint and reports quality metrics. This is the independent benchmark surface.
- `crates/myosu-games-poker/src/benchmark.rs`: Contains `benchmark_against_bootstrap_reference()` and the `NlheScenarioBenchmarkError` type.
- `crates/myosu-miner/src/training.rs`: The miner's `run_poker_training_batch()` function rejects `--train-iterations > 0` when artifacts have `postflop_complete=false`.

A "dossier" in this context is a structured manifest that points at an artifact directory, records its hash, and captures benchmark results. The dossier can point at either the repo-owned sparse artifacts (which will produce low quality scores) or external stronger artifacts (which will produce higher scores). The promotion pipeline cares about the benchmark evidence, not whether the artifacts are checked in.

## Plan of Work

1. In `crates/myosu-games-poker/src/artifacts.rs`, add an `NlheArtifactDossier` struct that contains: artifact directory path, manifest hash (SHA-256 of `manifest.json`), manifest summary (entries per street, completeness flags), and a `verified: bool` field set by the hash check.

2. Add `load_and_verify_artifact_dossier(dir: &Path) -> Result<NlheArtifactDossier, ArtifactCodecError>` that reads the manifest, computes its hash, and populates the dossier.

3. In `crates/myosu-games-poker/src/benchmark.rs`, add an `NlheBenchmarkDossier` struct that contains: artifact dossier, scenario count, pass count, mean quality score, quality summary, and a `promotion_ready: bool` field.

4. Add `write_benchmark_dossier(dossier: &NlheBenchmarkDossier, output: &Path) -> Result<(), ...>` that serializes the dossier to JSON.

5. Add tests that:
   - Load the repo-owned sparse artifacts, produce a dossier, verify the hash matches
   - Tamper with the manifest and verify hash rejection
   - Run the benchmark on sparse artifacts and verify the dossier captures the result

## Implementation Units

### Unit 1: Artifact dossier type and hash verification

Goal: Add typed dossier loading with hash verification.
Requirements advanced: R1, R4.
Dependencies: None.
Files to modify: `crates/myosu-games-poker/src/artifacts.rs`.
Tests to add: Hash verification, tamper rejection.
Approach: Build on existing `load_encoder_bundle()` pattern, add SHA-256 hash of manifest.json.
Test scenarios: (a) Repo-owned artifacts produce valid dossier, (b) Modified manifest.json → hash mismatch error.

### Unit 2: Benchmark dossier and writer

Goal: Capture benchmark results in a structured dossier.
Requirements advanced: R2, R3, R5.
Dependencies: Unit 1.
Files to modify: `crates/myosu-games-poker/src/benchmark.rs`.
Tests to add: Dossier from sparse artifacts, JSON serialization roundtrip.
Approach: Run existing `benchmark_against_bootstrap_reference()`, capture results into `NlheBenchmarkDossier`, serialize to JSON.
Test scenarios: (a) Sparse artifacts produce dossier with `promotion_ready=false`, (b) Dossier JSON roundtrips cleanly.

## Concrete Steps

Use the repo-root sequence below to exercise the dossier path and the full poker crate.

## Verification

The commands below prove both the new dossier path and the absence of
regressions in the existing poker crate.

From the repository root:

    cargo test -p myosu-games-poker --quiet dossier
    cargo test -p myosu-games-poker --quiet

Both should pass. The first command exercises the new dossier tests specifically. The second ensures no regressions.

To manually produce a dossier from repo-owned artifacts:

    cargo run -p myosu-games-poker --example benchmark_scenario_pack -- --dossier-output /tmp/nlhe-dossier.json

(The `--dossier-output` flag is new and triggers dossier mode.)

## Acceptance Criteria

- artifact dossiers can be loaded and hash-verified
- benchmark dossiers can be serialized for downstream promotion use
- sparse checked-in artifacts remain supported for zero-iteration proof flows
- sparse checked-in artifacts are clearly marked as not promotion-ready

## Validation and Acceptance

1. `cargo test -p myosu-games-poker --quiet dossier` passes with at least 3 tests.
2. The dossier JSON file contains `manifest_hash`, `scenario_count`, `mean_quality_score`, and `promotion_ready` fields.
3. Existing poker tests still pass.
4. The miner still rejects positive-iteration training on sparse artifacts (no regression).

## Idempotence and Recovery

All changes are additive to the artifacts and benchmark modules. Existing behavior is preserved. The dossier writer can be rerun and will overwrite the output file.

## Artifacts and Notes

Expected modifications:
    crates/myosu-games-poker/src/artifacts.rs (add NlheArtifactDossier)
    crates/myosu-games-poker/src/benchmark.rs (add NlheBenchmarkDossier, writer)
    crates/myosu-games-poker/examples/benchmark_scenario_pack.rs (add --dossier-output flag)

## Interfaces and Dependencies

In `crates/myosu-games-poker/src/artifacts.rs`:

    pub struct NlheArtifactDossier {
        pub artifact_dir: PathBuf,
        pub manifest_hash: String,
        pub preflop_entries: usize,
        pub flop_entries: usize,
        pub turn_entries: usize,
        pub river_entries: usize,
        pub complete_streets: Vec<String>,
        pub sampled_streets: Vec<String>,
        pub postflop_complete: bool,
        pub verified: bool,
    }

    pub fn load_and_verify_artifact_dossier(
        dir: &Path,
    ) -> Result<NlheArtifactDossier, ArtifactCodecError>;

In `crates/myosu-games-poker/src/benchmark.rs`:

    pub struct NlheBenchmarkDossier {
        pub artifact: NlheArtifactDossier,
        pub scenario_count: usize,
        pub pass_count: usize,
        pub mean_quality_score: f64,
        pub quality_summary: String,
        pub promotion_ready: bool,
    }

    pub fn write_benchmark_dossier(
        dossier: &NlheBenchmarkDossier,
        output: &Path,
    ) -> Result<(), std::io::Error>;

Dependencies: `sha2` (already in workspace), `serde_json` (already in workspace).

Revision note (2026-04-11 / Codex review): added explicit verification and
acceptance headings so the plan matches both live repo tooling and the newer
ExecPlan structure.
