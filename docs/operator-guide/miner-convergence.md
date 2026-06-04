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
> `tests/e2e/quality_benchmark_liars_dice.sh` /
> `tests/e2e/poker_quality_benchmark.sh` / the doc-reg drift guard
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
| `nlhe-heads-up` | 0.95 match_ratio | —             | Mixed bootstrap reference (L1 distance vs `bootstrap_scenarios()` reference pack) | Resolved (NEM-001B substitute) | `tests/e2e/poker_quality_benchmark.sh` (asserts `reference_self_match_count=80`, `reference_self_match_l1=0.0`, `match_ratio_threshold=0.95`, `mix_ladder=0.0,0.25,0.5,0.75,1.0`); unit tests `poker_quality_benchmark_points_self_match_is_exact_at_mix_one` / `poker_quality_benchmark_points_default_ladder_is_monotonic`; example `cargo run -p myosu-validator --example poker_quality_benchmark`. The `mix_ladder` is the F-003 / NEM-001B substitute for a positive-iteration exploitability ladder (see [Poker Convergence Status](#poker-convergence-status) below). |
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

## How the Poker mix-ladder was derived

The poker recommendation is the lowest `mix` on the default
`POKER_REFERENCE_LADDER = [0.0, 0.25, 0.5, 0.75, 1.0]` whose
`exact_action_match_ratio` against the `bootstrap_scenarios()` reference
pack (80 scenarios — 8 preflop + 24 flop + 24 turn + 24 river) meets
`POKER_USEFUL_REFERENCE_MATCH_RATIO = 0.95` (a 76/80-or-better match).
The `mix=0.0` endpoint is a uniform-weight perturbation of the
closed-form reference profile; the `mix=1.0` endpoint is the
closed-form reference profile itself (a bit-exact self-match with
`mean_l1_distance=0.0` and `exact_action_matches=80`). Intermediate
`mix` values linearly interpolate between the two, so the ladder's
`exact_action_match_ratio` is monotonically non-decreasing in `mix`
(the regression the `poker_quality_benchmark_points_default_ladder_is_monotonic`
unit test pins).

The current checked-in bootstrap encoder is so concentrated on its
top action that even a `mix=0.25` candidate (25% closed-form reference +
75% uniform) still hits `80/80` top-action matches against the reference
pack — the recommendation is `mix=0.25` on the current pinned solver
constants. Operators with a stricter "useful" target can re-run the
example binary with a wider or finer ladder; the source-of-truth entries
recompute the recommendation from the ladder on every call, so the
doc-reg drift guard does not need to be re-tuned by hand.

Reproduce locally with one of:

```bash
# Operator-facing reproduction of the recommended mix ladder.
SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example poker_quality_benchmark

# Wider ladder for stricter-match experiments.
SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example poker_quality_benchmark -- 0.0 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 1.0

# Unit-test gate (shares a single source of truth with the example).
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- poker_quality_benchmark

# CI-grade E2E proof harness.
bash tests/e2e/poker_quality_benchmark.sh
```

A solver-constant change that shifts the live `mix_ladder` off the
documented `0.0,0.25,0.5,0.75,1.0` sequence — or shifts the
`reference_self_match_count` / `reference_self_match_l1` anchors off
their `80` / `0.0` values — is a regression of the operator-facing
guarantee. The E2E harness `tests/e2e/poker_quality_benchmark.sh` is
the gate that catches it, and the F-003 doc-reg drift guard
`tests/e2e/miner_convergence_doc_keeps_truthful_thresholds.sh` is the
secondary guard that prevents the operator-guide text from drifting away
from the live benchmark.

## Poker Convergence Status

Poker is on the resolved list **via the F-003 / NEM-001B mix-ladder
substitute**, not via a positive-iteration exploitability ladder. The
truthful poker exploitability ladder requires a full encoder, and the
checked-in `bootstrap_artifacts.rs` emits a deliberately sparse
`NlheArtifactDossier` with `postflop_complete=false`. Positive-iteration
poker training against the checked-in artifacts therefore fails upstream
with `isomorphism not found` (the same path that keeps the stage-0 miner
self-check honest), and a real exploitability number cannot be recorded
against the bootstrap fixtures without lying about the dossier's
`passing=true` claim.

The mix-ladder is the truthful substitute: instead of training a
candidate and measuring its exploitability, it builds the closed-form
reference profile, builds a convex perturbation of it, and measures
the L1 distance + exact-action-match count between the candidate and
the reference pack on a fresh `bootstrap_scenarios()` reference. The
helper is reproducible end-to-end on a fresh checkout, monotonic in
`mix` (the helper's contract), and proves the candidate-vs-reference
benchmark harness is wired correctly without depending on richer
encoder artifacts. The "self-match is the only currently-useful
checkpoint" caveat still holds — `mix=1.0` is the only ladder point
where the candidate is provably bit-equal to the reference, and any
positive-iteration poker training past the zero-th iteration is still
rejected upstream by `isomorphism not found`.

The repo ships the truthful full-encoder path for operators who have the
required robopoker/PostgreSQL hardware footprint
([`poker-quality-benchmark.md`](../execution-playbooks/poker-quality-benchmark.md)
and `bash ops/poker_quality_benchmark.sh`), but it is explicitly an
out-of-band operator run, not a stage-0 miner default. The recommended
path for an operator who needs a real positive-iteration poker exploitability
number today is:

1. Materialize a full `isomorphism` lookup from robopoker's PostgreSQL
   export per the playbook.
2. Run `bash ops/poker_quality_benchmark.sh` to record an exploitability
   ladder.
3. Record the chosen iteration count + threshold against the recorded
   ladder, and treat the recorded number as that operator's per-deploy
   poker convergence floor.

This is the truthful shape the F-003 / NEM-001B blocker notes call out:
the recommendation must be derived from a real benchmark surface, not
from a placeholder or a same-checkpoint self-score, and the mix-ladder
is the closest truthful surface available against the checked-in
sparse bootstrap encoder. The remaining unblock for a
positive-iteration poker exploitability ladder is the PROMOTE-001
external artifact supply, which is outside this row's scope by design.

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
- `tests/e2e/poker_quality_benchmark.sh` — the CI-grade executable
  proof that the NEM-001B mix-ladder row above stays true (asserts
  `reference_self_match_count=80`, `reference_self_match_l1=0.0`,
  `match_ratio_threshold=0.95`, and the monotonic
  `mix_ladder=0.0,0.25,0.5,0.75,1.0`).
- `tests/e2e/miner_convergence_doc_keeps_truthful_thresholds.sh` — the
  F-003 doc-reg drift guard that prevents this doc from drifting away
  from the live benchmark, the live `LIARS_DICE_SOLVER_TREES` constant,
  the live `recommended_minimum_iterations` value, and the live poker
  `POKER_QUALITY_BENCHMARK` configuration.
