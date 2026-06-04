//! Content-level verification of the promotion outputs tree for one slug.
//!
//! The promotion ledger in `ops/solver_promotion.yaml` declares a `tier` for
//! every research game; the `promotion_manifest.sh` shell harness only checks
//! that the three on-disk outputs exist and are non-empty. This binary encodes
//! the stricter, content-level gate that `genesis/plans/005-nlhe-promotion.md`
//! unit 2 calls for: a tier of `promotable_local` (or stricter) requires the
//! emitted `bundle.json` to actually verify and the `benchmark-summary.json`
//! to record a passing dossier with finite values. A sparse or placeholder
//! outputs tree — the failure mode the previous worker called out for
//! `nlhe-heads-up` — fails closed with a typed `PromotionGateError` and a
//! non-zero exit code, so the bash harness can `grep` the failure reason and
//! exit 1 before the promotion would otherwise be silently accepted.
//!
//! Usage:
//!   cargo run -p myosu-games-canonical --example verify_promotion_outputs --
//!     --slug <slug> [--outputs-dir <path>]
//!   # default outputs dir is `<repo>/outputs/solver-promotion`
//!
//! Exit codes:
//!   0 — outputs tree passes the gate
//!   1 — bundle missing / fails verification / has a non-passing benchmark
//!   2 — benchmark-summary.json missing / malformed / non-passing
//!   3 — slug not declared in the promotion ledger, or declared below the
//!       required tier for the gate to apply

use std::{env, fs, path::PathBuf};

use myosu_games_canonical::{
    CanonicalPolicyBundle, PolicyPromotionTier, ResearchGame, code_reported_bundle_support,
    parse_solver_promotion_ledger, verify_policy_bundle,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BenchmarkSummary {
    benchmark_id: String,
    metric_name: String,
    metric_value: f64,
    threshold: f64,
    passing: bool,
}

#[derive(Debug)]
enum PromotionGateError {
    Missing(String),
    BundleNotVerifiable(String),
    BundleBenchmarkNotPassing(String),
    SummaryMalformed(String),
    SummaryNotPassing(String),
}

impl std::fmt::Display for PromotionGateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromotionGateError::Missing(path) => write!(formatter, "missing output: {path}"),
            PromotionGateError::BundleNotVerifiable(reason) => {
                write!(formatter, "bundle failed verify_policy_bundle: {reason}")
            }
            PromotionGateError::BundleBenchmarkNotPassing(reason) => {
                write!(formatter, "bundle benchmark not passing: {reason}")
            }
            PromotionGateError::SummaryMalformed(reason) => {
                write!(formatter, "benchmark-summary.json malformed: {reason}")
            }
            PromotionGateError::SummaryNotPassing(reason) => {
                write!(formatter, "benchmark-summary.json not passing: {reason}")
            }
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("PROMOTION_GATE_FAIL reason={error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), PromotionGateError> {
    let args = parse_args();
    let repo_root = repo_root();
    let outputs_dir = args
        .outputs_dir
        .unwrap_or_else(|| repo_root.join("outputs/solver-promotion"));
    let slug = args.slug;

    let ledger_path = env::var("MYOSU_SOLVER_PROMOTION_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root.join("ops/solver_promotion.yaml"));
    let ledger_text = fs::read_to_string(&ledger_path).map_err(|source| {
        PromotionGateError::Missing(format!("ledger `{}`: {source}", ledger_path.display()))
    })?;
    let ledger = parse_solver_promotion_ledger(&ledger_text)
        .map_err(|source| PromotionGateError::Missing(format!("ledger parse failed: {source}")))?;

    let entry = ledger
        .games
        .iter()
        .find(|entry| entry.game == slug)
        .ok_or_else(|| {
            PromotionGateError::Missing(format!(
                "ledger has no row for slug `{slug}`; declared rows: {}",
                ledger
                    .games
                    .iter()
                    .map(|entry| entry.game.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

    if entry.tier < PolicyPromotionTier::PromotableLocal {
        let game = ResearchGame::from_slug(&slug).unwrap_or(ResearchGame::NlheSixMax);
        println!(
            "PROMOTION_GATE_SKIP slug={slug} tier={} reason=below_promotable_local code_bundle_support={}",
            entry.tier.as_str(),
            code_reported_bundle_support(game).as_str()
        );
        return Ok(());
    }

    let bundle_path = outputs_dir.join(&slug).join("bundle.json");
    let summary_path = outputs_dir.join(&slug).join("benchmark-summary.json");
    let manifest_path = outputs_dir.join(&slug).join("artifact-manifest.json");

    for required in [&bundle_path, &summary_path, &manifest_path] {
        if !required.is_file() {
            return Err(PromotionGateError::Missing(format!(
                "{}",
                required.display()
            )));
        }
    }

    let bundle_text = fs::read_to_string(&bundle_path).map_err(|source| {
        PromotionGateError::Missing(format!("read {}: {source}", bundle_path.display()))
    })?;
    let bundle: CanonicalPolicyBundle = serde_json::from_str(&bundle_text).map_err(|source| {
        PromotionGateError::BundleNotVerifiable(format!(
            "bundle.json does not parse as CanonicalPolicyBundle: {source}"
        ))
    })?;

    verify_policy_bundle(&bundle).map_err(|source| {
        PromotionGateError::BundleNotVerifiable(format!(
            "verify_policy_bundle rejected bundle.json: {source}"
        ))
    })?;

    if !bundle.provenance.benchmark.passing {
        return Err(PromotionGateError::BundleBenchmarkNotPassing(format!(
            "bundle.provenance.benchmark.passing=false (metric_name={}, metric_value={}, threshold={})",
            bundle.provenance.benchmark.metric_name,
            bundle.provenance.benchmark.metric_value,
            bundle.provenance.benchmark.threshold
        )));
    }
    if !bundle.provenance.benchmark.metric_value.is_finite()
        || !bundle.provenance.benchmark.threshold.is_finite()
    {
        return Err(PromotionGateError::BundleBenchmarkNotPassing(format!(
            "bundle benchmark values are non-finite (metric_value={}, threshold={})",
            bundle.provenance.benchmark.metric_value, bundle.provenance.benchmark.threshold
        )));
    }

    let summary_text = fs::read_to_string(&summary_path).map_err(|source| {
        PromotionGateError::SummaryMalformed(format!("read {}: {source}", summary_path.display()))
    })?;
    let summary: BenchmarkSummary = serde_json::from_str(&summary_text).map_err(|source| {
        PromotionGateError::SummaryMalformed(format!(
            "benchmark-summary.json does not parse: {source}"
        ))
    })?;

    if !summary.passing || !summary.metric_value.is_finite() || !summary.threshold.is_finite() {
        return Err(PromotionGateError::SummaryNotPassing(format!(
            "benchmark-summary.json not passing (passing={}, metric_name={}, metric_value={}, threshold={})",
            summary.passing, summary.metric_name, summary.metric_value, summary.threshold
        )));
    }

    if summary.benchmark_id != bundle.provenance.benchmark.benchmark_id {
        return Err(PromotionGateError::SummaryNotPassing(format!(
            "benchmark_summary.benchmark_id mismatch: bundle={}, summary={}",
            bundle.provenance.benchmark.benchmark_id, summary.benchmark_id
        )));
    }

    println!(
        "PROMOTION_GATE_PASS slug={slug} tier={} benchmark_id={} metric_name={} metric_value={} threshold={} bundle_hash={}",
        entry.tier.as_str(),
        summary.benchmark_id,
        summary.metric_name,
        summary.metric_value,
        summary.threshold,
        bundle.bundle_hash
    );
    Ok(())
}

struct CliArgs {
    slug: String,
    outputs_dir: Option<PathBuf>,
}

fn parse_args() -> CliArgs {
    let mut slug: Option<String> = None;
    let mut outputs_dir: Option<PathBuf> = None;
    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--slug" => {
                slug = iter.next();
            }
            "--outputs-dir" => {
                outputs_dir = iter.next().map(PathBuf::from);
            }
            "--help" | "-h" => {
                eprintln!("usage: verify_promotion_outputs --slug <slug> [--outputs-dir <path>]");
                std::process::exit(0);
            }
            other => {
                if slug.is_none() {
                    slug = Some(other.to_string());
                } else {
                    eprintln!("unexpected extra argument: {other}");
                    std::process::exit(2);
                }
            }
        }
    }
    let slug = slug.unwrap_or_else(|| {
        eprintln!("missing --slug <slug>");
        std::process::exit(2);
    });
    CliArgs { slug, outputs_dir }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
