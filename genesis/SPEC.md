# Myosu System Specification

Generated: 2026-04-11
Grounded in: `trunk @ 4e0b37fbaa` plus current working tree

## Problem Statement

Myosu is trying to prove a decentralized solver stack for imperfect-information
games: chain coordination, off-chain solver computation, off-chain validation,
and a shared human/agent consumption surface.

The important current-state caveat is that the repo already proves local
end-to-end wiring, but it does **not** yet prove independent solver-quality
measurement for the flagship poker path.

## System Identity

Today, Myosu is best described as a stage-0 local proof stack with these real
capabilities:

1. A Substrate runtime and node that can author blocks locally and expose the
   game-solver pallet at runtime index `7`.
2. Miner and validator binaries that can register, exchange strategy files, and
   submit weights on a local or operator-provided chain endpoint.
3. Dedicated solver paths for NLHE heads-up, Liar's Dice, and Kuhn, plus
   portfolio-routed rule-aware engines for 20 additional research games.
4. A terminal gameplay surface (`myosu-play`) with smoke-test, train, and pipe
   modes.

The intended direction is a solver marketplace with promotion tiers and shared
policy-bundle contracts, but those promotion surfaces are not implemented yet.

## Current Architecture

```text
chain
  runtime index 7 -> SubtensorModule / pallet-game-solver
  local proof posture -> dev/local/test_finney paths are runnable; finney mainnet spec is still stubbed

miners
  myosu-miner
  poker HTTP axon or file-based response path
  bounded training and checkpoint writing

validators
  myosu-validator
  file-based or poker HTTP query path
  current score = L1 distance against validator-loaded checkpoint expectation

gameplay
  myosu-play + myosu-tui
  smoke-test, train, pipe
  human and agent access through the same shell contract
```

## Crate Map

| Crate | Current role |
|-------|--------------|
| `myosu-games` | Shared game traits, registry, canonical helper types |
| `myosu-games-poker` | NLHE solver wrapper, artifacts, benchmark surface, wire codec |
| `myosu-games-liars-dice` | Native Liar's Dice solver, checkpointing, wire codec |
| `myosu-games-kuhn` | Exact Kuhn solver and wire codec |
| `myosu-games-portfolio` | 22-game research manifest, 20 portfolio-routed rule-aware engines |
| `myosu-games-canonical` | Canonical-ten truth layer and playtrace helpers |
| `myosu-miner` | Operator-facing miner bootstrap, training, serving |
| `myosu-validator` | Operator-facing validator scoring and weight submission |
| `myosu-play` | Human/agent gameplay entry point |
| `myosu-tui` | Shared TUI shell and renderer contracts |
| `myosu-keys` | Operator key lifecycle management |
| `myosu-chain-client` | Shared RPC client helpers |
| `pallet-game-solver` | Yuma/emission/staking/subnet logic inside the chain fork |
| `myosu-chain-runtime` | Runtime composition |
| `myosu-chain` | Node binary and chain spec wiring |

## Current Game Support

### Dedicated Paths

| Game | Solver path | Wire/storage surface | Important caveat |
|------|-------------|----------------------|------------------|
| NLHE heads-up | `myosu-games-poker` | `bincode 1.3` wire/checkpoint helpers plus artifact manifests | Positive-iteration training is blocked on sparse checked-in artifacts |
| Liar's Dice | `myosu-games-liars-dice` | `bincode 1.3` wire/checkpoint helpers | Exact exploitability is available and test-backed |
| Kuhn poker | `myosu-games-kuhn` | `bincode 1.3` wire/checkpoint helpers | Exact solver, not MCCFR training |

### Portfolio-Routed Paths

`myosu-games-portfolio` exposes:

- `ALL_RESEARCH_GAMES.len() == 22`
- `ALL_PORTFOLIO_ROUTED_GAMES.len() == 20`
- `NlheHeadsUp` and `LiarsDice` are intentionally **not** portfolio-routed

Those engines are explicitly described in code as “compact rule-aware reference
engines,” not as trained CFR solvers.

## Current Miner Behavior

`myosu-miner` currently supports:

- chain probing and registration
- axon publication
- bounded training or checkpoint bootstrap
- file-based strategy serving for all supported games
- HTTP `/strategy` only for poker

The important constraint is in `crates/myosu-miner/src/training.rs`:
positive-iteration poker training rejects artifact directories with
`postflop_complete=false`.

## Current Validator Behavior

`myosu-validator` currently:

- loads a checkpoint chosen on the validator CLI
- decodes a miner query/response pair
- computes an L1-distance score against the validator's own expected answer
- emits `exact_match=true` and `score=1.0` when the response matches that same
  checkpoint expectation

This makes the current validator path a determinism/self-consistency proof, not
an independent quality oracle. Independent benchmark surfaces exist separately,
for example poker's `benchmark_scenario_pack.rs`.

## Current Gameplay Behavior

`myosu-play` exposes:

- `--smoke-test` for fast visible proof
- `train` for interactive local TUI play
- `pipe` for line-oriented agent use

Only poker currently supports on-chain miner discovery via `--chain` and
`--subnet`; the CLI rejects those flags for other games.

## Emission and Economics

The chain fork contains Yuma/emission/staking machinery, but stage-0 economics
are intentionally simplified:

- `Stage0NoopSwap` is a 1:1 identity swap
- block emission defaults are tuned for local proofs, not market reality
- dust is measured and tolerated, not fully policy-resolved

That is enough for local chain proofs and not enough for funded-product claims.

## Invariants

The active repo invariants are still the ones in `INVARIANTS.md`:

1. structured closure honesty
2. proof honesty
3. validator determinism
4. miner/play dependency separation
5. plan/runtime coherence
6. robopoker fork coherence

## Near-Term Direction

The evidence-backed near-term direction inside this repo is:

1. define canonical policy-bundle and sampling-proof types
2. build a promotion ledger that cannot drift into aspiration
3. unblock truthful NLHE benchmark/provenance with pinned dossiers
4. promote NLHE and Liar's Dice only when their evidence clears the declared bar
5. deepen one portfolio-routed game, still defaulting to cribbage
6. freeze Myosu-side export artifacts and policy bundles as the input contract
   for a sibling Bitino local adapter
7. land the offline same-TUI Bitino pilot only after the policy/promotion
   gates are satisfied

Sibling Bitino implementation is a later milestone, not a current-state fact
about this repo.

## What Myosu Is Not Yet

- not a public production deployment
- not a verified independent validator-quality market for NLHE
- not a finished funded-settlement product
- not a web product
- not a repo with the sibling Bitino integration already landed
