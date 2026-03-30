# Bootstrap Chain Client, Miner, and Validator

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

Stage 0 is not complete when the chain compiles. It is complete when off-chain
participants can actually use it: a miner trains and serves strategy, a
validator scores that strategy and submits weights, and both talk to the same
chain.

After this plan, the repo will contain the first real distributed-compute loop:
`myosu-chain-client` for shared RPC/extrinsic access, `myosu-miner` for
training and serving, and `myosu-validator` for scoring and weight submission.

## Progress

- [x] (2026-03-28) Confirmed that `myosu-miner` and `myosu-validator` are still
  commented out in the workspace root.
- [x] (2026-03-28) Added `myosu-chain-client` as a shared JSON-RPC client crate
  with typed node health/method discovery plus a narrow custom-RPC wrapper for
  `neuronInfo_getNeuronsLite`.
- [x] (2026-03-28) Added the first `myosu-miner` crate slice: stage-0 CLI,
  startup probe via `myosu-chain-client`, and an operator-facing bootstrap
  report.
- [x] (2026-03-28) Extended `myosu-miner` with bounded checkpoint training plus
  a single-query wire response path, so the miner can save a profile and answer
  one real strategy query before exiting.
- [x] (2026-03-28) Added the first `myosu-validator` crate slice: stage-0 CLI,
  startup probe via `myosu-chain-client`, local checkpoint-backed scoring of a
  wire query/response artifact pair, and a stable validation report.
- [x] (2026-03-28) Added the first determinism proof for repeated validator
  scoring on the same checkpoint, query, and response artifacts.
- [x] (2026-03-28) Produced a documented local operator loop that runs a live
  devnet node plus bootstrap miner and validator passes against the same
  artifacts.
- [x] (2026-03-28) Extended `myosu-chain-client` through the first signed
  extrinsic seam so miners and validators can register on-chain, and miners
  can publish a stage-0 axon endpoint.
- [x] (2026-03-28) Extended `myosu-validator` from local artifact scoring into
  on-chain bootstrap weight submission through the live commit-reveal path.
- [x] (2026-03-28) Added subnet-owner bootstrap helpers for starting a fresh
  subnet, overriding tempo, topping validator stake, and waiting for live
  validator permit on a clean devnet subnet.
- [x] (2026-03-29) Added subnet-owner sudo control for `weights_set_rate_limit`
  and used it to complete the first clean Bob -> Alice commit-reveal weight
  submission on subnet `2`.
- [x] (2026-03-29) Fixed a fresh-subnet bootstrap bug in `myosu-chain-client`
  where absent `WeightsSetRateLimit` storage was misread as `0` instead of the
  pallet default `100`, then proved the full node-owned
  `--stage0-local-loop-smoke` path end to end.
- [x] (2026-03-29) Extended the node-owned stage-0 smoke proof beyond weight
  storage so it now waits for a real post-weight epoch outcome and confirms
  positive Alice miner incentive/emission plus positive Bob validator
  dividends on the fresh subnet.
- [x] (2026-03-29) Carried the same node-owned proof into gameplay by running
  `myosu-play` against the miner-produced checkpoint and completing one
  artifact-backed demo hand before the chain loop is declared green.
- [x] (2026-03-29) Extended `myosu-play` through the first chain-visible miner
  discovery seam and proved it from the node-owned smoke after the clean
  subnet had a real nonzero incentive winner.
- [x] (2026-03-29) Added a real stage-0 HTTP miner axon plus a live-query seam
  in `myosu-play`, then proved the node-owned smoke can discover the winning
  miner and execute one real health/strategy request against it.
- [x] (2026-03-29) Extended `myosu-play` pipe mode so it can overlay
  chain-visible live miner recommendations on each visible state while keeping
  the existing artifact-backed renderer as fallback.
- [x] (2026-03-29) Re-ran the node-owned `--stage0-local-loop-smoke` after the
  pipe-mode live recommendation change and confirmed the full local loop still
  closes cleanly.
- [x] (2026-03-29) Extended `myosu-play train` with a background live-advice
  refresh path so the TUI can show chain-visible miner recommendations without
  putting network calls in the ratatui paint loop, then re-proved the
  node-owned stage-0 smoke on the same code.
- [x] (2026-03-29) Made the interactive live-advice path surface `fresh`,
  `stale`, and `offline` states instead of silently clearing the cached hint,
  and re-proved the same node-owned stage-0 smoke afterward.
- [x] (2026-03-29) Added a lightweight age signal to the interactive live
  advice overlay so `fresh` and `stale` recommendations report how old the
  last successful miner answer is, then re-proved the same node-owned stage-0
  smoke afterward.
- [x] (2026-03-29) Routed interactive live-advice connectivity transitions
  into the shell transcript so `myosu-play train` visibly reports offline,
  stale, and recovered live-miner state changes, then re-proved the same
  node-owned stage-0 smoke afterward.
- [x] (2026-03-29) Made those interactive transcript transitions more
  operator-actionable by including the recovered miner endpoint/action and the
  first observed failure reason, then re-proved the same node-owned stage-0
  smoke afterward.
- [x] (2026-03-29) Tightened the pipe-mode state surface so each printed state
  now carries live-query truth when discovery is active: successful states
  report the live connect endpoint/action, and fallback states report the live
  query failure detail, then re-proved the same node-owned stage-0 smoke
  afterward.
- [x] (2026-03-29) Finished that pipe-mode contract by making even
  non-discovery states report `live_query=not_requested`, so agents no longer
  need to infer disabled live-query mode from a missing field, then re-proved
  the same node-owned stage-0 smoke afterward.
- [x] (2026-03-29) Expanded live-backed pipe success metadata so it now reports
  action count and recommended edge as well as connect endpoint/action,
  bringing the pipe surface closer to the smoke/live-query contract, then
  re-proved the same node-owned stage-0 smoke afterward.
- [x] (2026-03-29) Expanded that live-backed pipe success payload again so it
  now exposes both the chain-advertised miner endpoint and the normalized
  connect endpoint, then re-proved the same node-owned stage-0 smoke
  afterward.

## Surprises & Discoveries

- Observation: The game-side crates are strong enough to support miner and
  validator work once the chain is ready.
  Evidence: `myosu-games-poker` already provides solver, request, and artifact
  seams that can seed the off-chain participants.
- Observation: The live node's `rpc_methods` payload omits `version`.
  Evidence: the first miner probe failed until `myosu-chain-client` treated the
  field as optional, after which both miner and validator bootstraps succeeded
  against the same devnet.
- Observation: the pallet rejects `127.0.0.1` as an axon address even on the
  local devnet.
  Evidence: the first on-chain `serve_axon` submission timed out until the
  miner switched to the stage-0 test-safe `ip = 0` path that the pallet
  explicitly allows for testing.
- Observation: subnet `1`'s live commit-reveal window is much slower than the
  first bootstrap assumption suggested.
  Evidence: Bob's weight commit landed at block `307`, but the first legal
  reveal block was `402`, so the operator-facing validator run sat quietly
  until the reveal window opened.
- Observation: non-self weight submission is honestly blocked by validator
  permit, but self-weight remains allowed and is enough to prove the on-chain
  bootstrap seam.
  Evidence: a Bob -> Alice submission failed with `ValidatorPermitMissing`,
  while Bob -> Bob commit-reveal succeeded and left `bob_weights=[(2, 65535)]`
  in live chain storage.
- Observation: subnet `1` is no longer trustworthy as the canonical local
  bootstrap surface.
  Evidence: live inspection showed `subtoken_enabled=false` and a zero-account
  owner on subnet `1`, while owner-side bootstrap actions timed out there even
  though the earlier self-weight proof had succeeded.
- Observation: a fresh subnet restores the expected stage-0 owner/permit path.
  Evidence: subnet `2` was created live, Alice successfully enabled subtoken
  and set tempo, Bob registered, Bob added stake, and `bob_has_validator_permit`
  became `true`.
- Observation: the remaining Bob -> Alice failure is now specifically the final
  weight-control seam, not registration or permit.
  Evidence: commit-reveal submission and commit-reveal toggle attempts on
  subnet `2` still time out even after owner, tempo, stake, permit, and
  admin-freeze prerequisites look correct in storage.
- Observation: the failing Bob commit on subnet `2` was being rejected as
  `CommittingWeightsTooFast`, not because commit-reveal itself was broken.
  Evidence: decoded live block events showed `ModuleError ... error: [80, 0, 0, 0]`
  from pallet index `7`, which maps to `CommittingWeightsTooFast` in
  `pallet-game-solver`.
- Observation: once subnet `2` runs with `weights_set_rate_limit=0`, the clean
  Bob -> Alice commit-reveal path succeeds.
  Evidence: the validator later wrote `bob_weights=[(0, 65535)]` on subnet `2`
  and printed `WEIGHTS myosu-validator submission ok`.
- Observation: the fresh-subnet smoke failure was a client default-read bug,
  not a pallet inconsistency.
  Evidence: `myosu-chain-client` treated a missing `WeightsSetRateLimit`
  storage key as `0`, skipped the owner-side sudo write on fresh subnets, and
  then the first Bob commit hit the pallet's real default of `100`. After
  fixing that read, `myosu-chain --stage0-local-loop-smoke` completed with
  `bob_weights=[(0, 65535)]`.
- Observation: the clean subnet now proves chain reaction, not just chain
  acceptance.
  Evidence: after Bob's weight vector landed on subnet `2`, the node-owned
  smoke continued until it observed `alice_miner_incentive=65535`,
  `bob_validator_dividend=65535`, and positive `alice_miner_emission`.
- Observation: the gameplay seam is now part of the same executable local
  proof, not a separate manual demo.
  Evidence: `myosu-chain --stage0-local-loop-smoke` now runs
  `myosu-play --smoke-test --require-artifact` against the miner checkpoint and
  only succeeds after it reports `gameplay_advice_source=artifact` and
  `gameplay_final_state=complete`.
- Observation: best-miner discovery is truthful only after the subnet has a
  real incentive winner.
  Evidence: the first attempt to require discovery during gameplay smoke
  failed with `chain_none` on fresh subnet `2`; moving the gameplay smoke to
  the post-epoch phase then produced `gameplay_discovered_miner_uid=0` and
  `gameplay_discovered_miner_endpoint=0.0.0.0:8080`.
- Observation: the first HTTP axon readiness probe killed the server before
  gameplay could use it.
  Evidence: the node-owned smoke initially failed with `Connection refused`
  because the TCP readiness check opened and closed a bare socket, the axon
  treated the resulting broken-pipe response write as fatal, and the process
  exited. Downgrading response-write failures to warnings kept the server alive
  for the real gameplay query.
- Observation: the new live recommendation overlay in `myosu-play` pipe mode
  did not weaken the executable stage-0 proof.
  Evidence: after wiring per-state live recommendation refresh into pipe mode,
  `myosu-chain --stage0-local-loop-smoke` still completed with the same fresh
  subnet weight, gameplay, and post-epoch economic outputs.
- Observation: the interactive TUI can consume live miner advice without
  pushing HTTP work into the render loop.
  Evidence: `myosu-play train` now refreshes a cached live recommendation in a
  background task against the discovered miner, while the same code still
  passes focused tests/clippy and the full node-owned stage-0 smoke.
- Observation: cached live advice is more truthful when refresh failures mark
  it stale instead of erasing it.
  Evidence: the renderer now keeps the last good live recommendation visible as
  `[STALE]`, shows `LIVE ADVICE OFFLINE` before any recommendation is available,
  and the node-owned stage-0 smoke still closes with the same miner discovery,
  gameplay, and post-epoch outputs.
- Observation: freshness status is clearer when it carries a small age signal.
  Evidence: the interactive renderer now shows `FRESH 0s` on successful live
  refresh and a growing stale age after refresh failures, while focused checks
  and the node-owned stage-0 smoke still pass unchanged.
- Observation: connectivity changes in interactive live advice are easier to
  trust when they are logged as transcript events, not just shown in the state
  panel.
  Evidence: `myosu-play train` now emits shell messages when live advice first
  goes offline, recovers, or becomes stale after a prior success; focused unit
  tests cover those transition messages, and the node-owned stage-0 smoke
  still completes unchanged.
- Observation: transcripted live-advice transitions are more useful when they
  carry real operator detail instead of generic state names.
  Evidence: recovery messages now include the discovered miner connect endpoint
  plus the returned action, while offline/stale messages carry the first live
  failure detail that triggered the transition; focused checks and the
  node-owned stage-0 smoke both stayed green.
- Observation: pipe mode is more truthful for agents when each state line says
  whether the live query succeeded or fell back.
  Evidence: live-backed pipe states now append `live_query=live_http` with the
  connect endpoint and action, while fallback states append
  `live_query=live_failed` plus the failure detail; focused checks and the
  node-owned stage-0 smoke both stayed green.
- Observation: pipe-mode live-query metadata is more reliable when it is
  always explicit, even when live querying was never requested.
  Evidence: non-discovery pipe states now append `live_query=not_requested`
  instead of omitting the field entirely; focused checks and the node-owned
  stage-0 smoke both stayed green.
- Observation: pipe-mode live success states are more useful when they carry
  the same core solver payload as the smoke proof, not just the final action.
  Evidence: live-backed pipe states now append `live_miner_action_count` and
  `live_miner_recommended_edge` in addition to the connect endpoint and final
  action; focused checks and the node-owned stage-0 smoke both stayed green.
- Observation: pipe-mode live success states are more truthful when they show
  both the chain-visible and locally connectable miner endpoints.
  Evidence: live-backed pipe states now append
  `live_miner_advertised_endpoint=0.0.0.0:8080` and
  `live_miner_connect_endpoint=127.0.0.1:8080`; focused checks and the
  node-owned stage-0 smoke both stayed green.

## Decision Log

- Decision: `myosu-chain-client` comes first and both participants go through it.
  Rationale: The miner and validator should not each invent their own chain
  integration logic.
  Date/Author: 2026-03-28 / Codex

- Decision: The miner publishes snapshots and the validator proves
  determinism against pinned artifacts.
  Rationale: `OS.md` and `AGENTS.md` make deterministic scoring a stage-0
  requirement.
  Date/Author: 2026-03-28 / Codex

- Decision: the stage-0 local miner publishes `ip = 0` instead of
  `127.0.0.1`.
  Rationale: the pallet rejects localhost addresses but explicitly allows zero
  IP for testing, which keeps the local on-chain proof honest without
  pretending the devnet has routable networking.
  Date/Author: 2026-03-28 / Codex

- Decision: the first validator on-chain proof uses self-weight through
  commit-reveal instead of pretending miner-targeted weights are already live.
  Rationale: self-weight is valid without validator permit and proves the real
  extrinsic/storage path now, while non-self weighting should wait for the
  permit/emission seam to be truthful.
  Date/Author: 2026-03-28 / Codex

- Decision: current operator proof should move from stale subnet `1` to fresh
  subnet `2`.
  Rationale: subnet `1` still carries enough inherited/stale bootstrap state
  that it is no longer the right source of truth for current stage-0 bring-up.
  Date/Author: 2026-03-28 / Codex

- Decision: the fresh-subnet local playbook explicitly sets
  `weights_set_rate_limit=0`.
  Rationale: stage-0 bootstrap needs immediate post-registration weighting,
  while the subnet's carried rate limit otherwise rejects the first Bob commit
  as `CommittingWeightsTooFast`.
  Date/Author: 2026-03-29 / Codex

## Outcomes & Retrospective

The first shared off-chain seam now exists. `myosu-chain-client` is a real
workspace crate, not a placeholder, and it compiles lint-clean. The surface is
still intentionally narrow, but it is no longer read-only: it now covers
WebSocket connection setup, typed node-health discovery, the custom
`neuronInfo_getNeuronsLite` RPC, storage reads for subnet membership and axon
state, signed extrinsic submission for burned registration, `serve_axon`,
stake top-ups, subnet-owner bootstrap, validator weight paths, and post-epoch
reads for incentive/dividend/emission state. That is enough to prove the first
real chain-participation seam and give
miner/validator code one place to start from. The miner has moved past
probe-only startup too: `myosu-miner` exists as a real workspace member,
parses the intended stage-0 operator flags, probes the node through
`myosu-chain-client`, can ensure it is registered on-chain, can publish a
devnet-safe axon endpoint, runs a bounded training batch against an
encoder/checkpoint set, and can answer one wire-encoded poker strategy query to
a saved response artifact before exiting. That is still intentionally narrower
than a full HTTP axon, but it gives the repo a real local inference seam tied
to live chain state. The validator slice now does the same at the scoring edge:
`myosu-validator` is a real workspace member, consumes the same shared chain
client, can ensure validator registration on-chain, can bootstrap subnet owner
state on a fresh subnet, can top up stake and wait for validator permit, loads
the same checkpoint and encoder artifacts, decodes one wire query plus one
miner response, derives the validator's expected answer locally, and scores the
miner response against that expectation with a stable deterministic report. The
earlier subnet-`1` self-weight proof remains useful as evidence that the
commit-reveal path can work, but current operator truth has moved to a fresh
subnet because subnet `1` now reads back as stale bootstrap state. On subnet
`2`, the repo now proves clean owner control, clean validator registration,
live permit acquisition, and a real Bob -> Alice commit-reveal write on the
clean subnet once `weights_set_rate_limit=0` is applied. The participant
bootstrap seam this plan was meant to prove is now real. The newest proof is
stronger than the manual subnet-`2` run too: `myosu-chain --stage0-local-loop-smoke`
now owns the entire local bootstrap loop, and it only went green once the
shared client stopped lying about fresh-subnet rate-limit defaults. It now also
waits for the next local epoch outcome and confirms that the committed weights
propagate into positive miner incentive/emission and validator dividends,
which is the first honest proof that the chain is reacting to the participant
loop instead of only storing it. The newest addition carries that same local
proof one layer upward into the product surface: the node-owned stage-0 loop
now starts a real checkpoint-backed HTTP miner axon, runs `myosu-play` against
the miner-produced artifacts, discovers the live winning miner from chain
state, and completes one
artifact-backed hand before the local devnet is considered green. That gameplay
proof now also asks the chain who the best visible miner is, but only after the
same clean subnet has produced a nonzero incentive vector, which keeps miner
selection honest instead of fabricating a winner before the first scored epoch.

## Context and Orientation

The future crates are:

- `crates/myosu-chain-client/`
- `crates/myosu-miner/`
- `crates/myosu-validator/`

The chain client speaks to the node from plan 004. The miner depends on the
game crates and serves strategy over HTTP. The validator depends on the same
artifacts, queries the miner, computes exploitability, and submits weights.

## Milestones

### Milestone 1: Shared chain client

Create a typed client for the node's RPC and extrinsic submission surface.

Proof command:

    cargo check -p myosu-chain-client

Current status: initial RPC client landed and passes `cargo check`, `cargo
test`, and `cargo clippy`. It now also proves signed burned registration, axon
publication, and weight commit/reveal against the live devnet.

### Milestone 2: Miner binary

Create the miner with training, checkpoint, and serving behavior.

Proof commands:

    cargo check -p myosu-miner
    cargo test -p myosu-miner --quiet

Current status: `cargo run -p myosu-miner -- --help` works, the startup path
probes the chain through `myosu-chain-client`, bounded checkpoint training
passes, the miner can ensure on-chain registration and publish an axon, and it
can answer one wire query to a wire response artifact.

### Milestone 3: Validator binary

Create the validator with scoring and weight submission behavior.

Proof commands:

    cargo check -p myosu-validator
    cargo test -p myosu-validator --quiet

Current status: `cargo run -p myosu-validator -- --help` works, the startup
path probes the chain through `myosu-chain-client`, local artifact scoring
passes, the validator can ensure on-chain registration, and
`cargo test -p myosu-validator inv_003_determinism --quiet` proves repeated
scoring is stable on fixed inputs. The validator can also bootstrap subnet
owner state on a fresh subnet, top up stake, wait for permit, and submit one
historical bootstrap self-weight through commit-reveal on the stale subnet-`1`
path. It can now also submit a clean Bob -> Alice commit-reveal weight on
subnet `2` once the subnet's weights-set rate limit is overridden to `0`.

### Milestone 4: Determinism proof

Prove the validator scores the same miner the same way when artifact and query
inputs are fixed.

Proof command:

    cargo test -p myosu-validator inv_003_determinism --quiet

### Milestone 5: Local operator loop

Prove that one machine can run the devnet node, generate bootstrap artifacts,
run the miner, and run the validator against the same files while proving one
real on-chain weight write.

Proof commands:

    cargo run -p myosu-games-poker --example bootstrap_artifacts -- /tmp/myosu-bootstrap-encoder /tmp/myosu-bootstrap-query.bin
    cargo run -p myosu-miner -- --subnet 2 --key //Alice --encoder-dir /tmp/myosu-bootstrap-encoder --query-file /tmp/myosu-bootstrap-query.bin --response-file /tmp/myosu-bootstrap-response.bin --data-dir /tmp/myosu-bootstrap-miner
    cargo run -p myosu-validator -- --subnet 2 --key //Alice --enable-subtoken --sudo-tempo 2 --sudo-weights-rate-limit 0
    cargo run -p myosu-validator -- --subnet 2 --key //Bob --register --stake-amount 100000000000000 --submit-weights --weight-hotkey //Alice --encoder-dir /tmp/myosu-bootstrap-encoder --checkpoint /tmp/myosu-bootstrap-miner/checkpoints/latest.bin --query-file /tmp/myosu-bootstrap-query.bin --response-file /tmp/myosu-bootstrap-response.bin

Current status: the live devnet now proves a clean subnet-`2` bootstrap loop.
Subnet `2` can be created, Alice can enable subtoken, set tempo, and set
`weights_set_rate_limit=0`, Bob can register and reach validator permit, the
validator reports an exact-match score of `1.000000` on the shared artifacts,
and the Bob -> Alice commit-reveal loop completes with
`bob_weights=[(0, 65535)]` on subnet `2`.

## Plan of Work

Bring up the shared chain client first, then wire the miner and validator
through it. Keep the first loop local and narrow: one node, one miner, one
validator, one subnet, one set of pinned artifacts, one reproducible scoring
path.

## Concrete Steps

From `/home/r/coding/myosu`:

    nl -ba Cargo.toml | sed -n '1,24p'
    cargo check -p myosu-chain-runtime
    cargo check -p myosu-chain

After the crates exist:

    cargo check -p myosu-chain-client -p myosu-miner -p myosu-validator
    cargo test -p myosu-chain-client --quiet

## Validation and Acceptance

This plan is complete when:

- All three crates compile.
- The miner can train and serve strategy.
- The validator can score a miner and submit at least one honest bootstrap
  weight vector on-chain.
- The determinism proof passes.
- A documented local loop exists for node, miner, and validator together.

Current truthful edge: the plan has achieved local scoring, on-chain
registration, fresh-subnet permit bootstrap, one historical self-weight proof,
and one clean Bob -> Alice miner-targeted weight on the fresh subnet.

## Idempotence and Recovery

Add the three crates incrementally. If one participant blocks, keep the others
building and tested; do not collapse them into one giant implementation step.

## Interfaces and Dependencies

Depends on: plans 003, 004, and 005.
Blocks: integration harness and stage-0 exit proof.
