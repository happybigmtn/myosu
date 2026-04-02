# Specification: Python Research Quality Gates

Source: Genesis Plan 012 (Python Research Stack QA), ASSESSMENT.md code-review findings
Status: Draft
Depends-on: none

## Purpose

The Python research stack (5 root-level files, ~3.2K lines) implements an ML
experiment framework for evaluating game-solving approaches against a 20-game
corpus. It shares the repository but has no tests, no linting, no type checking,
and several identified code quality issues including an `__import__()` anti-
pattern and an exponential-complexity permutation test. As a research tool used
to evaluate solver approaches that feed into the Rust product, its correctness
matters — but it does not warrant the same investment as production code.
Baseline quality gates prevent silent regressions and catch the known issues
without over-investing in a research support tool.

## Whole-System Goal

Current state: The Python files (`main.py`, `data.py`, `methods.py`,
`metrics.py`, `runner.py`) have no CI gate. `methods.py:807` uses
`__import__("data")` as a circular import workaround. `metrics.py:74-84`
generates 2^n permutations which will hang for n>20. Broad exception catching in
`main.py` and `runner.py` hides failures. `run_eval.py` assumes stdout JSON but
`main.py` writes to file. The framework uses seeded randomness throughout,
which is valuable for reproducibility.

This spec adds: Automated linting, basic test coverage for critical paths, and
fixes for the known anti-patterns. A CI job that gates the Python stack on
these quality checks.

If all ACs land: The Python research stack passes linting, has basic test
coverage for metrics and data loading, the known anti-patterns are fixed, and
CI prevents regressions.

Still not solved here: Comprehensive test coverage, type checking, performance
optimization beyond the exponential fix, documentation, and integration with the
Rust product pipeline.

## Scope

In scope:
- Fixing the `__import__()` anti-pattern in `methods.py`
- Fixing the exponential complexity in `metrics.py` permutation test
- Configuring and running a linter on the Python files
- Writing basic tests for metrics computation and data loading
- Adding a Python CI job

Out of scope:
- Comprehensive test coverage for all Python modules
- Type annotations or type checking
- Refactoring the experiment framework architecture
- Performance optimization beyond the exponential fix
- Integration testing between Python research and Rust product
- Fixing the `run_eval.py` stdout/file mismatch (may indicate dead code)

## Current State

The five Python files at the repository root form a self-contained experiment
framework:

- `main.py` (489 lines): Entry point, experiment orchestration, broad exception
  catching
- `data.py` (1,060 lines): Dataset loading, hardcoded paths, silent fallbacks
- `methods.py` (1,105 lines): Game-solving method implementations,
  `__import__("data")` at line 807
- `metrics.py` (246 lines): Evaluation metrics, 2^n permutation test at lines
  74-84
- `runner.py` (337 lines): Experiment runner, no checkpoint/recovery

The framework uses seeded randomness throughout, making results reproducible.
No `pyproject.toml`, `requirements.txt`, or test directory exists for the
research stack.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Experiment framework | 5 root-level Python files | Extend | Add quality gates around existing code |
| Seeded randomness | Throughout all Python files | Reuse | Valuable for reproducibility; preserve |
| CI pipeline | `.github/workflows/ci.yml` | Extend | Add Python job |
| Research game corpus | 20-game evaluation corpus | Reuse | Test data for metrics tests |
| Nothing for Python QA | No pyproject.toml, tests, or lint config | Create | Must be created from scratch |

## Non-goals

- Rewriting the experiment framework in a different language or architecture.
- Adding type annotations to all Python files.
- Achieving high test coverage (80%+) for the research stack.
- Publishing the research stack as a Python package.
- Integrating the research stack output directly into the Rust build pipeline.
- Fixing all code quality issues — only the identified anti-patterns and the
  exponential complexity.

## Behaviors

The `__import__()` anti-pattern in `methods.py` is replaced with a standard
import that does not rely on runtime dynamic import resolution. The circular
dependency it worked around is resolved through module restructuring or
deferred import.

The permutation test in `metrics.py` uses an approximate method (random
sampling) for inputs larger than a threshold (n>15), avoiding the 2^n
combinatorial explosion while preserving statistical validity for the metric.

A linter configured via `pyproject.toml` checks all five root-level Python
files. The linter configuration is reasonable for research code — strict enough
to catch real issues but not so aggressive that it flags idiomatic research
patterns.

Basic tests cover the metrics module (rank correlation, top-k overlap, the
permutation test with both small and large n) and data loading (missing inputs,
empty inputs, valid loading). Tests use the seeded randomness pattern already
present in the codebase.

A CI job runs the linter and test suite on the Python files. The job fails if
linting or tests fail.

## Acceptance Criteria

- The `__import__()` anti-pattern in `methods.py` is replaced with a standard
  import.
- The permutation test in `metrics.py` completes in bounded time for n>20.
- A linter passes on all five root-level Python files.
- Tests exist and pass for metrics computation (including the permutation test
  edge case) and data loading.
- A Python CI job runs linting and tests and gates the pipeline on their
  results.
