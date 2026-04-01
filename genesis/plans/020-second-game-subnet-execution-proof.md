# Second-Game Subnet Execution Proof

Status: Completed locally on 2026-03-30. The remaining remote-only `010`
timing proof still lives outside the tree, but it no longer blocks the
second-game execution claim.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: New plan. The additive Liar's Dice architecture proof in `012` is
complete, but the master plan now calls for a fuller second-game end-to-end
execution story before doctrine/governance work resumes.

## Purpose / Big Picture

Poker already proves the stage-0 loop end to end:

`chain -> miner -> validator -> weights/emissions -> gameplay`

Liar's Dice currently proves only the additive seam:

`shared registry/traits -> second game crate lands without poker edits`

This plan closes the gap between those two truths. After this plan, Myosu has
one full poker subnet and one full Liar's Dice subnet sharing the same chain,
with distinct query/response surfaces, validator scoring, discovery, and a
consumable local gameplay or agent surface.

## Progress

- [x] (2026-03-29) Confirmed the remaining `010` work is external-only. The
  repo has no GitHub Actions workflow history yet on `happybigmtn/myosu`, so
  the CI timing proof cannot be completed from local code changes alone.
- [x] (2026-03-29) Mapped the real blocker for second-game execution: the chain
  client is generic enough already, but miner, validator, and play are still
  poker-specific because Liar's Dice lacked a wire-safe query/response seam.
- [x] (2026-03-29) Added the first reusable second-game seam in
  `myosu-games-liars-dice`: wire-safe `StrategyQuery` / `StrategyResponse`
  aliases, bounded bincode encode/decode helpers, and solver `query` /
  `answer` / `recommend` helpers.
- [x] (2026-03-29) Generalized `myosu-miner` around executable game selection
  for bounded local work. It now supports `--game poker` and
  `--game liars-dice` for checkpoint-backed training plus one-shot wire query
  serving, while keeping live HTTP axon explicitly unsupported for Liar's Dice
  instead of silently pretending the poker path is generic.
- [x] (2026-03-29) Generalized `myosu-validator` to score Liar's Dice miner
  responses deterministically from checkpoint-backed local expectation, using
  the same bounded query/response contract and L1-distance scoring shape as
  poker.
- [x] (2026-03-29) Added a real local second-game consumption surface in
  `myosu-play`. `--game liars-dice` now drives the shared shell and pipe
  surfaces with a built-in demo or checkpoint-backed renderer, while keeping
  chain miner discovery and live HTTP advice explicitly poker-only until the
  network path exists.
- [x] (2026-03-29) Extended the owned stage-0 harness far enough to prove the
  remaining open problem is the second subnet registration step itself. The
  harness now clears the full poker owned loop and only then stops at
  `registering_liars_dice_subnet`.
- [x] (2026-03-29) Ruled out two attractive but false explanations for that
  blocker. The live sudo path does lower `NetworkRateLimit` to `0`, and moving
  the second subnet to a distinct endowed operator key still ends in the same
  `register_network` timeout.
- [x] (2026-03-29) Made the shared chain client inclusion-aware for
  `register_network`, so the two-subnet blocker no longer collapses into a
  timeout. The owned loop now proves poker subnet registration writes
  `NetworkLastRegistered=2` at inclusion, and the later Liar's Dice failure is
  an actual runtime dispatch error, not just "no new subnet appeared."
- [x] (2026-03-29) Narrowed the remaining contradiction further. Right before
  the failing Liar's Dice registration, the owned loop sees
  `best_block=17 network_rate_limit=0 network_last_registered=2`, and the
  failing inclusion block still reads `network_rate_limit_at_block=0
  network_last_registered_at_block=2`, yet the runtime emits
  `ModuleError { index: 7, error: [32, 0, 0, 0] }`.
- [x] (2026-03-30) Proved the current local smoke lane is not honest enough to
  keep treating that contradiction as a live pallet bug. The cached runtime
  wasm under `target/debug/wbuild/myosu-chain-runtime/` is older than the
  current runtime sources, the stale wasm does not contain the newest
  `register_network_check` instrumentation string, and both
  `--dual-register-smoke` and `--stage0-local-loop-smoke` now fail fast with a
  stale-runtime explanation instead of surfacing misleading subnet-registration
  results.
- [x] (2026-03-30) Refreshed the embedded runtime wasm with a wasm-capable
  stable toolchain path, proved second-subnet registration succeeds again on
  fresh runtime code, fixed two shared-client default mismatches
  (`NetworkRateLimit` and `Tempo`) so bootstrap reads match pallet `ValueQuery`
  defaults, split the Liar's Dice owner/root harness commands so root-only sudo
  knobs run under `//Alice`, and closed the owned coexistence proof. The full
  local loop now completes with distinct poker and Liar's Dice subnets,
  positive validator permits, Bob -> Alice weights, and positive miner/validator
  economics for both games.

## Surprises & Discoveries

- Observation: the first real blocker was not chain state or subnet plumbing.
  It was missing second-game transport and scoring interfaces.
  Evidence: `myosu-chain-client` already works in terms of subnet IDs, serving
  metadata, stake, weights, and incentives. The poker lock-in starts higher up
  where miner, validator, and play import poker wire types directly.
- Observation: `myosu-games` already had the right generic transport shape.
  Evidence: `StrategyQuery<I>` and `StrategyResponse<E>` were already shared
  abstractions, so the honest first slice was to alias and wire them in
  Liar's Dice instead of inventing a new protocol crate.
- Observation: the next blocker after transport was persistence, not chain
  plumbing.
  Evidence: Liar's Dice already had exact solver logic, but miner and
  validator could not use it until the crate gained checkpoint save/load and a
  bounded training wrapper comparable to poker's local artifact path.
- Observation: live miner HTTP should not be generalized by implication.
  Evidence: once `--game` existed in `myosu-miner`, the honest behavior for
  `--serve-http --game liars-dice` was an explicit unsupported error until the
  second-game consumption surface is ready, not a poker-only path hidden
  behind a generic flag.
- Observation: the shared play shell can absorb a second game without pretending
  that every gameplay surface has the same network contract.
  Evidence: `myosu-play` now branches explicitly on game selection, reuses the
  same shell/pipe scaffolding for Liar's Dice, and keeps poker-only discovery
  and live query logic behind explicit poker checks instead of inventing a
  fake generic live-advice abstraction.
- Observation: the Milestone 5 blocker is narrower than "multi-game chain
  support is missing."
  Evidence: the owned smoke now proves poker end to end and reaches the second
  subnet registration step before failing.
- Observation: the current chain-client `register_network` proof remains too
  lossy for second-subnet diagnosis.
  Evidence: even after adding rate-limit and subnet-state context, the client
  still reports only a timeout because it waits on "new subnet appears" rather
  than the actual inclusion/dispatch outcome.
- Observation: the old "network state disappeared before the second subnet"
  theory was false.
  Evidence: poker registration now reports
  `network_last_registered_at_inclusion=2` and
  `network_last_registered_at_head=2`, and the failing Liar's Dice block still
  reads `network_last_registered_at_block=2`.
- Observation: the blocker has moved from missing state to an impossible live
  runtime contradiction.
  Evidence: the failure block still shows `network_rate_limit_at_block=0` and
  `network_last_registered_at_block=2`, which should satisfy
  `RegisterNetwork.passes_rate_limit`, yet the runtime emits pallet error
  `[32]`.
- Observation: that contradiction is no longer trustworthy as-is on the local
  smoke lane.
  Evidence: the node binary contains the newest
  `register_network_check` marker, the cached runtime wasm does not, and the
  smoke entrypoints now prove the cached wasm is older than the runtime source
  tree before any child node starts.
- Observation: the second-subnet contradiction was a stack of truth bugs, not a
  pallet impossibility.
  Evidence: once the embedded runtime wasm was rebuilt from current sources,
  `--dual-register-smoke` succeeded; then the shared client still needed
  pallet-default handling for `NetworkRateLimit` and `Tempo`, and the owned
  harness still needed to stop issuing root-only sudo calls under the Liar's
  Dice subnet owner key.

## Decision Log

- Decision: Keep Liar's Dice as the second-game proof target.
  Rationale: It is already the chosen additive proof game, its solver state
  space is small, and it tests a distinct imperfect-information bluffing shape
  rather than "more poker".
  Date/Author: 2026-03-29 / Genesis

- Decision: Start with local wire/query/response support in the game crate.
  Rationale: Miner and validator can only be generalized cleanly once the
  second game exposes a transport contract equivalent to poker.
  Date/Author: 2026-03-29 / Genesis

- Decision: Add checkpoint-backed Liar's Dice training before touching play.
  Rationale: The first honest cross-game proof after wire transport is that
  miner and validator can share durable second-game state, not that the UI can
  already consume it.
  Date/Author: 2026-03-29 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Game wire seam | Second game gains an ad hoc protocol unlike poker | Reuse `myosu-games::StrategyQuery` / `StrategyResponse` and bounded bincode |
| Miner generalization | CLI grows a fake `--game` flag without real implementation | Add game selection only together with an executable path |
| Validator scoring | Liar's Dice scoring becomes non-deterministic across validators | Prefer exact best-response / exploitability checks over heuristic grading |
| Harness extension | Second subnet proof quietly reuses poker-only chain assumptions | Require explicit coexistence proof for discovery, weights, and emissions |
| Harness diagnosis | `register_network` failure collapses into timeout | Make the shared chain client inclusion-aware before changing pallet logic blindly |
| Runtime diagnosis | Failing block still shows permissive rate-limit state | Treat pallet index/error interpretation and exact failing extrinsic attribution as the next proof target |

## Milestones

### Milestone 1: Liar's Dice wire contract

Expose a bounded, wire-safe query/response seam directly in
`myosu-games-liars-dice` so miner and validator can consume the second game the
same way they consume poker.

Proof command:

    cargo test -p myosu-games-liars-dice --quiet

### Milestone 2: Miner game selection

Generalize `myosu-miner` so it can train and/or answer one bounded request for
either poker or Liar's Dice without cross-game type leakage.

Proof command:

    cargo test -p myosu-miner --quiet

Status:
- Completed locally on 2026-03-29 for bounded training and one-shot wire
  serving. Live HTTP remains intentionally poker-only until Milestone 4.

### Milestone 3: Validator second-game scoring

Generalize `myosu-validator` so it can score a Liar's Dice response
deterministically from the second game's query/response contract.

Proof command:

    cargo test -p myosu-validator --quiet

Status:
- Completed locally on 2026-03-29.

### Milestone 4: Consumption surface

Add a local human/agent-consumable Liar's Dice surface so the second game's
strategy contract is not only validator-visible.

Proof command:

    cargo test -p myosu-play --quiet
    SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test --game liars-dice

Status:
- Completed locally on 2026-03-29 for local shell/pipe consumption. Chain
  miner discovery and live HTTP advice remain intentionally poker-only until
  Milestone 5 proves a real second subnet.

### Milestone 5: Two-subnet owned proof

Extend the owned stage-0 loop so poker and Liar's Dice can coexist as distinct
subnets with no game-specific chain logic.

Proof command:

    SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet

Status:
- Completed locally on 2026-03-30. `myosu-chain --stage0-local-loop-smoke`
  now proves poker subnet `2` and Liar's Dice subnet `3` coexist on the same
  local chain with distinct miner/validator flows, positive post-epoch
  incentive/dividend/emission values, and successful local gameplay surfaces
  for both games.

## Plan of Work

1. Finish the second-game wire and recommendation seam inside
   `myosu-games-liars-dice`.
2. Generalize miner and validator around executable game selection, not around
   speculative abstractions.
3. Add a second-game consumption surface.
4. Extend the owned integration harness to prove subnet coexistence.

## Validation and Acceptance

Accepted when:
- Liar's Dice exposes bounded query/response transport and recommendation helpers
- Miner and validator can both operate on a second game without poker-only imports
- A second-game surface is consumable by a human or agent
- The owned integration harness proves two subnets coexist

Current state:
- Accepted locally on 2026-03-30.

## Interfaces and Dependencies

Depends on: completed `012`, completed `011`, and the still-open remote-only
timing proof from `010`.
Blocks: the next deeper multi-game claim in `001`.
