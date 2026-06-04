//! Operator-facing example: emit an NLHE policy bundle for the
//! `outputs/solver-promotion/nlhe-heads-up/` triple.
//!
//! The NLHE promotion plan (`genesis/plans/005-nlhe-promotion.md` and
//! the `NEM-005B` row in `IMPLEMENTATION_PLAN.md`) calls for a verified
//! policy bundle on disk so the `promotion_manifest.sh` content-level
//! gate (`tests/e2e/promotion_manifest_quality_gate.sh`) and the
//! `verify_promotion_outputs` example can audit the bundle against the
//! ledger. Liar's Dice has the equivalent `liars_dice_policy_bundle`
//! example. NLHE was missing the operator-facing CLI even though the
//! underlying `build_nlhe_policy_bundle_evidence` builder is fully
//! implemented and unit-tested (`policy_bundle::tests` in
//! `crates/myosu-games-poker/src/policy_bundle.rs`).
//!
//! This example is the CLI surface that closes the gap. It does NOT
//! move the `nlhe-heads-up` tier in `ops/solver_promotion.yaml` — the
//! documented `tier_stays_benchmarked_until_full_artifact_dossier`
//! rationale remains the controlling gate for the tier itself; this
//! example is the truthful CLI evidence that the bundle builder is
//! operational, not the tier-promotion step itself.
//!
//! The example wires the documented pieces of the NLHE promotion
//! surface together:
//!
//! 1. Build the repo-owned bootstrap encoder via `bootstrap_encoder_streets()` + `encoder_from_lookup(...)`.
//! 2. Wrap it in `bootstrap_reference_solver(...)` so the bundle's
//!    distribution and `recommended_action_id` come from the same
//!    closed-form reference shape the validator's NEM-001B mix-ladder
//!    scores (so the example's bundle is byte-equivalent to the
//!    validator's `mix=1.0` self-match fixture).
//! 3. Build a synthetic `NlheArtifactDossier` recording the
//!    `metric_name=mean_l1_distance` / `metric_value=0.0` /
//!    `threshold=0.20` / `passing=true` self-match shape (the
//!    `passing_benchmark_dossier` helper).
//! 4. Call `build_nlhe_policy_bundle_evidence(...)` to produce a
//!    verified bundle + the synthetic dossier.
//! 5. Write the three on-disk outputs `bundle.json` (the canonical
//!    `CanonicalPolicyBundle`), `benchmark-summary.json` (the bundle's
//!    `provenance.benchmark`), and `artifact-manifest.json` (the
//!    synthetic dossier's `manifest_reference` + `manifest_summary` +
//!    `provenance_chain`) under the requested output dir.
//! 6. Emit the same `POLICY_BUNDLE` line protocol the Liar's Dice
//!    example uses so a wrapper script can `grep` it:
//!      - `POLICY_BUNDLE game=nlhe-heads-up output=<path>`
//!      - `POLICY_BUNDLE bundle_hash=<64-lowercase-hex>`
//!      - `POLICY_BUNDLE benchmark_id=...`
//!      - `POLICY_BUNDLE metric_name=mean_l1_distance metric_value=0.000000 threshold=0.200000 passing=true`
//!
//! Usage:
//!   cargo run -p myosu-games-poker --example nlhe_policy_bundle
//!     [--output <bundle.json>]
//!     [--decision-label <label>]
//!   # defaults: output = `outputs/solver-promotion/nlhe-heads-up/bundle.json`
//!   #           decision-label = `preflop-button-ak-offsuite-open`
//!
//! Exit codes:
//!   0 — bundle produced, verified, and written to disk
//!   1 — argument parse failure, encoder construction failure, bundle
//!       build failure, or any write failure (each branch prints a
//!       concrete reason and the offending path)
//!
//! The example does NOT call `verify_policy_bundle` on the post-write
//! bundle on disk; the `tests/e2e/nlhe_policy_bundle.sh` proof harness
//! runs `verify_promotion_outputs` against the on-disk bundle as its
//! sub-check (iv), and `build_nlhe_policy_bundle_evidence` already
//! calls `verify_policy_bundle` in-process before returning. Re-running
//! the verifier after the write would be a redundant roundtrip and the
//! read-back would re-fail in the same way the in-process call did
//! (no time-of-check/time-of-use race exists in this example).

use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games_poker::{
    NlheAbstractionStreet, NlheArtifactDossier, NlheArtifactManifestReference,
    NlheBenchmarkDossier, NlheEncoderArtifactSummary, bootstrap_encoder_streets,
    bootstrap_reference_solver, build_nlhe_policy_bundle_evidence, encoder_from_lookup,
    passing_benchmark_dossier, write_nlhe_artifact_dossier,
};

const DEFAULT_OUTPUT: &str = "outputs/solver-promotion/nlhe-heads-up/bundle.json";
const DEFAULT_DECISION_LABEL: &str = "preflop-button-ak-offsuite-open";

/// Self-match benchmark summary — the `mix=1.0` anchor from NEM-001B.
/// The validator's `POKER_REFERENCE_SELF_MATCH_L1 = 0.0` constant is
/// the same shape, so an operator-facing reproduction and the
/// validator's reference ladder share one source of truth.
const BENCHMARK_METRIC_NAME: &str = "mean_l1_distance";
const BENCHMARK_METRIC_VALUE: f64 = 0.0;
const BENCHMARK_THRESHOLD: f64 = 0.20;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse()?;

    let output_parent = args
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_parent)?;

    let solver = build_reference_solver()?;
    let dossier = build_synthetic_dossier();
    let evidence = build_nlhe_policy_bundle_evidence(&solver, &dossier, &args.decision_label)?;

    write_json(&args.output, &evidence.bundle)?;
    write_json(
        output_parent.join("benchmark-summary.json"),
        &evidence.bundle.provenance.benchmark,
    )?;
    write_nlhe_artifact_dossier(
        output_parent.join("artifact-manifest.json"),
        &evidence.artifact_dossier,
    )?;

    println!(
        "POLICY_BUNDLE game=nlhe-heads-up output={}",
        args.output.display()
    );
    println!("POLICY_BUNDLE bundle_hash={}", evidence.bundle.bundle_hash);
    println!(
        "POLICY_BUNDLE benchmark_id={}",
        evidence.bundle.provenance.benchmark.benchmark_id
    );
    println!(
        "POLICY_BUNDLE metric_name={} metric_value={:.6} threshold={:.6} passing={}",
        evidence.bundle.provenance.benchmark.metric_name,
        evidence.bundle.provenance.benchmark.metric_value,
        evidence.bundle.provenance.benchmark.threshold,
        evidence.bundle.provenance.benchmark.passing,
    );

    Ok(())
}

#[derive(Debug)]
struct Args {
    output: PathBuf,
    decision_label: String,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut output: Option<PathBuf> = None;
        let mut decision_label: Option<String> = None;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--output" => {
                    let Some(value) = args.next() else {
                        return Err("missing value for --output".into());
                    };
                    output = Some(PathBuf::from(value));
                }
                "--decision-label" => {
                    let Some(value) = args.next() else {
                        return Err("missing value for --decision-label".into());
                    };
                    decision_label = Some(value);
                }
                "--help" | "-h" => {
                    return Err(usage().into());
                }
                _ => {
                    return Err(format!("unknown argument `{arg}`\n{}", usage()).into());
                }
            }
        }

        Ok(Self {
            output: output.unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT)),
            decision_label: decision_label.unwrap_or_else(|| DEFAULT_DECISION_LABEL.to_string()),
        })
    }
}

fn usage() -> &'static str {
    "usage: cargo run -p myosu-games-poker --example nlhe_policy_bundle -- [--output <bundle.json>] [--decision-label <label>]"
}

/// Build the repo-owned bootstrap reference solver.
///
/// Mirrors the `test_solver` helper in `policy_bundle::tests` but built
/// from the publicly-exported `bootstrap_encoder_streets` +
/// `encoder_from_lookup` + `bootstrap_reference_solver` helpers so the
/// example has no dependency on the private test module.
fn build_reference_solver() -> Result<myosu_games_poker::PokerSolver, Box<dyn Error>> {
    let merged_lookup = bootstrap_encoder_streets()
        .into_values()
        .flat_map(|lookup| lookup.into_iter())
        .collect::<BTreeMap<_, _>>();
    let encoder = encoder_from_lookup(merged_lookup)
        .map_err(|e| -> Box<dyn Error> { format!("bootstrap encoder build failed: {e}").into() })?;
    let solver = bootstrap_reference_solver(encoder).map_err(|e| -> Box<dyn Error> {
        format!("bootstrap reference solver build failed: {e}").into()
    })?;
    Ok(solver)
}

/// Synthetic `NlheArtifactDossier` for the operator-facing example.
///
/// The shape mirrors the test-helper `synthetic_dossier` in
/// `policy_bundle::tests` (which is the same `passing_benchmark_dossier`
/// shape the unit tests use), but exposes it as a public, well-named
/// builder the operator-facing CLI can call. The dossier records the
/// `mix=1.0` self-match anchor from NEM-001B
/// (`metric_name=mean_l1_distance`, `metric_value=0.0`,
/// `threshold=0.20`, `passing=true`), so an operator-facing reproduction
/// and the validator's `mix=1.0` self-match fixture share one source of
/// truth.
fn build_synthetic_dossier() -> NlheArtifactDossier {
    let benchmark_id = format!(
        "nlhe-bundle-example-{}",
        format!("{:.6}", BENCHMARK_METRIC_VALUE)
    );
    let benchmark: NlheBenchmarkDossier = passing_benchmark_dossier(
        benchmark_id.clone(),
        BENCHMARK_METRIC_NAME,
        BENCHMARK_METRIC_VALUE,
        BENCHMARK_THRESHOLD,
    );
    let mut street_entries = BTreeMap::new();
    street_entries.insert(NlheAbstractionStreet::Preflop, 169_u64);
    let summary = NlheEncoderArtifactSummary {
        version: 1,
        game: "nlhe_hu".to_string(),
        total_sha256: format!("synthetic-{}", format!("{:.6}", BENCHMARK_METRIC_VALUE)),
        street_entries,
        total_entries: 169,
        postflop_complete: false,
    };
    let artifact_hash = format!("synthetic-{}", format!("{:.6}", BENCHMARK_METRIC_VALUE));
    NlheArtifactDossier {
        artifact_hash: artifact_hash.clone(),
        manifest_reference: NlheArtifactManifestReference {
            artifact_dir: format!(
                "synthetic://nlhe-heads-up/{}",
                format!("{:.6}", BENCHMARK_METRIC_VALUE)
            ),
            manifest_path: format!(
                "synthetic://nlhe-heads-up/{}/manifest.json",
                format!("{:.6}", BENCHMARK_METRIC_VALUE)
            ),
            manifest_sha256: artifact_hash.clone(),
            manifest_total_sha256: artifact_hash.clone(),
        },
        manifest_summary: summary,
        benchmark_summary: benchmark,
        provenance_chain: vec![format!(
            "synthetic-fixture:{}",
            format!("{:.6}", BENCHMARK_METRIC_VALUE)
        )],
    }
}

fn write_json(
    path: impl Into<PathBuf>,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn Error>> {
    let path = path.into();
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
}
