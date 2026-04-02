# 012 - Python Research Stack QA

## Purpose / Big Picture

The Python research stack (`main.py`, `data.py`, `methods.py`, `metrics.py`,
`runner.py`) is an ML experiment framework sharing the repo. It has no tests, no
CI, and several code quality issues identified in the assessment. This plan adds
basic quality gates without over-investing in a research codebase.

## Context and Orientation

Issues identified in ASSESSMENT.md:
- `methods.py:807`: `__import__("data")` anti-pattern (circular import workaround)
- `metrics.py:74-84`: 2^n permutation test that will hang for n>20
- Broad exception catching in `main.py`, `runner.py`
- `run_eval.py:70`: Assumes stdout JSON but `main.py` writes to file
- No tests, no linting, no type checking
- Hardcoded dataset paths, magic numbers

## Architecture

```
Python QA additions:
├── pyproject.toml          # ruff config, test config
├── tests/
│   ├── test_metrics.py     # Metrics edge cases
│   └── test_data.py        # Data loading validation
└── .github/workflows/ci.yml  # Add Python lint job
```

## Progress

### Milestone 1: Fix critical code quality issues

- [ ] M1. Fix `__import__()` anti-pattern and exponential complexity
  - Surfaces: `methods.py`, `metrics.py`
  - What exists after: `methods.py:807` uses direct import. `metrics.py:74-84`
    uses approximate permutation test (random sample) for n>15.
  - Why now: These are correctness issues, not style.
Proof command: `rg '__import__' methods.py` returns 0 matches
  - Tests: `python -c "from methods import *; print('ok')"`

### Milestone 2: Add ruff linting

- [ ] M2. Configure and run ruff on Python files
  - Surfaces: `pyproject.toml` (new or updated), `main.py`, `data.py`, `methods.py`, `metrics.py`, `runner.py`
  - What exists after: `ruff check` passes on all 5 root Python files.
    Configuration in `pyproject.toml` with reasonable rule set.
  - Why now: Linting catches obvious bugs.
Proof command: `ruff check main.py data.py methods.py metrics.py runner.py`
  - Tests: `ruff check` exits 0

### Milestone 3: Add basic test suite

- [ ] M3. Write tests for metrics and data loading
  - Surfaces: `tests/test_metrics.py` (new), `tests/test_data.py` (new)
  - What exists after: Tests cover: rank correlation, top-k overlap, calibration
    error, bootstrap CI, sign flip test with small and large n, data loading
    with missing/empty inputs.
  - Why now: Statistical code without tests is untrustworthy.
Proof command: `python -m pytest tests/ -q`
  - Tests: `pytest` passes

### Milestone 4: Add Python CI job

- [ ] M4. Add Python linting and testing to CI pipeline
  - Surfaces: `.github/workflows/ci.yml`
  - What exists after: CI job that runs `ruff check` and `pytest` on Python files.
  - Why now: Quality gates only work if enforced automatically.
Proof command: `gh run list --branch trunk --limit 1`
  - Tests: Python CI job green

## Surprises & Discoveries

- The Python files are a research experiment framework evaluating 10 ML
  conditions against a 20-game corpus. The code is deterministic by design
  (seeded randomness throughout). This is unusual and valuable for reproducibility.
- `run_eval.py` has a bug where it expects JSON on stdout but `main.py` writes
  to a file. This suggests `run_eval.py` was never actually used successfully.

## Decision Log

- Decision: Fix critical issues only, not rewrite the research stack.
  - Why: This is research code, not product code. Diminishing returns on quality
    investment. Fix correctness issues, add linting, add basic tests.
  - Failure mode: Research code is abandoned and rots.
  - Mitigation: Basic CI prevents silent breakage.
  - Reversible: yes

- Decision: ruff (not pylint/flake8) for linting.
  - Why: ruff is faster, handles more rules, and is the standard per repo CLAUDE.md.
  - Failure mode: None -- ruff is strictly better.
  - Mitigation: N/A.
  - Reversible: yes

## Validation and Acceptance

1. `__import__()` anti-pattern removed.
2. Exponential complexity fixed.
3. `ruff check` passes.
4. `pytest` passes with basic test suite.
5. Python CI job exists and passes.

## Outcomes & Retrospective
_Updated after milestones complete._
