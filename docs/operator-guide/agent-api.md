# Agent / Operator Read-Only Solver Surface (W-03)

This document is the public operator-facing contract for the Myosu
`myosu-solver-read` binary: the read-only, JSON-in / line-out solver
surface an external agent, partner, or human operator can call without
going through a chain RPC, opening a wallet, or holding a token. The
binary is the single stable read-only path over every
`benchmarked` portfolio game. The companion executable drift guard lives
at `tests/e2e/solver_read.sh` and is wired into the `solver-read` CI
job.

> The Eng + Design lens: external agents and humans need a stable
> read-only surface, not a chain RPC. The chain RPC is the
> miner/validator surface; the read surface here is the agent/operator
> surface. They share zero state, zero auth, and zero tokens. If a
> regression breaks the line protocol or routes a non-portfolio-routed
> game through the rule-aware engine, this gate fails closed.

## What the read surface guarantees

### Read-only by construction

The binary never:

- mutates chain state (no extrinsics, no storage writes)
- opens or touches a wallet
- emits, signs, or verifies a policy bundle
- registers an operator or a miner
- requires authentication, a key, or a token

If abuse becomes a problem, the rate-limit is a separate ticket, not
a permissioned-API problem. The surface is intentionally open; the
`myosu-solver-read` process is the only thing an external caller
needs to spawn.

### Stable line protocol

For every well-formed request, the binary prints exactly one line to
stdout:

```text
SOLVER_READ game=<slug> action=<action-label> confidence=<f32> engine_tier=<rule-aware|static-baseline> engine_family=<...> legal_action_count=<usize> elapsed_ms=<u64>
```

For every malformed / fail-closed request, the binary prints exactly
one line to stdout and exits non-zero:

```text
SOLVER_READ_FAIL reason=<reason>
```

The line is grep-friendly: a wrapper script can `grep ^SOLVER_READ`
for success and `grep ^SOLVER_READ_FAIL` for triage. The `reason`
field is sanitized to one line (embedded newlines are escaped to
`\n`) so a downstream parser never sees a stray newline.

### Stable input JSON shape

The binary reads a single JSON object from stdin:

```json
{ "game": "<slug>", "challenge": { <PortfolioChallenge variant JSON> } }
```

`game` is a research-game slug (see the supported-slug table below).
`challenge` is the matching `PortfolioChallenge` variant from
`myosu_games_portfolio::state::PortfolioChallenge`; the variant key
must be the snake_case form (`"cribbage"`, `"hearts"`, `"dou_di_zhu"`,
etc.). The binary uses `serde_json::from_str` with
`#[serde(deny_unknown_fields)]` on the outer request, so an unknown
top-level field fails closed.

## Supported game slugs

Every `benchmarked` portfolio game is supported. The two dedicated
solver games (`nlhe-heads-up`, `liars-dice`) are explicitly rejected
with `SOLVER_READ_FAIL reason=not_portfolio_routed: <slug> has a
dedicated solver crate; use that crate's read surface` — they have
their own read surfaces in the `myosu-games-poker` and
`myosu-games-liars-dice` crates and are NOT served by the portfolio
binary.

| Slug | Engine family | Engine tier | Sources of truth |
|---|---|---|---|
| `cribbage` | `state-aware pegging-crib heuristic` | `rule-aware` | `outputs/solver-promotion/cribbage/`, `crates/myosu-games-portfolio/src/cribbage_benchmark.rs` |
| `hearts` | `state-aware hearts penalty heuristic` | `rule-aware` | `outputs/solver-promotion/hearts/`, `crates/myosu-games-portfolio/src/hearts_benchmark.rs` |
| `bridge` | `state-aware bridge contract heuristic` | `rule-aware` | `outputs/solver-promotion/bridge/`, `crates/myosu-games-portfolio/src/bridge_benchmark.rs` |
| `spades` | `state-aware spades trick heuristic` | `rule-aware` | `outputs/solver-promotion/spades/`, `crates/myosu-games-portfolio/src/spades_benchmark.rs` |
| `call-break` | `state-aware call-break trick heuristic` | `rule-aware` | `outputs/solver-promotion/call-break/`, `crates/myosu-games-portfolio/src/call_break_benchmark.rs` |
| `gin-rummy` | `state-aware gin-rummy meld heuristic` | `rule-aware` | `outputs/solver-promotion/gin-rummy/`, `crates/myosu-games-portfolio/src/gin_rummy_benchmark.rs` |
| `dou-di-zhu` | `state-aware bomb-preservation heuristic` | `rule-aware` | `outputs/solver-promotion/dou-di-zhu/`, `crates/myosu-games-portfolio/src/dou_di_zhu_benchmark.rs` |
| `backgammon` | `state-aware backgammon race heuristic` | `rule-aware` | `outputs/solver-promotion/backgammon/`, `crates/myosu-games-portfolio/src/backgammon_benchmark.rs` |
| `hanafuda-koi-koi` | `state-aware hanafuda yaku heuristic` | `rule-aware` | `outputs/solver-promotion/hanafuda-koi-koi/`, `crates/myosu-games-portfolio/src/hanafuda_benchmark.rs` |
| `hwatu-go-stop` | `state-aware hwatu go-stop heuristic` | `rule-aware` | `outputs/solver-promotion/hwatu-go-stop/`, `crates/myosu-games-portfolio/src/hwatu_benchmark.rs` |
| `stratego` | `state-aware belief-scout heuristic` | `rule-aware` | `outputs/solver-promotion/stratego/`, `crates/myosu-games-portfolio/src/stratego_benchmark.rs` |
| `nlhe-six-max` | `state-aware poker range heuristic` | `rule-aware` | `outputs/solver-promotion/nlhe-six-max/`, `crates/myosu-games-portfolio/src/nlhe_six_max_benchmark.rs` |
| `plo` | `state-aware PLO nut-draw heuristic` | `rule-aware` | `outputs/solver-promotion/plo/`, `crates/myosu-games-portfolio/src/plo_benchmark.rs` |

The seven `routed` portfolio games (`nlhe-tournament`, `short-deck`,
`teen-patti`, `riichi-mahjong`, `ofc-chinese-poker`, `pusoy-dos`,
`tien-len`) are NOT supported yet — they have no engine
implementation, so a `scenario → recommendation` call would not be
meaningful. They will land as their own dossier slices advance.

The list above is exactly the rows of `ops/solver_promotion.yaml` that
satisfy `route: portfolio, tier: benchmarked`. The two sources of
truth must stay in sync — a drift in either is a W-03 follow-up
finding, not a silent override of the supported-slug table.

## How to call it

### Build

```bash
SKIP_WASM_BUILD=1 cargo build -p myosu-games-portfolio --bin myosu_solver_read
```

The binary lives at `target/debug/myosu_solver_read` (release path:
`target/release/myosu_solver_read`).

### Invoke

```bash
echo '{
  "game": "cribbage",
  "challenge": {
    "cribbage": {
      "spot": {
        "challenge_id": "discard-pressure",
        "decision": "Cribbage discard pressure spot",
        "rule_file": "20-cribbage.md",
        "solver_family": "rule-aware"
      },
      "pegging_count": 4,
      "run_potential": 2,
      "crib_edge": 0,
      "pair_trap": false,
      "go_window": false,
      "fifteen_outs": 1,
      "max_immediate_points": 2
    }
  }
}' | target/debug/myosu_solver_read
```

A live call returns one line:

```text
SOLVER_READ game=cribbage action=peg-run confidence=0.591837 engine_tier=rule-aware engine_family=state-aware pegging-crib heuristic legal_action_count=3 elapsed_ms=0
```

A curl-style invocation is intentionally not part of the contract —
this surface is a process-spawn, not an HTTP endpoint. The
hosting/rate-limiting question is operator-side (see the W-01 public
testnet contract for the chain-RPC equivalent).

### Fail-closed responses

| Trigger | Line | Exit code |
|---|---|---|
| Empty stdin | `SOLVER_READ_FAIL reason=empty_stdin` | non-zero |
| Malformed JSON | `SOLVER_READ_FAIL reason=invalid_json: <serde error>` | non-zero |
| Empty / whitespace `game` slug | `SOLVER_READ_FAIL reason=empty_game_slug` | non-zero |
| Unknown `game` slug | `SOLVER_READ_FAIL reason=unknown_game: <slug>` | non-zero |
| `game` slug not portfolio-routed (NLHE heads-up, Liar's Dice) | `SOLVER_READ_FAIL reason=not_portfolio_routed: <slug> has a dedicated solver crate; use that crate's read surface` | non-zero |
| `game` slug != challenge variant key (mismatch guard) | `SOLVER_READ_FAIL reason=game_mismatch: slug=<slug> challenge_variant=<variant>` | non-zero |
| Engine dispatch error | `SOLVER_READ_FAIL reason=engine_dispatch_failed: <error>` | non-zero |
| Engine returns no recommendation | `SOLVER_READ_FAIL reason=empty_recommendation` | non-zero |
| Engine returns non-finite confidence | `SOLVER_READ_FAIL reason=non_finite_confidence: <confidence>` | non-zero |
| Stdin read error | `SOLVER_READ_FAIL reason=stdin_read_failed: <io error>` | non-zero |

## How the surface is wired

- **Binary**: `crates/myosu-games-portfolio/src/bin/myosu_solver_read.rs`
  (the entry point, `ExitCode` return, `serde_json::from_str` with
  `deny_unknown_fields`, plus 7 unit tests in `mod tests`)
- **Public dispatch surface**: `myosu_games_portfolio::answer_typed_challenge`
  (the typed `PortfolioChallenge` → `EngineAnswer` dispatch)
- **Recommended-action helper**: `myosu_games_portfolio::recommended_action`
  (the argmax picker that turns a `PortfolioStrategyResponse` into the
  single label the line protocol prints)
- **Engine tier tag**: `myosu_games_portfolio::EngineTier` (the
  `RuleAware` / `StaticBaseline` enum that the line's `engine_tier=`
  field serializes from)
- **Slug resolution**: `myosu_games_portfolio::ResearchGame::from_slug`
  (the inverse of `ResearchGame::slug` the binary uses to parse the
  outer `game` field)
- **Proof harness**: `tests/e2e/solver_read.sh` (9 sub-checks: binary
  builds, three happy-path dispatches across the three engine
  families, three fail-closed paths, the bin's 7 unit tests pass, the
  full portfolio suite stays green)
- **CI job**: `.github/workflows/ci.yml` `solver-read` job
  (the executable gate that runs the proof harness on every push / PR)
- **README pointer**: `README.md` Operator Path (the one-line link
  an operator follows to reach this doc)

## Companion docs

- [docs/operator-guide/public-testnet.md](public-testnet.md) — the
  chain-side public read contract (W-01); the chain RPC is the
  miner/validator surface, this doc is the agent/operator surface,
  and the two stay disjoint
- [docs/operator-guide/architecture.md](architecture.md) — the
  operator-facing mental model that names the
  read-only-by-construction property of the rule-aware engine
- [docs/operator-guide/quickstart.md](quickstart.md) — the operator
  path that this surface is a thin slice of
- [genesis/plans/000-ceo-testnet-roadmap.md](../../genesis/plans/000-ceo-testnet-roadmap.md) —
  the CEO testnet roadmap that names the read surface as a
  milestone

## Dedicated-solver games (liars-dice, nlhe-heads-up)

The W-03 `myosu-solver-read` binary explicitly rejects the two
non-portfolio-routed dedicated-solver games (`nlhe-heads-up` and
`liars-dice`). The W-07 `myosu-solver-read-dedicated` binary is the
read-only, JSON-in / line-out solver surface for those two games.
It is a separate binary because the dedicated solver crates have
different input contracts (a checkpoint file path, and for NLHE an
encoder directory) than the rule-aware portfolio engine.

### Build

```bash
SKIP_WASM_BUILD=1 cargo build -p myosu-solver-read-dedicated --bin myosu-solver-read-dedicated
```

The binary lives at `target/debug/myosu-solver-read-dedicated`.

### Stable input JSON shape

The binary reads a single JSON object from stdin:

```json
{
  "game": "liars-dice",
  "checkpoint": "/absolute/path/to/checkpoint.bin",
  "query": { <LiarsDiceStrategyQuery JSON> }
}
```

or for NLHE:

```json
{
  "game": "nlhe-heads-up",
  "checkpoint": "/absolute/path/to/checkpoint.bin",
  "encoder_dir": "/absolute/path/to/nlhe/encoder/dir",
  "query": { <NlheStrategyQuery JSON> }
}
```

`game` is one of `liars-dice` or `nlhe-heads-up`. `checkpoint` is a
`MYOS`-magic + version-1 checkpoint file produced by the matching
dedicated solver crate. `encoder_dir` is required only for NLHE and
must be the bootstrap encoder directory shape the
`myosu-games-poker` artifact surface documents. `query` is the same
wire type the W-02 policy-bundle examples serialize. The outer
request uses `#[serde(deny_unknown_fields)]`, so unknown top-level
fields fail closed.

### Stable line protocol

For every well-formed request, the binary prints exactly one line
to stdout:

```text
SOLVER_READ game=<slug> action=<edge-debug> confidence=<f32> engine_tier=dedicated-cfr checkpoint_sha256=<64-hex> legal_action_count=<usize> elapsed_ms=<u64>
```

The `checkpoint_sha256` field is the SHA-256 of the checkpoint file
bytes the answer was computed from, so an operator can `sha256sum`
the checkpoint locally and verify the answer's provenance.

For every malformed / fail-closed request, the binary prints
exactly one line to stdout and exits non-zero:

```text
SOLVER_READ_FAIL reason=<reason>
```

The `reason` field is sanitized (embedded newlines collapsed to
`\\n`, embedded `=` replaced with `_`) so a wrapper script can
`grep ^SOLVER_READ_FAIL reason=...` without a multi-line parser.

### Fail-closed responses

| Trigger | Line | Exit code |
|---|---|---|
| Empty stdin | `SOLVER_READ_FAIL reason=io: empty stdin` | non-zero |
| Malformed JSON | `SOLVER_READ_FAIL reason=query_decode: ...` | non-zero |
| Unknown `game` slug | `SOLVER_READ_FAIL reason=unknown_game: <slug>` | non-zero |
| Missing `checkpoint` field | `SOLVER_READ_FAIL reason=query_decode: ...` | non-zero |
| Checkpoint does not exist | `SOLVER_READ_FAIL reason=not_a_directory: <path>` | non-zero |
| Checkpoint is empty | `SOLVER_READ_FAIL reason=empty_checkpoint: <path>` | non-zero |
| Checkpoint magic wrong | `SOLVER_READ_FAIL reason=checkpoint_magic: <found>` | non-zero |
| Checkpoint version wrong | `SOLVER_READ_FAIL reason=checkpoint_version: found=X expected=1` | non-zero |
| Missing `encoder_dir` (NLHE) | `SOLVER_READ_FAIL reason=missing_encoder_dir` | non-zero |
| `encoder_dir` not a directory | `SOLVER_READ_FAIL reason=not_a_directory: <path>` | non-zero |
| Encoder directory fails to load | `SOLVER_READ_FAIL reason=encoder_load: ...` | non-zero |
| Solver fails to load checkpoint | `SOLVER_READ_FAIL reason=solver_load: ...` | non-zero |
| Query decode fails | `SOLVER_READ_FAIL reason=query_decode: ...` | non-zero |
| Empty recommendation | `SOLVER_READ_FAIL reason=empty_recommendation` | non-zero |
| Non-finite confidence | `SOLVER_READ_FAIL reason=non_finite_confidence: ...` | non-zero |

### How the surface is wired

- **Binary**: `crates/myosu-solver-read-dedicated/src/bin/myosu_solver_read_dedicated.rs`
  (the entry point, stdin→JSON→dispatch→`SOLVER_READ` line, plus
  sanitize-and-exit-1 failure path)
- **Public dispatch surface**: `myosu_solver_read_dedicated::answer_liars_dice`
  and `myosu_solver_read_dedicated::answer_nlhe` (the typed helpers
  that load the checkpoint + encoder and run the query)
- **Recommended-action helpers**:
  `myosu_solver_read_dedicated::liars_dice_recommendation` and
  `myosu_solver_read_dedicated::nlhe_recommendation` (the argmax
  pickers that turn a solver response into the single label the
  line protocol prints)
- **Checkpoint hash helper**:
  `myosu_solver_read_dedicated::checkpoint_sha256` (SHA-256 over
  the checkpoint bytes, lowercase hex)
- **Proof harness**: `tests/e2e/solver_read_dedicated.sh` (8
  sub-checks: binary builds, Liar's Dice happy path, NLHE happy
  path, unknown slug fail-closed, missing checkpoint fail-closed,
  missing encoder_dir fail-closed, checkpoint SHA-256 byte-stable,
  crate unit tests green)
- **CI job**: `.github/workflows/ci.yml`
  `solver-read-dedicated` job (the executable gate that runs the
  proof harness on every push / PR)
- **README pointer**: `README.md` Operator Path (the one-line link
  an operator follows to reach this doc)

### Companion docs

- [docs/operator-guide/public-testnet.md](public-testnet.md) — the
  chain-side public read contract (W-01)
- [docs/operator-guide/quickstart.md](quickstart.md) — the operator
  path that this surface is a thin slice of
- `crates/myosu-games-liars-dice/src/policy_bundle.rs` — the Liar's
  Dice policy-bundle builder that defines the `LiarsDiceStrategyQuery`
  JSON contract
- `crates/myosu-games-poker/src/policy_bundle.rs` — the NLHE
  policy-bundle builder that defines the `NlheStrategyQuery` JSON
  contract
