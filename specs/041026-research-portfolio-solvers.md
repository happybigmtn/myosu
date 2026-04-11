# Specification: Research Portfolio Solvers

Date: 2026-04-10

## Objective

Turn the `research/game-rules` corpus into a solver surface: every researched
game has a deterministic query, checkpointable solver, wire response, miner
file-serving path, validator scoring path, and first rule-aware strength
benchmark.

This started as a bootstrap baseline and now routes portfolio games through
compact rule-aware reference engines. It is still not a claim of solved or
production-strength play; the purpose is to let the network register, score,
and improve one solver per game behind the same contract.

## Game Coverage

The rules corpus has 21 files. `21-hearts-cribbage.md` contains two distinct
games, so the portfolio exposes 22 research games:

| Game | CLI slug | Solver family |
|------|----------|---------------|
| No-Limit Hold'em Heads-Up | `poker`, `nlhe-heads-up`, `nlhe-hu` | dedicated robopoker-backed poker solver |
| No-Limit Hold'em 6-Max | `nlhe-six-max` | portfolio abstracted MCCFR / blueprint poker policy |
| Pot-Limit Omaha | `plo` | portfolio abstracted MCCFR / blueprint poker policy |
| No-Limit Hold'em Tournament | `nlhe-tournament` | portfolio ICM-aware push/fold policy |
| Short Deck Hold'em | `short-deck` | portfolio abstracted MCCFR / blueprint poker policy |
| Teen Patti | `teen-patti` | portfolio blind/seen poker heuristic policy |
| Hanafuda Koi-Koi | `hanafuda-koi-koi` | portfolio Monte Carlo yaku-value policy |
| Hwatu Go-Stop | `hwatu-go-stop` | portfolio Monte Carlo yaku-value policy |
| Riichi Mahjong | `riichi-mahjong` | portfolio shanten/ukeire tile-efficiency policy |
| Contract Bridge | `bridge` | portfolio PIMC plus double-dummy-inspired policy |
| Gin Rummy | `gin-rummy` | portfolio meld-distance draw/discard policy |
| Stratego | `stratego` | portfolio belief-tracking deployment/play policy |
| Open Face Chinese Poker | `ofc-chinese-poker` | portfolio foul-aware placement policy |
| Spades | `spades` | portfolio trick-taking Monte Carlo control policy |
| Liar's Dice | `liars-dice` | dedicated Liar's Dice MCCFR solver |
| Dou Di Zhu | `dou-di-zhu` | portfolio shedding-game control policy |
| Pusoy Dos | `pusoy-dos` | portfolio shedding-game control policy |
| Tien Len | `tien-len` | portfolio shedding-game control policy |
| Call Break | `call-break` | portfolio trick-taking Monte Carlo control policy |
| Backgammon | `backgammon` | portfolio race/contact equity policy |
| Hearts | `hearts` | portfolio trick-taking Monte Carlo control policy |
| Cribbage | `cribbage` | portfolio pegging and crib-equity policy |

Kuhn Poker remains a separate exact benchmark solver under `--game kuhn`; it is
not part of `research/game-rules`, but it has the same compact
checkpoint/query/response roundtrip example as the dedicated research games so
the original three-game framework stays covered while the corpus expands.

## Artifacts

- Crate: `crates/myosu-games-portfolio`
- Portfolio-routed query example:
  `cargo run -p myosu-games-portfolio --example bootstrap_query -- <game> <query-file>`
- Portfolio-routed checkpoint example:
  `cargo run -p myosu-games-portfolio --example bootstrap_checkpoint -- <game> <checkpoint-file> [iterations]`
- Portfolio-routed solver/query/response/scoring roundtrip:
  `cargo run -p myosu-games-portfolio --example bootstrap_roundtrip -- <game> <output-dir> [iterations]`
- Portfolio-routed strength manifest:
  `cargo run -p myosu-games-portfolio --example strength_manifest -- [table|slugs]`
- Portfolio-routed strength roundtrip:
  `cargo run -p myosu-games-portfolio --example strength_roundtrip -- <game> <output-dir> [iterations]`
- Portfolio-routed typed strength query:
  `cargo run -p myosu-games-portfolio --example strength_query -- <game> <query-file>`
- Portfolio-routed typed strength validation check:
  `cargo run -p myosu-games-portfolio --example strength_validate -- <game> <checkpoint-file> <strength-query-file> <strength-response-file>`
- Portfolio-routed quality budget:
  `cargo run -p myosu-games-portfolio --example engine_quality_budget`
- Portfolio-routed latency budget:
  `MYOSU_ENGINE_BUDGET_MS=50 cargo run -p myosu-games-portfolio --example engine_latency_budget`
- Portfolio-routed offline validation check:
  `cargo run -p myosu-games-portfolio --example bootstrap_validate -- <game> <checkpoint-file> <query-file> <response-file>`
- Research game manifest:
  `cargo run -p myosu-games-portfolio --example bootstrap_manifest -- [table|all-slugs|portfolio-slugs|dedicated-slugs]`
- Checkpoint magic: `MYOP`, version `2`, and an optional scoped
  `ResearchGame`. Miner-generated portfolio checkpoints are scoped to the
  requested game; unscoped solvers are only for in-process demo/test surfaces.
- Portfolio query/checkpoint compatibility: miner strategy serving, validator
  scoring, and artifact-backed play reject a checkpoint/query pair when the
  checkpoint game does not match the query or requested CLI game. Miner and
  validator file-serving paths accept both the legacy bootstrap strategy query
  and the typed strength query for portfolio-routed games.
- Wire format: bincode fixed-int with trailing-byte rejection
- Shared framework mapping:
  - `ResearchGame::game_type()` maps every research game to a built-in `myosu_games::GameType`.
  - `ResearchGame::game_config()` produces a `GameConfig` with typed params for `nlhe-heads-up` and `liars-dice`, and metadata-backed `GameParams::Custom` for portfolio-routed games.
  - `myosu-games-portfolio` tests assert that the 21 markdown files in
    `research/game-rules` exactly match the exposed research-game rule-file
    mapping, with `21-hearts-cribbage.md` intentionally mapped to two game
    identities.
- Miner path: `myosu-miner --game <portfolio-slug> --query-file ...`
  reports a `quality_summary` with engine tier, engine family, challenge id,
  quality score, baseline L1 distance, legal action count, and determinism.
- Validator path: `myosu-validator --game <portfolio-slug> --checkpoint ... --query-file ... --response-file ...`
  keeps the exact-match transport score and also reports the same portfolio
  strength `quality_summary`.
- Miner, validator, and play CLIs accept both the stable hyphenated slug and
  canonical chain id for built-in framework games. For example,
  `riichi-mahjong` and `riichi_mahjong` select the same game.
- Dedicated research game paths:
  - `nlhe-heads-up` / `nlhe-hu` are aliases for the existing poker solver and use `myosu-games-poker --example bootstrap_artifacts`, `myosu-games-poker --example bootstrap_roundtrip`, and `myosu-games-poker --example strength_roundtrip`.
  - `liars-dice` uses the dedicated `myosu-games-liars-dice --example bootstrap_query`, `myosu-games-liars-dice --example bootstrap_roundtrip`, and `myosu-games-liars-dice --example strength_roundtrip` surfaces.
- Dedicated non-research benchmark path:
  - `kuhn` uses the exact `myosu-games-kuhn` solver and `myosu-games-kuhn --example bootstrap_roundtrip`.
- Play surface: `myosu-play --game <research-slug> --smoke-test` accepts
  every research game. Portfolio-routed games use `PortfolioRenderer` backed by
  the typed rule-aware challenge surface; dedicated games use their existing
  renderers. Artifact-backed portfolio play is proven with
  `--require-artifact --smoke-checkpoint <checkpoint-file>`.

## Verification

- `cargo test -p myosu-games-portfolio --quiet`
- `SKIP_WASM_BUILD=1 cargo test -p myosu-miner portfolio --quiet`
- `SKIP_WASM_BUILD=1 cargo test -p myosu-validator portfolio --quiet`
- `bash tests/e2e/research_games_harness.sh`
- `bash tests/e2e/research_portfolio_harness.sh`
- `bash tests/e2e/research_strength_harness.sh`
- `research_games_harness.sh` and `research_portfolio_harness.sh` derive
  their game lists from `bootstrap_manifest` so the proof surface follows the
  Rust-owned corpus registry. `research_games_harness.sh` also checks the
  table view for the two dedicated routes and the split Hearts/Cribbage rule
  file mapping, then runs dedicated checkpoint/query/response roundtrips for
  NLHE heads-up, Liar's Dice, and the non-research Kuhn benchmark.
- `research_portfolio_harness.sh` runs the portfolio roundtrip example for
  every portfolio-routed game and requires `exact_match=true` /
  `score=1.000000`, re-validates every emitted checkpoint/query/response tuple
  through `bootstrap_validate`, then proves a scoped Bridge checkpoint rejects a
  Cribbage query before scoring. It also runs `strength_manifest`, requires
  every portfolio-routed game to report `engine_tier=rule-aware`, and runs
  `strength_roundtrip` for every portfolio-routed game.
- `research_strength_harness.sh` is the dedicated rule-aware proof. It runs
  portfolio `strength_roundtrip` for every portfolio-routed game, dedicated
  `STRENGTH` roundtrips for heads-up NLHE and Liar's Dice, the exact Kuhn
  benchmark roundtrip, and negative checks for cross-game and malformed typed
  strength queries.
- `strength_roundtrip` prints `baseline_l1_distance`, `elapsed_ms`,
  `budget_ms`, and `budget_status` when `MYOSU_ENGINE_BUDGET_MS` is
  configured. The lightweight budget examples print stable `ENGINE_QUALITY`
  and `ENGINE_LATENCY` key-value lines without adding a benchmark dependency.
  The default quality budget requires `score >= 1.01`, so a portfolio
  rule-aware engine must differ from the static compatibility baseline by
  top action or action distribution.
- CI gate: `.github/workflows/ci.yml` `active-crates` runs
  `research_games_harness.sh` and `research_strength_harness.sh`, while cargo
  check/test/clippy/rustfmt include `myosu-games-portfolio`.
- Representative live devnet proof:
  `MYOSU_E2E_GAMES='bridge' bash tests/e2e/validator_determinism.sh`
