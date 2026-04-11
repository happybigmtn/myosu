# Build a Play-Against-Strategy Harness for the Canonical Ten

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be maintained in accordance with it. This plan assumes the canonical truth layer and the all-ten core game logic gate from `EXECPLAN_CANONICAL_GAME_TRUTH_LAYER.md` and `EXECPLAN_CANONICAL_TEN_CORE_GAME_LOGIC.md`.

## Purpose / Big Picture

After this plan is implemented, Myosu will have a harness that actually plays each canonical-ten game against the best available local strategy for that game. This is stronger than the current smoke tests, which prove that the UI can print a state and accept a sample action. The new harness proves that canonical legal actions, solver recommendations, core transitions, and trace verification can run together for an actual play loop.

An operator should be able to run one command and see ten `PLAYTRACE` lines, one per canonical-ten game. Each line should name the game, strategy source, step count, terminal or bounded status, payoff if terminal, and canonical truth hash.

## Progress

- [x] (2026-04-10T00:00Z) Confirmed that `myosu-play --game <slug> --smoke-test` exists for research games, but it is a static smoke surface rather than a full play-against-best-strategy harness.
- [x] (2026-04-10T00:00Z) Confirmed that `tests/e2e/research_strength_harness.sh` proves strategy output quality and transport compatibility but not full play loops.
- [x] (2026-04-10T00:00Z) Add a local playtrace driver that uses canonical core states and best available local strategy responses.
- [x] (2026-04-10T00:00Z) Add a shell harness that runs the playtrace driver for all canonical-ten games.
- [x] (2026-04-10T00:00Z) Add negative tests for illegal action selection and trace hash mismatch.
- [x] (2026-04-10T00:00Z) Add canonical playtrace harness checks to CI active-crates coverage.
- [x] (2026-04-10T00:00Z) Expand the playtrace driver and shell harness from the canonical ten to all 22 research games.
- [ ] Optionally wire the driver into `myosu-play` after the standalone example is green.

## Surprises & Discoveries

- Observation: Existing play smoke reports do not answer "can I play each game against the best strategy?" For portfolio games, `portfolio_smoke_report` grabs the first renderer completion and returns `final_state=rule_aware_demo`; it does not loop through core transitions to terminal or bounded completion.
  Evidence: `crates/myosu-play/src/main.rs` chooses the first completion in `portfolio_smoke_report`, then prints `final_state=rule_aware_demo`.

- Observation: The phrase "best strategy" must be scoped. For the canonical ten, the best available local strategy is the current `PortfolioSolver::strength_query` / rule-aware engine output unless a stronger game-specific solver exists.
  Evidence: `specs/041026-research-portfolio-solvers.md` explicitly says the rule-aware reference engines are not a claim of solved or production-strength play.

- Observation: The current portfolio solver's action vocabulary is not identical to the new core action vocabulary. The playtrace driver maps solver hints when possible, and otherwise reports `strategy_source=best-local+legal-continuation` instead of silently pretending the solver selected the exact core move.
  Evidence: Hanafuda, OFC, Dou Di Zhu, and Backgammon playtraces use the legal-continuation fallback because the bootstrap strategy action does not decode to a currently legal bounded core action.

## Decision Log

- Decision: Start with a standalone example in `crates/myosu-games-canonical`, then wire `myosu-play` only after the example is green.
  Rationale: A harness should not depend on terminal rendering. A small example gives the validator-style proof first and keeps `myosu-play` simpler until the game cores are stable.
  Date/Author: 2026-04-10 / Codex

- Decision: A playtrace may end with `status=bounded` for games whose first core state does not yet reach a full terminal hand, but it must explain the reason and still prove all applied actions were legal.
  Rationale: Full terminal play for Bridge, Riichi, Stratego, and NLHE 6-max may require several milestones. A bounded honest trace is acceptable; a fake terminal payoff is not.
  Date/Author: 2026-04-10 / Codex

- Decision: The harness must run without a chain by default.
  Rationale: This is a game-engine and strategy proof. Chain-visible best-miner discovery can be added later, but the first proof should not require devnet startup.
  Date/Author: 2026-04-10 / Codex

## Outcomes & Retrospective

The standalone playtrace driver and shell harness are implemented for both the original canonical ten and the full 22-game research corpus. A local run with `seed=42` and `max_steps=8` produced deterministic `PLAYTRACE` lines for every research game. OFC reaches the bounded terminal foul scenario with payoff `-6,6`, Liar's Dice reaches a terminal challenge resolution, and bounded representative states continue to report `status=bounded` plus `terminal=false` instead of fabricating end-of-game scores.

The `myosu-play` CLI integration remains intentionally deferred because the standalone harness is green and the execplan marked that integration optional.

## Context and Orientation

The current evidence surfaces are:

- `tests/e2e/research_games_harness.sh` for all-corpus transport and smoke coverage.
- `tests/e2e/research_strength_harness.sh` for all-corpus strategy strength output.
- `crates/myosu-play/src/main.rs` for smoke, train, and pipe modes.
- `crates/myosu-games-portfolio/src/solver.rs` for `PortfolioSolver`, strength queries, and quality reports.
- `crates/myosu-games-portfolio/src/renderer.rs` for portfolio render output.

This plan adds a playtrace harness. A playtrace is a deterministic record of a game loop. It starts from a canonical core state, asks the best local strategy for an action distribution, decodes the recommended action to a canonical action id, applies it through core game logic, records the transition trace, and repeats until terminal or a configured step limit.

For this plan, "best available local strategy" means the strongest strategy route available in this working tree without chain access. For the canonical ten, that is the portfolio rule-aware `PortfolioSolver` strength route. Later plans can replace individual games with dedicated CFR, PIMC, rollout, or external artifact-backed solvers as they land.

## Plan of Work

Create a playtrace driver after the core game API exists. Put the driver example in the canonical crate:

- `crates/myosu-games-canonical/examples/play_against_strategy.rs`

The example should accept:

- `--game <slug>` for one game.
- `--all-canonical-ten` for the full ten-game run.
- `--max-steps <N>` with a default such as 200.
- `--seed <u64>` with a default such as 1.
- `--policy best-local` as the default, with optional `legal-first` and `random-legal` for negative and baseline tests.

For each step, the driver should:

1. Build or carry a core state.
2. Convert the core state to a canonical snapshot.
3. Ask the local solver for a strategy response if the current state can be represented by the solver. If the solver only supports the bootstrap challenge for the first version, use it for step one and then fall back to a legal policy with `strategy_source=best-local+legal-continuation`.
4. Select the highest-probability legal action. If the solver recommends an action that is not legal in the current state, fail the trace.
5. Apply the action through the core transition API.
6. Append a `CanonicalTransitionTrace`.
7. Stop when the core state is terminal or when `max_steps` is reached.

The printed output for each game should be one stable line:

    PLAYTRACE game=bridge status=bounded steps=4 strategy_source=best-local+legal-continuation terminal=false payoff=none truth_hash=<hex>

If a game reaches terminal:

    PLAYTRACE game=cribbage status=terminal steps=3 strategy_source=best-local terminal=true payoff=2,-2 truth_hash=<hex>

Add a shell harness:

- `tests/e2e/canonical_ten_play_harness.sh`

The shell harness should derive the game list from the Rust-owned canonical-ten manifest rather than duplicating the list in shell. If the manifest example prints slugs, use that output. If a stable Rust manifest function exists but no example exists, add the example first.

The harness should assert:

- exactly ten `PLAYTRACE` lines were produced;
- every canonical-ten game appears once;
- every line has `strategy_source=best-local` or `strategy_source=best-local+legal-continuation`;
- no line has `illegal_action`, `trace_hash_mismatch`, or `panic`;
- terminal games include a payoff;
- bounded games include a reason or `terminal=false`;
- running the harness twice with the same seed produces identical `truth_hash` values.

After the standalone harness is green, decide whether to wire it into `myosu-play`. If doing so in this plan, add:

- A `--play-harness` or `--playtrace` flag to `crates/myosu-play/src/cli.rs`.
- A noninteractive path in `crates/myosu-play/src/main.rs` that calls the playtrace driver and prints the same `PLAYTRACE` lines.
- Tests proving `myosu-play --game bridge --playtrace --max-steps 4` prints a `PLAYTRACE game=bridge` line.

Do not route this through the ratatui terminal. This is a harness, not a TUI feature.

## Concrete Steps

Start by running the existing smoke and strength harnesses:

    bash tests/e2e/research_strength_harness.sh
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-play -- --game bridge --smoke-test

Add the standalone `play_against_strategy` example in the crate that owns the canonical/core APIs after those APIs exist.

Run one game while developing:

    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical --example play_against_strategy -- --game bridge --max-steps 8 --seed 1

Then add the shell harness:

    bash tests/e2e/canonical_ten_play_harness.sh

If `myosu-play` integration is included, run:

    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-play -- --game bridge --playtrace --max-steps 8

## Validation and Acceptance

This plan is accepted when:

- A standalone `play_against_strategy` example exists and runs for every canonical-ten game.
- `tests/e2e/canonical_ten_play_harness.sh` exists and derives its game list from the Rust canonical-ten manifest.
- The harness prints exactly ten `PLAYTRACE` lines.
- Two harness runs with the same seed produce the same truth hashes.
- A forced illegal action test fails before transition.
- A trace hash mismatch test fails validation.
- Existing research strength and smoke harnesses still pass.

Run this validation set from the repository root:

    bash tests/e2e/research_strength_harness.sh
    bash tests/e2e/canonical_ten_play_harness.sh
    bash tests/e2e/canonical_ten_play_harness.sh > target/e2e/playtrace-run-1.txt
    bash tests/e2e/canonical_ten_play_harness.sh > target/e2e/playtrace-run-2.txt
    diff -u target/e2e/playtrace-run-1.txt target/e2e/playtrace-run-2.txt
    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-canonical

If `myosu-play` integration is added, also run:

    SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-play playtrace
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-play -- --game bridge --playtrace --max-steps 8

## Idempotence and Recovery

The harness should write only under `target/e2e/` and stdout. It must be safe to rerun.

If a game loops without terminal, stop at `max_steps` and mark `status=bounded`. Do not hide the bounded status. If a game produces zero legal actions but is not terminal, fail the harness and fix the core game logic.

If a solver recommendation cannot decode to a legal core action, fail the trace. Do not silently choose a different action unless the output names the fallback policy and records that the best local strategy was not applicable to that state.

## Interfaces and Dependencies

The driver should call the canonical/core APIs directly. Avoid shelling out from Rust to `myosu-play` or the examples.

Keep chain discovery out of the first harness. A later plan can add a `--strategy-source chain-best` option once portfolio games support chain-visible miner discovery. Today only poker supports `--chain/--subnet` miner discovery in `myosu-play`, so pretending chain discovery covers all canonical-ten games would be misleading.

## Artifacts and Notes

The important artifacts are:

- `play_against_strategy.rs` example in the canonical crate.
- `tests/e2e/canonical_ten_play_harness.sh`.
- `tests/e2e/research_play_harness.sh`.
- Ten stable `PLAYTRACE` lines.
- Twenty-two stable research-corpus `PLAYTRACE` lines.
- Deterministic trace output under `target/e2e/playtrace-run-*.txt`.

Change note, 2026-04-10 / Codex: Initial plan written to answer the harness gap: current Myosu smoke tests demonstrate surface wiring, but not actual play loops against the strongest available local strategy for each canonical-ten game.

Change note, 2026-04-10 / Codex: Implemented `play_against_strategy`, `tests/e2e/canonical_ten_play_harness.sh`, playtrace determinism tests, and illegal-action/hash-mismatch negative tests. Verified the existing research strength harness and bridge smoke surface still pass.

Change note, 2026-04-10 / Codex: Extended `play_against_strategy` and CI coverage to the entire research corpus via `--all-research-games` plus `tests/e2e/research_play_harness.sh`.
