# Miner Convergence Gate — Per-Game Iteration Guidance

This document is the operator-facing source of truth for the F-003 miner
convergence gate: how many training iterations produce a strategy worth
serving, per game type, on the current pinned stage-0 solver constants. The
numbers are the truthful benchmark surface (exact best-response exploitability
on a fresh-solver ladder), not the same-checkpoint validator self-score, and
the per-game entries link to the executable proof that backs the number.

> Acceptance target: a `tier: benchmarked` operator run should be able to
> defend the chosen `--train-iterations N` against the per-game numbers in
> the table below and the executable proof in
> `tests/e2e/quality_benchmark_liars_dice.sh` / the doc-reg drift guard
> `tests/e2e/miner_convergence_doc_keeps_truthful_thresholds.sh`. A
> recommendation that diverges from the live benchmark ladder is a finding,
> not a silent override.

## Why this exists

The repo has a stage-0 self-check validator path that scores a miner's
response against the same checkpoint the miner was trained on. That path is
useful as a determinism proof, but it is **not** a convergence metric: a
miner that trains for 1 iteration will get `exact_match = true` against
itself. Operators therefore have no truthful signal from the validator on
how many iterations to train before serving a strategy, which is exactly
the F-003 blocker.

The truthful per-game signal is exact best-response exploitability measured
from a fresh solver start, scaled against a fixed threshold. The table
below records the per-game threshold and the minimum iteration count that
meets it, on the current pinned solver constants.

## Per-Game Convergence Table

The table is a one-page summary. The "Source of truth" column is the
executable Rust unit test, integration test, or example binary that
re-derives the number. A solver-constant change that moves the
recommended iteration count must be caught by the source-of-truth entry,
not by hand.

| Game           | Iterations | Exploitability (≤) | Surface                                | Status             | Source of truth                                                                                  |
| -------------- | ---------- | ------------------ | -------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------ |
| `kuhn_poker`   | 0          | 0.000000 (closed form) | Closed-form Nash (no MCCFR needed) | Resolved           | `crates/myosu-games-kuhn/src/solver.rs` — `KuhnSolver::train` is a no-op constant; the strategy is exact. |
| `liars-dice`   | 512        | 0.700000           | Exact best-response exploitability     | Resolved           | `tests/e2e/quality_benchmark_liars_dice.sh` (asserts `recommended_minimum_iterations=512` at `exploitability<=0.70`); unit test `quality_benchmark_liars_dice_exploitability_converges`; example `cargo run -p myosu-validator --example quality_benchmark`. |
| `liars-dice` (better) | 1024   | ~0.560             | Exact best-response exploitability     | Resolved (stretch) | Same as above; the example binary prints both 512 and 1024 by default.                            |
| `nlhe-heads-up` | —          | —                  | Exact best-response exploitability     | **Blocked**        | See [Poker Convergence Status](#poker-convergence-status) below. The checked-in bootstrap encoder artifacts are intentionally sparse (`postflop_complete=false`), so positive-iteration poker training fails upstream with `isomorphism not found` and a real exploitability ladder cannot be recorded against the stage-0 bootstrap fixtures. A truthful ladder requires the robopoker/PostgreSQL `isomorphism` materialization path in [`poker-quality-benchmark.md`](../execution-playbooks/poker-quality-benchmark.md) and `bash ops/poker_quality_benchmark.sh`. |
| `kuhn_poker` (validator determinism) | 0   | 0.000000 (closed form) | Same-checkpoint self-check        | Not a convergence metric (determinism proof only) | `tests/e2e/validator_determinism.sh` exercises the self-check path. Listed here only to keep it on the same page so operators do not mistake the determinism proof for a convergence signal. |

The thresholds and iteration counts are the truthful numbers. A future
operator who wants a stricter or looser convergence target (for example,
`exploitability <= 0.50` for a research-grade local run) can re-run the
example binary with a wider iteration ladder; the source-of-truth entries
recompute the recommendation from the ladder on every call, so the
doc-reg drift guard does not need to be re-tuned by hand.

## How the Liar's Dice number was derived

The 512-iteration recommendation is the smallest entry on the default
iteration ladder `0, 128, 256, 512, 1024` whose exact best-response
exploitability meets the `LIARS_DICE_USEFUL_EXPLOITABILITY_THRESHOLD = 0.70`
constant in `crates/myosu-validator/src/validation.rs`. The ladder
measures exploitability from a fresh `LiarsDiceSolver::<1024>::new()`
start at each requested iteration count, so the recommendation is
independent of any cached checkpoint state. The 1024-iteration stretch
target is not a separate "minimum" — it is a documented observation that
exploitability continues to fall past 512, so operators with a larger
runtime budget can serve a measurably better strategy by going to 1024.

Reproduce locally with one of:

```bash
# Operator-facing reproduction of the recommended threshold.
SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example quality_benchmark -- 0 128 256 512

# Wider ladder for the 1024-iteration stretch observation.
SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example quality_benchmark -- 0 128 256 512 1024

# Unit-test gate (shares a single source of truth with the example).
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- quality_benchmark

# CI-grade E2E proof harness.
bash tests/e2e/quality_benchmark_liars_dice.sh
```

A solver-constant change that shifts the 512-iteration exploitability
above `0.70` is a regression of the operator-facing guarantee. The
E2E harness `tests/e2e/quality_benchmark_liars_dice.sh` is the gate that
catches it, and the F-003 doc-reg drift guard
`tests/e2e/miner_convergence_doc_keeps_truthful_thresholds.sh` is the
secondary guard that prevents the operator-guide text from drifting away
from the live benchmark.

## Poker Convergence Status

Poker is intentionally **not** on the resolved list. The truthful poker
exploitability ladder requires a full encoder, and the checked-in
`bootstrap_artifacts.rs` emits a deliberately sparse
`NlheArtifactDossier` with `postflop_complete=false`. Positive-iteration
poker training against the checked-in artifacts therefore fails upstream
with `isomorphism not found` (the same path that keeps the stage-0 miner
self-check honest), and a real exploitability number cannot be recorded
against the bootstrap fixtures without lying about the dossier's
`passing=true` claim.

The repo ships the truthful full-encoder path for operators who have the
required robopoker/PostgreSQL hardware footprint
([`poker-quality-benchmark.md`](../execution-playbooks/poker-quality-benchmark.md)
and `bash ops/poker_quality_benchmark.sh`), but it is explicitly an
out-of-band operator run, not a stage-0 miner default. The recommended
path for an operator who needs a real poker iteration recommendation
today is:

1. Materialize a full `isomorphism` lookup from robopoker's PostgreSQL
   export per the playbook.
2. Run `bash ops/poker_quality_benchmark.sh` to record an exploitability
   ladder.
3. Record the chosen iteration count + threshold against the recorded
   ladder, and treat the recorded number as that operator's per-deploy
   poker convergence floor.

This is the truthful shape the F-003 blocker note calls out: the
recommendation must be derived from a real benchmark surface, not from a
placeholder or a same-checkpoint self-score. Until a promotion-grade
NLHE artifact dossier is committed (the PROMOTE-001 unblock), the
"blocked" row in the table above is the operator-facing recommendation.

## Negative Test: A bad iteration count must fail the gate

The repo intentionally does **not** relax the per-game thresholds to
absorb a poorly-trained miner. The `quality_benchmark` example and the
`tests/e2e/quality_benchmark_liars_dice.sh` harness both fail closed
when the live exploitability exceeds the threshold, and the F-003
doc-reg drift guard fails closed if the operator-facing text drifts
away from the live `recommended_minimum_iterations` value. A change
that tries to lower the recommended iteration count by re-anchoring
the doc to a placeholder benchmark is a regression, not a
"simplification."

## Where this lives in the operator flow

- [`docs/operator-guide/quickstart.md`](quickstart.md) — the
  per-step quickstart links to this doc in the "Miner Quality"
  section so a new operator sees the per-game table alongside the
  poker/Liar's Dice checkpointing walkthrough.
- [`docs/execution-playbooks/poker-quality-benchmark.md`](../execution-playbooks/poker-quality-benchmark.md)
  — the operator-facing playbook for recording a real poker
  exploitability ladder against the robopoker/PostgreSQL path.
- `tests/e2e/quality_benchmark_liars_dice.sh` — the CI-grade
  executable proof that the Liar's Dice row above stays true.
- `tests/e2e/miner_convergence_doc_keeps_truthful_thresholds.sh` —
  the F-003 doc-reg drift guard that prevents this doc from drifting
  away from the live benchmark, the live `LIARS_DICE_SOLVER_TREES`
  constant, and the live `recommended_minimum_iterations` value.
