# Reduce Pallet Game-Solver to the Stage-0 Surface

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The pallet is the heart of the chain. It should own subnet management,
registration, staking, weight submission, and Yuma-driven emissions. Right now
the live `pallet-game-solver` still carries old subtensor baggage: CRV3
timelock paths, crowdloan-era leasing surfaces, and swap-heavy staking logic
that do not belong in the stage-0 product.

After this plan, the pallet will expose the smaller stage-0 surface described in
`OS.md`: subnets, neurons, serving, stake, weights, commit-reveal v2, and Yuma.

## Progress

- [x] (2026-03-28) Confirmed that `pallet-game-solver` is the live pallet in
  the runtime.
- [x] (2026-03-28) Confirmed that the pallet Config trait has already dropped
  the old drand/crowdloan supertrait.
- [x] (2026-03-28) Confirmed that CRV3 timelock, drand-based reveal logic, and
  swap-heavy staking paths still exist inside the pallet code and tests.
- [x] (2026-03-28) Removed the live CRV3/timelocked weight extrinsics from the
  pallet call surface.
- [x] (2026-03-29) Kept commit-reveal v2 as the only live weight-hiding
  mechanism and added a focused stage-0 proof that direct `set_weights` is
  blocked while commit-reveal is enabled, then the surviving v2
  `commit_weights` / `reveal_weights` path opens and sets the live weights in
  the valid reveal window.
- [x] (2026-03-29) Replaced the pallet's full inherited swap-handler
  dependency with a pallet-local stage-0 swap seam that only carries the live
  TAO<->alpha swap directions plus the surviving price and fee helpers, while
  keeping the focused stage-0 proof, stripped runtime build, and warning gates
  green.
- [x] (2026-03-28) Removed the crowdloan/leasing extrinsics from the live
  pallet call surface.
- [x] (2026-03-28) Proved the stage-0 flow with a focused pallet test:
  create subnet, register, stake, serve axon, submit weights, advance epoch,
  and observe nonzero miner incentive plus validator dividends.
- [x] (2026-03-28) Deleted the dead timelocked/CRV3 commit implementation and
  legacy timelocked commit events from the live pallet logic while keeping
  storage cleanup for historical state removal.
- [x] (2026-03-28) Collapsed the leasing module down to legacy storage types
  only and removed lease-only events from the live pallet event surface.
- [x] (2026-03-28) Removed dead lease/timelock benchmark entries, stale
  lease/timelock error variants, and the commented CRV3 dispatch stub from the
  live pallet source.
- [x] (2026-03-28) Replaced the dead mutable commit-reveal version storage and
  event with a fixed stage-0 constant so the pallet stops carrying an unused
  runtime knob.
- [x] (2026-03-28) Moved the deprecated CRV3 commit storages out of the
  default stage-0 build behind `legacy-subtensor-tests`, keeping them only for
  explicit archival test runs.
- [x] (2026-03-28) Moved `TimelockedWeightCommits` out of the default stage-0
  build as well, so the live pallet now keeps only the hash-based
  `WeightCommits` commit-reveal storage.
- [x] (2026-03-28) Moved the remaining lease-era storage maps out of the
  default stage-0 build behind `legacy-subtensor-tests`, keeping lease state as
  archive-only by default.
- [x] (2026-03-29) Collapsed the coldkey stake-summary helpers down to direct
  price-based valuation instead of AMM-style `sim_swap` calls, matching the
  stage-0 noop swap contract the runtime already provides.
- [x] (2026-03-29) Added a focused stage-0 regression proving those coldkey
  stake summaries now match direct alpha-price conversion in the pallet test
  runtime.
- [x] (2026-03-29) Extended that same stage-0 regression to cover the hotkey
  aggregate summary too, so the full surviving summary surface is pinned to the
  direct price model rather than swap simulation.
- [x] (2026-03-29) Added a focused stage-0 regression proving the stage-0
  `StakeInfo` payload can drop the carried swap-era `locked`, `tao_emission`,
  and `drain` fields entirely without breaking the active pallet/runtime path.
- [x] (2026-03-29) Added direct runtime tests proving the real stage-0
  `Stage0NoopSwap` contract is identity conversion, zero fee, unit price, and
  zero protocol TAO, instead of relying on the broader pallet mock runtime to
  stand in for that claim.
- [x] (2026-03-29) Extended that runtime proof to the swap runtime API wrapper,
  so `current_alpha_price` and both `sim_swap_*` methods are now pinned to the
  same stage-0 identity contract they expose outward.
- [x] (2026-03-29) Added a direct runtime test proving the real stage-0
  `get_stake_fee` surface is zero too, so the outward fee contract is now
  pinned on the stripped runtime rather than only inferred from `Stage0NoopSwap`.
- [x] (2026-03-29) Routed the runtime `StakeInfoRuntimeApi::get_stake_fee`
  wrapper through the same tiny helper the runtime proof uses, so the outward
  API surface and the test now share one stage-0 code path.
- [x] (2026-03-29) Shrunk the stage-0 `DynamicInfo` payload too by dropping the
  dead zero-only `emission` and `pending_root_emission` fields, with a focused
  regression that proves the smaller payload still mirrors live storage.
- [x] (2026-03-29) Shrunk the stage-0 `Metagraph` and `SelectiveMetagraph`
  payloads by removing the deprecated zero-only `subnet_emission` and
  `pending_root_emission` fields, while retiring those selective index slots
  without renumbering later indexes.
- [x] (2026-03-29) Shrunk `SubnetInfo` and `SubnetInfov2` by removing the
  dead zero-only `network_modality`, `network_connect`, and
  `emission_value[s]` fields, with a focused stage-0 regression proving the
  smaller payloads still mirror the live getters.
- [x] (2026-03-29) Removed the zero-only `tao_dividends_per_hotkey` payload
  from `Metagraph` and `SelectiveMetagraph`, while retiring selective index
  `70` in place so later metagraph indexes keep their meaning.
- [x] (2026-03-29) Removed the old v1 `SubnetInfo` runtime/RPC path entirely,
  leaving `SubnetInfov2` as the only live subnet-info interface and updating
  the focused stage-0 proof to validate that single surviving contract.
- [x] (2026-03-29) Removed the old v1 `SubnetHyperparams` runtime/RPC path
  entirely, leaving `SubnetHyperparamsV2` as the only live subnet-hyperparams
  interface and adding a focused stage-0 proof for the surviving v2 payload.
- [x] (2026-03-29) Shrunk the active `SubnetState` payload by removing the
  inherited `emission_history` field, which had been bundling cross-subnet
  history into a single-subnet response, and added a focused stage-0 proof for
  the smaller state surface.
- [x] (2026-03-29) Tightened `get_all_dynamic_info` from `Vec<Option<...>>` to
  `Vec<DynamicInfo>`, so the live surface now omits dead subnet entries instead
  of encoding them as `None`, and added a focused stage-0 proof for that
  filtered contract.
- [x] (2026-03-29) Tightened `get_all_metagraphs` from `Vec<Option<...>>` to
  `Vec<Metagraph<_>>`, so the live surface now omits dead subnet entries
  instead of encoding them as `None`, and added a focused stage-0 proof for
  that filtered aggregate contract.
- [x] (2026-03-29) Tightened `get_subnets_info_v2` from `Vec<Option<...>>` to
  `Vec<SubnetInfov2<_>>`, so the live surface now omits dead subnet entries
  instead of encoding them as `None`, and added a focused stage-0 proof for
  that filtered aggregate contract.
- [x] (2026-03-29) Tightened `get_all_mechagraphs` from `Vec<Option<...>>` to
  `Vec<Metagraph<_>>`, so the live surface now omits dead entries instead of
  encoding them as `None`, and added a focused stage-0 proof for that
  filtered mechanism aggregate contract.
- [x] (2026-03-29) Corrected the surviving neuron reporting contract so the
  `stake` map now reflects real per-coldkey subnet stake entries instead of
  attributing the hotkey's total stake to the owner coldkey alone, and added a
  focused stage-0 proof covering delegated stake on a single neuron.
- [x] (2026-03-29) Corrected the surviving delegate reporting contract so
  nominator stake survives sparse live-netuid layouts instead of being dropped
  behind dead subnet holes, and added a focused stage-0 proof for that sparse
  delegate path.
- [x] (2026-03-29) Corrected the surviving delegate economic reporting
  contract so `return_per_1000` divides by total hotkey stake instead of
  root-only stake, and added a focused stage-0 proof covering a delegate with
  live stake and emissions only on a non-root subnet.
- [x] (2026-03-29) Corrected the surviving delegate economic reporting
  contract so `total_daily_return` applies delegate take instead of exposing
  pre-take emission as if it were delegator return, and added a focused
  stage-0 proof covering a nonzero-take delegate.
- [x] (2026-03-29) Shrunk the custom runtime/RPC surface by removing the
  unused `get_coldkey_auto_stake_hotkey` and `get_subnet_to_prune` endpoints,
  plus the now-unreferenced pallet getter, so stage 0 no longer advertises
  helper methods that are outside the active genesis-plan contract.
- [x] (2026-03-29) Shrunk the custom runtime/RPC surface again by removing the
  unused `get_subnet_state`, `get_selective_metagraph`, and
  `get_selective_mechagraph` endpoints, so stage 0 no longer exports
  inspection helpers with no remaining doctrine, spec, client, or in-repo
  consumer outside the compatibility shell.
- [x] (2026-03-29) Shrunk the custom runtime/RPC surface again by removing the
  unused delegate RPC trio (`get_delegates`, `get_delegate`, `get_delegated`),
  so stage 0 no longer exports delegate inspection endpoints with no remaining
  doctrine, spec, chain-client, or in-repo consumer outside the compatibility
  shell.
- [x] (2026-03-29) Collapsed the remaining custom runtime/RPC surface down to
  the single `neuronInfo_getNeuronsLite` path that the checked-in bootstrap
  plans and shared chain client still use, removing the rest of the inherited
  neuron, subnet-info, metagraph, dynamic-info, mechagraph, stake-info, and
  lock-cost runtime/RPC contract.
- [x] (2026-03-29) Collapsed the active pallet-side neuron reporting helpers
  down to the same surviving lite-list contract, removing the inherited full
  neuron and single-item getters so the live pallet surface no longer carries a
  broader compatibility shell than the runtime and shared client expose.
- [x] (2026-03-29) Removed the unused aggregate pallet reporting helpers
  `get_delegates`, `get_delegated`, and `get_stake_info_for_coldkeys`, so the
  active stage-0 pallet surface no longer advertises batch reporting paths with
  no remaining runtime, client, or focused-proof consumer.
- [x] (2026-03-29) Shrunk `SubnetHyperparamsV2` by removing the dead
  `user_liquidity_enabled` field, which could only ever report `false` under
  the stripped stage-0 runtime swap seam, and kept the focused stage-0 proof
  green with the smaller payload.
- [x] (2026-03-29) Removed the pallet-side no-op liquidity adjustment and
  cleanup calls from coinbase emission and subnet dissolution, plus the dead
  `get_protocol_tao` wrapper, so the live stage-0 pallet no longer routes work
  through swap-interface hooks that the stripped runtime only implements as
  inert placeholders.

## Surprises & Discoveries

- Observation: The pallet has already moved farther than the older audit
  assumed.
  Evidence: `src/macros/config.rs` now depends only on `frame_system::Config`.

- Observation: The remaining baggage is internal surface area, not the top-level
  Config trait.
  Evidence: `src/subnets/weights.rs` and `src/macros/dispatches.rs` still carry
  `commit_timelocked_*` paths, while staking code still routes through
  `SwapInterface`.

- Observation: The fastest truthful reduction is at the public pallet API
  boundary, not the deepest storage layer.
  Evidence: deleting the four non-stage-0 extrinsics immediately shrank the
  metadata/call surface while leaving internal storage, migrations, and tests
  available for a follow-on cleanup slice.

- Observation: The carried subtensor unit-test corpus is no longer a truthful
  default proof surface for stage 0.
  Evidence: `cargo test -p pallet-game-solver stage_0_flow --quiet` initially
  failed before even reaching the new proof because dozens of historical test
  modules pulled in stripped migrations, removed CRV3 paths, and undeclared
  dev-only dependencies.

- Observation: some of the remaining swap-era residue is now only valuation
  ceremony, not live staking behavior.
  Evidence: the runtime's `Stage0NoopSwap` already reports identity pricing and
  zero fees, while `staking/helpers.rs` was still calling `sim_swap` for
  coldkey stake summaries until this slice reduced those paths to direct price
  conversion.

- Observation: the last real staking dependency was smaller than the inherited
  swap contract implied.
  Evidence: once the dead liquidity hooks were removed, the surviving pallet
  only needed two concrete swap directions plus price and fee helpers; a
  pallet-local `Stage0SwapInterface<Self>` now carries exactly that smaller
  seam while `stage_0_flow`, stripped runtime `cargo check`, and both clippy
  gates remain green.

- Observation: the stage-0 stake-reporting payload was small enough to shrink
  for real instead of remaining zeroed baggage.
  Evidence: `src/rpc_info/stake_info.rs` no longer carries `locked`,
  `tao_emission`, or `drain`, and the focused stage-0 regression plus runtime
  compile path both stay green with the smaller payload.

- Observation: the default pallet test runtime is not the same proof surface as
  the stripped stage-0 runtime.
  Evidence: `tests/mock.rs` still wires `SwapInterface` to the full swap pallet,
  while `runtime/src/lib.rs` uses `Stage0NoopSwap`; the new runtime tests now
  prove the zero-fee identity contract directly where it actually lives.

- Observation: the runtime/client neuron contract had already converged on the
  lite-list shape, while the pallet still carried heavier full and single-item
  reporting helpers behind it.
  Evidence: the only surviving custom node RPC is `neuronInfo_getNeuronsLite`,
  the shared chain client only calls that method, and the focused stage-0 proof
  still worked once it decoded the list payload directly instead of going
  through a private single-item pallet getter.

- Observation: some pallet-side reporting residue was now dead even by the
  narrower stage-0 proof standard.
  Evidence: `get_delegates`, `get_delegated`, and
  `get_stake_info_for_coldkeys` had no remaining runtime, chain-client, or
  active stage-0 test consumer; removing them left the focused proofs green and
  the runtime compile path unchanged.

- Observation: the surviving commit-reveal surface was already stage-0-shaped,
  but it still needed a truthful focused proof on the slim path.
  Evidence: the live pallet already blocked direct `set_weights` behind
  `CommitRevealEnabled` and only exposed the hash-based
  `commit_weights` / `reveal_weights` flow, but `stage_0_flow` had only been
  proving the direct non-hiding path until this slice added a commit/reveal v2
  proof.

- Observation: some of the remaining swap-era residue has now shrunk to
  impossible outward reporting flags rather than live economic behavior.
  Evidence: `SubnetHyperparamsV2` was still carrying
  `user_liquidity_enabled`, but the stripped runtime's swap seam can only
  return `false` there. Removing the field kept the focused stage-0 proof and
  runtime compile path green.

- Observation: some of the remaining swap-era staking dependence has now
  shrunk to dead plumbing rather than active economics.
  Evidence: coinbase emission and subnet dissolution were still calling
  `adjust_protocol_liquidity`, `dissolve_all_liquidity_providers`, and
  `clear_protocol_liquidity`, but the stripped runtime implements those hooks
  as inert no-ops. Removing those calls and the dead `get_protocol_tao`
  wrapper left the focused stage-0 flow, runtime check, and clippy proof set
  green.

- Observation: the outward runtime API already preserves the same stage-0 swap
  simplification instead of reintroducing AMM semantics at the RPC boundary.
  Evidence: `runtime/src/lib.rs` maps `current_alpha_price` to scaled unit
  price and both `sim_swap_*` calls to `Stage0NoopSwap::sim_swap`; the new
  runtime test now proves those outward results directly.

- Observation: the real stage-0 runtime fee surface has now been proven, not
  just reasoned about from lower layers.
  Evidence: `runtime/src/lib.rs` still forwards `get_stake_fee` through the
  pallet wrapper, and the new runtime test proves representative fee queries
  return zero under the stripped runtime's noop swap contract.

- Observation: the runtime fee proof now matches the actual API wrapper path,
  not just the pallet method underneath it.
  Evidence: `runtime/src/lib.rs` now routes `StakeInfoRuntimeApi::get_stake_fee`
  through the same helper used by the runtime test.

- Observation: `DynamicInfo` also had removable stage-0 baggage, but the honest
  invariant was storage coherence rather than symmetric reserve assumptions.
  Evidence: `src/rpc_info/dynamic_info.rs` no longer carries `emission` or
  `pending_root_emission`, and the focused stage-0 regression now decodes the
  smaller payload and checks the surviving fields against live storage values.

- Observation: the metagraph reporting surface still carried explicit
  deprecated placeholders beyond `DynamicInfo`.
  Evidence: `src/rpc_info/metagraph.rs` had been hardcoding
  `subnet_emission = 0` and `pending_root_emission = 0` in both the full and
  selective payload builders until this slice removed them and retired indexes
  `11` and `19` without shifting the later selective index map.

- Observation: the subnet-info reporting surface had a smaller but even more
  obvious version of the same problem.
  Evidence: `src/rpc_info/subnet_info.rs` was still hardcoding
  `network_modality = 0`, `network_connect = []`, and `emission_value[s] = 0`
  in both `SubnetInfo` builders until this slice removed those fields and
  proved the smaller payloads in `stage_0_flow`.

- Observation: the metagraph surface still had one more zero-only reporting
  branch even after the earlier deprecated-emission cleanup.
  Evidence: `src/rpc_info/metagraph.rs` was still building
  `tao_dividends_per_hotkey` as all-zero values and exposing it through
  selective index `70` until this slice removed that payload and retired the
  slot in place.

- Observation: keeping both subnet-info versions alive had become needless
  compatibility drag rather than truthful stage-0 surface.
  Evidence: the active runtime API, runtime impl, RPC layer, and focused
  `stage_0_flow` proof were all able to move cleanly to `SubnetInfov2` alone
  once the old v1 `SubnetInfo` path was removed.

- Observation: the same compatibility drag still existed one layer over in
  subnet hyperparams.
  Evidence: the active pallet path, runtime API, runtime impl, and RPC layer
  were still carrying both `SubnetHyperparams` and `SubnetHyperparamsV2` even
  though only the v2 surface was needed for the truthful surviving contract.

- Observation: `SubnetState` was still carrying one inherited field whose
  semantics no longer matched the surrounding payload.
  Evidence: `src/rpc_info/show_subnet.rs` was embedding `emission_history`
  gathered across all subnets inside a response that is otherwise scoped to a
  single queried subnet.

- Observation: the same “carry dead entries forward” habit still existed in one
  aggregate runtime surface.
  Evidence: `get_all_dynamic_info` was iterating all stored subnet keys and
  returning `Vec<Option<DynamicInfo>>`, which preserved dead `NetworksAdded =
  false` entries as `None` rather than exposing only live dynamic payloads.

- Observation: the metagraph aggregate surface was still preserving dead subnet
  keys as outward holes too.
  Evidence: `get_all_metagraphs` was iterating all stored subnet keys and
  returning `Vec<Option<Metagraph<_>>>`, which preserved dead `NetworksAdded =
  false` entries as `None` instead of exposing only live metagraph payloads.

- Observation: the surviving subnet-info aggregate path was still preserving
  dead subnet keys as outward holes too.
  Evidence: `get_subnets_info_v2` was iterating stored subnet keys and
  returning `Vec<Option<SubnetInfov2<_>>>`, which preserved dead
  `NetworksAdded = false` entries as `None` instead of exposing only live
  subnet-info payloads.

- Observation: the surviving mechanism aggregate path was still preserving dead
  entries as outward holes too.
  Evidence: `get_all_mechagraphs` was iterating only valid subnet/mechanism
  pairs but still returning `Vec<Option<Metagraph<_>>>`, which preserved a
  compatibility wrapper even though the aggregate builder only walks live
  mechanism IDs.

- Observation: the surviving neuron reporting path still had one semantic lie
  even after the aggregate surfaces were tightened.
  Evidence: `src/rpc_info/neuron_info.rs` documented `stake` as a coldkey map
  including delegations, but the active builder was emitting a single row for
  the owner coldkey with the hotkey's total subnet stake.

- Observation: the surviving delegate reporting path still had one sparse-key
  bug even after the aggregate surfaces were tightened.
  Evidence: `src/rpc_info/delegate_info.rs` was indexing live share pools by
  raw `netuid` into a dense vector, so a dead lower subnet key could make live
  nominator stake disappear from the outward delegate report.

- Observation: the surviving delegate reporting path still had one economic
  truth bug even after the sparse-netuid fix.
  Evidence: `src/rpc_info/delegate_info.rs` was computing
  `return_per_1000` with root-only stake while outward daily return was
  aggregating emissions across the delegate's registered subnets.

- Observation: the surviving delegate reporting path still had one more
  economic truth bug even after the denominator fix.
  Evidence: `src/rpc_info/delegate_info.rs` documented
  `total_daily_return` as delegator return, but the active builder was still
  exposing pre-take emission before subtracting delegate commission.

- Observation: the custom runtime/RPC surface was still carrying helper
  endpoints that the active stage-0 doctrine never names or uses.
  Evidence: `get_coldkey_auto_stake_hotkey` and `get_subnet_to_prune` were
  only wired through `runtime-api`, `runtime`, and `rpc`, with no remaining
  plan, spec, or in-repo consumer requiring them.

- Observation: the same compatibility shell still had a second layer of
  outward-only inspection helpers after the first RPC cut.
  Evidence: `get_subnet_state`, `get_selective_metagraph`, and
  `get_selective_mechagraph` were still exported through `runtime-api`,
  `runtime`, and `rpc`, but had no remaining plan, spec, chain-client, or
  other in-repo consumer beyond that outward wiring.

- Observation: the custom runtime/RPC shell still had a delegate-reporting
  layer even after the inspection-helper cuts.
  Evidence: `get_delegates`, `get_delegate`, and `get_delegated` were still
  exported through `runtime-api`, `runtime`, and `rpc`, but had no remaining
  plan, spec, chain-client, or other in-repo consumer beyond that outward
  compatibility surface.

- Observation: after the delegate cut, the checked-in doctrine still named
  only one custom node RPC.
  Evidence: the genesis plans and `myosu-chain-client` still referenced only
  `neuronInfo_getNeuronsLite`, while the remaining custom neuron/subnet/
  metagraph/stake runtime-RPC methods had no surviving plan, spec, or client
  consumer.

## Decision Log

- Decision: Commit-reveal v2 is the only weight-hiding path we keep for stage 0.
  Rationale: `AGENTS.md` explicitly says CRV3 timelock depends on Drand and is
  not part of the intended stage-0 chain.
  Date/Author: 2026-03-28 / Codex

- Decision: Pallet reduction is behavioral, not cosmetic.
  Rationale: The goal is not to relabel a broad subtensor pallet as
  "game-solver"; the goal is to shrink the live extrinsic and state surface to
  what stage 0 really uses.
  Date/Author: 2026-03-28 / Codex

## Outcomes & Retrospective

The first pallet reduction slice is complete. The live call surface no longer
offers `register_leased_network`, `terminate_lease`,
`commit_timelocked_weights`, `commit_timelocked_mechanism_weights`, or
`commit_crv3_mechanism_weights`, and the dead CRV3 event variants were removed
with them. There is now also a real `stage_0_flow` pallet proof that exercises
subnet creation, registration, stake, serving, weights, epoch timing, and
reward computation. The live timelocked commit code path is now gone as well,
and the live leasing behavior is gone too. The pallet also no longer carries an
unused mutable commit-reveal version storage item or event; stage 0 just uses
the fixed version required by the surviving commit-reveal v2 path. The
deprecated CRV3 storage maps are now also out of the default pallet build and
only retained behind `legacy-subtensor-tests` for archival coverage. The same
is now true of `TimelockedWeightCommits`, which means the default stage-0 build
keeps only the live hash-based `WeightCommits` path. The same archive-only
boundary now covers the old lease maps too. The remaining honest gap is now
mostly archive-only migration baggage and a few legacy type definitions, not
active timelocked/CRV3/leasing behavior. The historical subtensor unit corpus
is now gated behind the explicit `legacy-subtensor-tests` feature so it stops
blocking the stage-0 pallet proof path by default.

The next reduction is now in as well. The coldkey stake-summary helpers no
longer pretend stage 0 has a live AMM by routing through `sim_swap`; they now
use the direct alpha-price conversion that the noop runtime swap already
defines. That keeps the surviving staking helper surface aligned with the
runtime's actual stage-0 contract instead of preserving extra swap machinery
purely as inherited ceremony.

That helper reduction now has its own stage-0 proof too. The focused
`stage_0_flow` surface no longer only proves subnet creation, registration,
stake, serving, weights, and emissions; it also proves that the coldkey stake
summary helpers agree with direct alpha-price valuation in the pallet test
runtime. That makes the simplification executable rather than just a local code
cleanup.

The same proof now reaches the hotkey aggregate as well. That matters because
the earlier failed regression showed that "raw stake amount" was the wrong
claim; the honest stage-0 contract is direct alpha-price valuation. With the
hotkey and coldkey views both covered, the surviving summary helpers are now
anchored to that same contract instead of inheriting swap-era reporting
behavior.

The next truthful layer down was reporting rather than helper valuation. The
stage-0 `StakeInfo` shape had still been carrying `locked`, `tao_emission`, and
`drain` from the broader subtensor surface even though the product never used
them. The repo now takes the stronger path: those fields are gone from the
stage-0 payload entirely, and the focused regression plus runtime compile path
prove the smaller contract is still the active one. That is better than leaving
zero-only baggage in place just because it is inherited.

The same distinction mattered immediately for fees. The runtime's actual stage-0
swap surface is `Stage0NoopSwap`, which means identity conversion, zero fee,
unit price, and no protocol TAO. But the pallet's default test runtime still
uses the full swap pallet, so asserting "all stake fees are zero" there would
have been a fake proof. The honest move was to add direct runtime tests for the
noop swap contract itself instead of overclaiming from the broader pallet mock.

That proof now reaches the runtime API boundary too. The stripped runtime does
not only behave like a noop swap internally; it also reports that same contract
through `SwapRuntimeApi`: a scaled unit price and identity `sim_swap_*`
results with zero fees. That matters because stage 0 needs truthful external
surfaces, not just truthful internals.

The fee surface is now pinned in the same way. Instead of merely assuming that
`get_stake_fee` must be zero because `Stage0NoopSwap::approx_fee_amount` is
zero, the runtime now has a direct test over representative add/remove/move fee
shapes. That is a better stopping point for this seam because it proves the
real outward contract while still avoiding a premature compatibility edit to the
older runtime API/reporting structures.

The next reporting shrink is now in too. `DynamicInfo` had still been carrying
`emission` and `pending_root_emission` even though the stage-0 pallet path only
ever populated them with zero. Those fields are now gone from the active
payload, and the focused regression proves the remaining encoded shape is still
truthful by decoding it fully and matching the surviving reserves and pending
emission fields against live storage rather than a mistaken "all reserves must
match" assumption.

The same cleanup now reaches the broader subnet-reporting surface. `Metagraph`
and `SelectiveMetagraph` had still been carrying the deprecated
`subnet_emission` and `pending_root_emission` fields even though both builders
always hardcoded them to zero. Those fields are now gone from the active stage-0
payloads, and the old selective indexes are retired in place instead of
renumbering the remaining map. That keeps the truthful contract smaller without
quietly shifting the meaning of every later selective index.

The narrower subnet-info DTOs are now aligned the same way. `SubnetInfo` and
`SubnetInfov2` had still been carrying `network_modality`, `network_connect`,
and `emission_value[s]` even though both builders always returned the same dead
stage-0 values: `0`, `[]`, and `0`. Those placeholders are now gone from the
active payloads, and the focused `stage_0_flow` proof fully decodes both
smaller shapes and matches their surviving fields against the live pallet
getters.

That made the next compatibility cut possible too. Once both payloads had been
reduced to the same truthful stage-0 content, keeping the older v1 subnet-info
surface around no longer bought the branch anything. The active runtime API,
runtime implementation, and JSON-RPC layer now expose only `SubnetInfov2`.
`stage_0_flow` was tightened to prove that single surviving surface instead of
continuing to validate a compatibility duplicate that stage 0 no longer needs.

The same move is now complete for subnet hyperparams too. The active pallet
path, runtime API, runtime implementation, and JSON-RPC layer no longer carry
the older v1 `SubnetHyperparams` interface. `SubnetHyperparamsV2` is now the
only live hyperparams surface, and `stage_0_flow` gained a focused decode proof
that checks the surviving payload against the live getters rather than guessed
defaults for the v2-only flags.

The next reporting seam tightened in a slightly different way. `SubnetState`
was not carrying a zero-only placeholder; it was carrying a mismatched one.
The `emission_history` field was being assembled across every subnet while the
rest of the payload described one queried subnet. That is inherited reporting
shape, not truthful stage-0 state. The field is now gone from the active
payload, and the focused regression decodes the smaller struct and checks the
surviving fields against the same storage sources the builder uses.

The next aggregate seam is tighter now too. `get_all_dynamic_info` was not
wrong about the live payloads themselves, but it was still preserving dead
subnet keys as `None` entries because it walked the raw subnet-key map. Stage 0
does not need that compatibility padding. The active pallet path, runtime API,
and runtime implementation now expose `Vec<DynamicInfo>` directly, and the
focused regression proves false subnet markers are omitted rather than encoded
into the outward surface.

The next aggregate seam follows the same doctrine for metagraphs too.
`get_all_metagraphs` was not wrong about the live metagraph payloads, but it
was still preserving dead subnet keys as `None` entries because it walked the
raw subnet-key map. Stage 0 does not need that compatibility padding either.
The active pallet path, runtime API, and runtime implementation now expose
`Vec<Metagraph<_>>` directly, and the focused regression seeds a false subnet
marker and proves the aggregate surface matches the live per-netuid
`get_metagraph(...)` payloads instead of encoding dead holes.

That same aggregate cleanup now reaches the surviving subnet-info list too.
`get_subnets_info_v2` was not wrong about the live payloads, but it was still
preserving dead subnet keys as `None` entries because it walked stored subnet
keys through an optional aggregate shape. Stage 0 does not need that padding.
The active pallet path, runtime API, and runtime implementation now expose
`Vec<SubnetInfov2<_>>` directly, and the focused regression seeds a false
subnet marker and proves the aggregate surface matches the live per-netuid
`get_subnet_info_v2(...)` payloads instead of encoding dead holes.

That same cleanup now reaches the surviving mechanism aggregate too.
`get_all_mechagraphs` was not wrong about the live payloads, but it was still
carrying an optional wrapper even though the builder only walks valid
subnet/mechanism pairs. Stage 0 does not need that extra compatibility shell.
The active pallet path, runtime API, and runtime implementation now expose
`Vec<Metagraph<_>>` directly for the mechanism aggregate, and the focused
regression seeds a live mechanism count plus a false subnet marker and proves
the aggregate surface matches the live per-pair `get_mechagraph(...)`
payloads instead of encoding dead holes.

The next fix was not another shape cut; it was a truth correction. The active
neuron reporting path still claimed its `stake` field was a coldkey-to-stake
 map including delegations, but it was really emitting one owner-coldkey row
carrying the hotkey's total subnet stake. That is worse than inert baggage
because it misattributes live funds. The full and lite neuron builders now use
the real per-coldkey subnet stake entries, and the focused regression stakes
both the owner and a delegator into one hotkey and proves the outward map
matches the live staking getters.

The next fix was another truth correction in the same spirit. The delegate
reporting path was still assuming live subnets formed a dense index space when
it rebuilt nominator stake from share pools. That meant a dead lower subnet key
could cause a live higher-netuid nomination to disappear from the outward
delegate report. The delegate builder now keys share pools by `NetUid`
directly, and the focused regression creates a sparse live-netuid layout and
proves the live nominator stake still shows up in the outward delegate payload.

The next fix stayed in delegate reporting, but on the economic side instead of
the nominator map. `return_per_1000` was still dividing by root-only stake even
though the outward daily return aggregates emissions across the delegate's live
registrations. That could report a false zero or inflated value for delegates
whose active stake lived entirely off root. The builder now uses
`get_total_stake_for_hotkey`, and the focused regression creates a delegate
with stake and emissions only on a non-root subnet and proves the outward
`return_per_1000` matches the pallet helper formula.

The next fix stayed in the same struct, but corrected the other economic field.
`total_daily_return` was labeled as delegator return while still exposing
pre-take emission, so any nonzero delegate commission would overstate what
delegators actually receive. The builder now shares one helper for net
delegator return and per-1000 return, and the focused regression sets a 50%
delegate take and proves the outward `total_daily_return` matches the
post-commission amount instead of raw emission.

The next slice was a real outward-surface shrink again rather than another data
truth correction. The custom runtime/RPC layer was still exporting
`get_coldkey_auto_stake_hotkey` and `get_subnet_to_prune` even though neither
helper is part of the active stage-0 doctrine and neither had any remaining
consumer beyond the compatibility wiring itself. Those endpoints are now gone
from the runtime API, runtime impl, and RPC server, and the now-unreferenced
auto-stake pallet getter was removed too. This keeps the live stage-0 contract
closer to what the genesis plan actually uses instead of preserving old helper
surface by inertia.

The next slice applied the same rule to the next tier of RPC-only inspection
surface. `get_subnet_state`, `get_selective_metagraph`, and
`get_selective_mechagraph` were still available over the custom runtime/RPC
boundary, but they had no remaining doctrine, spec, chain-client, or other
in-repo consumer outside that compatibility shell. They are now removed from
the runtime API, runtime impl, and RPC server. The pallet-side helpers stay in
place for local code and tests, but stage 0 no longer advertises them as live
node RPC contract.

The next slice applied the same rule one more time to delegate reporting at the
node boundary. The pallet-level delegate helpers still matter for local tests
and truth corrections, but the custom runtime/RPC trio around
`get_delegates`, `get_delegate`, and `get_delegated` had no remaining
doctrine, spec, chain-client, or other in-repo consumer. Those outward methods
are now removed from the runtime API, runtime impl, and RPC server, keeping the
live node contract closer to the single RPC path the current genesis plans
actually depend on.

The next slice finished that contraction. Once the delegate layer was gone, the
checked-in doctrine still named only one custom node RPC:
`neuronInfo_getNeuronsLite`. Everything else in the custom runtime/RPC shell
was inherited compatibility surface. The runtime API, runtime impl, and RPC
server are now collapsed to that one path, and the dead runtime helper left
behind by the removed stake-fee surface is gone too. Stage 0 now advertises a
node-level custom RPC contract that matches the bootstrap plans and shared
client instead of a broad subtensor-era inspection API.

The metagraph cleanup now goes one step further too. Even after removing the
deprecated subnet-emission placeholders, the full and selective metagraph
payloads were still carrying `tao_dividends_per_hotkey` even though that branch
was always synthesized as zero for every hotkey. That zero-only payload is now
gone, and selective index `70` is retired in place while
`AlphaDividendsPerHotkey`, `Validators`, and `Commitments` keep their old index
meanings.

## Context and Orientation

The relevant files are:

- `crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs`
- `crates/myosu-chain/pallets/game-solver/src/subnets/weights.rs`
- `crates/myosu-chain/pallets/game-solver/src/staking/`
- `crates/myosu-chain/pallets/game-solver/src/coinbase/`
- `crates/myosu-chain/pallets/game-solver/src/tests/`

Stage-0 keep surface:

- subnet creation and dissolution
- neuron registration
- stake add/remove and delegation
- serving endpoint registration
- direct weights and commit-reveal v2
- epoch processing and emissions

Stage-0 remove surface:

- timelocked CRV3 commit/reveal
- crowdloan and leasing behavior
- root-network and AMM-era swap machinery that does not belong in a single-token
  stage-0 network

## Milestones

### Milestone 1: Remove CRV3 timelock from the live pallet path

Delete or quarantine the timelocked weight commit surface so the pallet only
exposes direct weights and commit-reveal v2.

Proof commands:

    cargo check -p pallet-game-solver
    rg -n "commit_timelocked|CRV3|timelock" crates/myosu-chain/pallets/game-solver/src

Current status: the live extrinsics, dead CRV3/timelocked event variants, dead
timelocked commit implementation, and live leasing behavior are removed; the
unused commit-reveal version knob is removed; the deprecated
timelocked/CRV3/leasing storages are no longer part of the default build; the
stage-0 pallet proof is green; what remains is archive-only migration
scaffolding and small type-level residue for follow-on cleanup.

### Milestone 2: Narrow staking to the stage-0 model

Replace the full swap-driven staking path with the stage-0 seam used by the
runtime, keeping only the logic required for stake, delegation, and emissions.

Proof commands:

    cargo check -p pallet-game-solver
    cargo test -p pallet-game-solver staking --quiet

### Milestone 3: Prove the stage-0 pallet flow

Add or update an integration-style pallet test that proves the intended chain
behavior from subnet creation through weight setting and reward emission.

Proof command:

    cargo test -p pallet-game-solver stage_0_flow --quiet

## Plan of Work

Start at the pallet edge where users and validators interact: weights,
registration, staking, and emissions. Remove the extrinsics and code paths that
cannot exist in the intended stage-0 network. Then write the test that proves
the surviving pallet can still run the actual chain loop it is supposed to own.

## Concrete Steps

From `/home/r/coding/myosu`:

    cargo check -p pallet-game-solver
    sed -n '1,240p' crates/myosu-chain/pallets/game-solver/src/macros/config.rs
    rg -n "commit_timelocked|CRV3|timelock|SwapInterface|crowdloan|leasing" \
      crates/myosu-chain/pallets/game-solver/src

## Validation and Acceptance

This plan is complete when:

- `cargo check -p pallet-game-solver` passes.
- The live pallet source no longer exposes CRV3 timelock paths.
- The surviving staking path matches the intended stage-0 model.
- `cargo test -p pallet-game-solver stage_0_flow --quiet` passes.

Current status: the public pallet API no longer exposes the CRV3/timelocked or
leasing extrinsics, the crate builds lint-clean, and
`cargo test -p pallet-game-solver stage_0_flow --quiet` now passes. The
remaining work is internal storage and migration cleanup, not public-surface
proof. The live coldkey stake-summary helpers also now use the same direct
stage-0 price model as the runtime's noop swap surface instead of AMM-style
swap simulation. The exposed `StakeInfo` payload has now been shrunk as well:
the old zero-only `locked`, `tao_emission`, and `drain` fields are gone from
the stage-0 payload instead of being carried as inert baggage. The real
runtime's zero-fee noop swap contract is pinned directly too, so the repo no
longer has to treat the broader pallet mock runtime as if it were
stage-0-identical.
The subnet-info runtime/RPC surface is also now v2-only: the old v1
`SubnetInfo` path has been removed instead of being carried alongside the
smaller active contract.
The same is now true of subnet hyperparams: the old v1
`SubnetHyperparams` path is gone, and the focused stage-0 proof family is now
`6 passed, 0 failed`.
The focused stage-0 proof family now also covers the smaller `SubnetState`
surface, bringing it to `7 passed, 0 failed`.
The aggregate dynamic-info, metagraph, subnet-info, and mechagraph paths are
now pinned too, and the surviving neuron stake-map path is pinned as truthful,
and the surviving delegate sparse-netuid, return-per-1000, and total-daily-
return paths are pinned as truthful, bringing the focused stage-0 proof family
to `15 passed, 0 failed`.

## Idempotence and Recovery

Prefer deleting one obsolete surface at a time and recompiling immediately. If a
test or helper still expects the removed path, update the test to the new
stage-0 truth instead of reviving the old mechanism.

## Interfaces and Dependencies

Depends on: plan 003 for runtime-side strip-down.
Blocks: plan 007 and full stage-0 exit proof.
