# Specification: Python Research Framework

Source: Reverse-engineered from root-level Python files (main.py, runner.py, data.py, methods.py, metrics.py)
Status: Draft
Depends-on: none

## Purpose

The Python research framework evaluates game-solving algorithm conditions against
a corpus of imperfect-information games. It provides an experiment harness that
loads survey data from 20 games, tests multiple algorithm conditions with proxy
benchmarks, computes recommendation fidelity metrics across 6 dimensions, and
produces leaderboard rankings with bootstrap confidence intervals. This
framework supports research into which algorithmic approaches best serve the
Myosu game-solving objective.

The primary consumer is a researcher evaluating algorithm variants for
game-solving quality.

## Whole-System Goal

Current state: The framework exists as 5 root-level Python files totaling
approximately 3,200 lines. It is not integrated into the Rust codebase or CI
pipeline. It has no tests, linting, or type checking.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: A researcher can run the experiment harness to evaluate
algorithm conditions, view ranked results with statistical confidence, and
export results to CSV.

Still not solved here: Integration with the Rust game crates, CI quality gates
for Python code, and automated experiment scheduling are separate concerns.

## Scope

In scope:
- Experiment harness loading 20-game survey corpus
- Multiple algorithm condition testing with CIFAR-100 proxy benchmark
- 6-dimension recommendation fidelity metric computation
- Multi-seed evaluation with bootstrap confidence intervals
- Paired statistical analysis between conditions
- Leaderboard ranking and CSV export
- Established baselines (random search, DARTS, AdamW) and proposed variants
- Ablation conditions

Out of scope:
- Integration with Rust game crates or the Myosu chain
- CI linting, type checking, or automated testing of Python code
- Deployment or scheduling of experiments
- Real-time metrics or dashboards
- Interaction with the miner, validator, or gameplay surface

## Current State

The framework consists of 5 files in the repository root:

main.py (approximately 17,700 lines) serves as the experiment harness entry
point. It loads survey data from a corpus of 20 imperfect-information games,
configures algorithm conditions, runs multi-seed evaluations, and produces
leaderboard output with CSV export.

runner.py (approximately 12,200 lines) manages per-seed and per-regime result
aggregation, computes primary and secondary metrics, calculates determinism
scores, handles condition ranking, and exports results to CSV.

data.py (approximately 42,600 lines) builds feature matrices from the game
corpus, loads experiment plans, and decodes game recommendations.

methods.py (approximately 42,900 lines) implements algorithm conditions:
established baselines (random search, DARTS, AdamW, and others) plus proposed
approach variants and ablation conditions.

metrics.py (approximately 9,500 lines) provides bootstrap confidence interval
computation, paired statistical analysis, and recommendation fidelity
calculations across 6 evaluation dimensions.

The framework has no tests, no linting configuration, and no type annotations.
Known issues include an exponential-complexity metric calculation in metrics.py
and an __import__() anti-pattern in methods.py.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Experiment harness | main.py with 20-game corpus | Reuse | Working evaluation pipeline |
| Result aggregation | runner.py with per-seed/per-regime logic | Reuse | Statistical aggregation |
| Data loading | data.py with feature matrix construction | Reuse | Corpus access |
| Algorithm conditions | methods.py with baselines and variants | Reuse | Established comparison set |
| Statistical analysis | metrics.py with bootstrap CI | Reuse | Confidence-aware evaluation |

## Non-goals

- Replacing the Rust MCCFR solver with Python implementations.
- Integrating Python evaluation into the Rust build pipeline.
- Providing real-time experiment monitoring or visualization.
- Deploying experiments to cloud compute infrastructure.
- Establishing CI quality gates for the Python code (planned separately).

## Behaviors

The experiment harness loads a survey corpus of 20 imperfect-information games
and configures a set of algorithm conditions to evaluate. Each condition
represents a distinct approach to game-solving (baselines, proposed variants,
or ablation conditions).

For each condition, the harness runs multi-seed evaluations using a CIFAR-100
proxy benchmark. Each seed produces a set of results that the runner aggregates
into per-regime metrics.

The runner computes primary and secondary metrics for each condition, calculates
determinism scores (consistency across seeds), and ranks conditions by their
aggregate performance.

Recommendation fidelity is measured across 6 dimensions. The metrics module
computes bootstrap confidence intervals for each dimension, providing
statistical bounds on the evaluation.

Paired statistical analysis compares conditions head-to-head, identifying
statistically significant differences in performance.

The final output is a leaderboard ranking all conditions by aggregate
performance, with confidence intervals and statistical significance indicators.
Results are also exported to CSV for further analysis.

Known behavioral issues: metrics.py contains an exponential-complexity
calculation path that may be prohibitively slow for large evaluation sets.
methods.py uses __import__() for dynamic module loading, which is an
anti-pattern that complicates static analysis.

## Acceptance Criteria

- The experiment harness loads the 20-game survey corpus without error.
- Algorithm conditions produce evaluation results across multiple seeds.
- Bootstrap confidence intervals are computed for all 6 fidelity dimensions.
- Paired statistical analysis identifies significant differences between
  conditions.
- The leaderboard ranks conditions by aggregate performance with confidence
  intervals.
- Results export to CSV with all metrics and rankings.
- The framework runs independently of the Rust codebase.
