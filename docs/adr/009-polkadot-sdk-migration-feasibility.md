# ADR 009: Polkadot SDK Migration Feasibility

- Status: Accepted
- Date: 2026-04-05
- Deciders: Myosu maintainers
- Consulted: `IMPLEMENTATION_PLAN.md`, `specs/050426-chain-runtime-pallet.md`, `specs/050426-network-consensus.md`, `docs/adr/002-substrate-fork-strategy.md`, `ops/decision_log.md`
- Informed: chain, runtime, node, and operator contributors
- Related: `Cargo.toml`, `crates/myosu-chain/node/Cargo.toml`, `crates/myosu-chain/runtime/Cargo.toml`, `WORKLIST.md`, `AGENTS.md`

## Context

`specs/050426-chain-runtime-pallet.md` leaves one important chain-dependency
question open: is Myosu's pinned `opentensor/polkadot-sdk` fork still
necessary, or can the repo move back to upstream `paritytech/polkadot-sdk`
without reopening stage-0 chain risk?

The live repo is deeply pinned to the fork:

- root `Cargo.toml` references `https://github.com/opentensor/polkadot-sdk.git`
  83 times at rev `71629fd93b6c12a362a5cfb6331accef9b2b2b61`
- `crates/myosu-chain/runtime/Cargo.toml` repeats that fork pin across 35
  runtime dependencies
- `crates/myosu-chain/node/Cargo.toml` directly depends on the node surfaces
  touched by the fork-only patch set: `sc-cli`, `sc-consensus-babe`,
  `sc-consensus-grandpa`, `sc-keystore`, and `sc-transaction-pool`
- `crates/myosu-chain/runtime/Cargo.toml` directly depends on
  `frame-support`, `frame-system`, `pallet-aura`, and `pallet-grandpa`

The initial migration hypothesis in the spec was optimistic: an upstream move
might be easy if the opentensor fork only carried `pallet-subtensor`-specific
APIs. The audit disproves that assumption.

### Audit Snapshot

The pinned fork commit is:

- `71629fd93b6c12a362a5cfb6331accef9b2b2b61`
- dated `2026-02-26 15:35:03 -0300`
- subject: `Merge pull request #8 from opentensor/polkadot-stable2506-2-otf-dispatch-guard`

The nearest upstream line is `upstream/stable2506`, not `master` and not the
older stable branches.

The original 2026-04-05 audit snapshot in this ADR recorded `43` upstream-only
commits. A fresh fetch on 2026-04-08 shows `49`, so the fork-only count is
stable but the upstream branch has moved since the first write-up:

| Upstream branch | Fork-only commits | Upstream-only commits |
|---|---:|---:|
| `stable2506` | 21 | 49 |
| `stable2503` | 327 | 191 |
| `stable2412` | 955 | 209 |
| `stable2409` | 1320 | 193 |
| `stable2407` | 1581 | 123 |
| `master` | 88 | 971 |

Against `stable2506`, the fork is not huge, but it is not trivial either:

- merge-base: `2caeef482a437414c6bed2395a16abe08fccbfbb`
- fork-only commits: 21
- upstream-only commits: 43
- fork-only changed files: 21
- direct head-to-head changed files: 135

The fork-only file set is concentrated in live consensus and runtime seams, not
in `pallet-subtensor`:

- `substrate/client/consensus/babe/src/*`
- `substrate/client/consensus/grandpa/src/*`
- `substrate/client/transaction-pool/src/*`
- `substrate/client/cli/src/lib.rs`
- `substrate/client/keystore/src/local.rs`
- `substrate/frame/system/src/*`
- `substrate/frame/support/src/dispatch.rs`
- `substrate/frame/support/procedural/src/construct_runtime/expand/call.rs`
- `substrate/frame/aura/src/lib.rs`
- `substrate/frame/babe/src/lib.rs`

The fork-only commit subjects reinforce that this is consensus-path code, not
isolated subtensor glue:

- `sc-grandpa: fix warp set id when non-static`
- `Modify block import.`
- `Modify warp proof`
- `Handle invalid warp proofs`
- `sc-babe: remove bad hardcoding in fn find_pre_digest`
- `pallet-babe: don't check slot on genesis`
- `sc-keystore: expose additional pub methods`
- `fix transaction pool tx replacement by banning them`
- `sc-cli: support creating Runner more than once`

This matters because the repo recently had a misleading multi-node consensus
story. The old `P-011` contract assumed a 3-authority equal-weight GRANDPA set
could keep finalizing after one authority stopped, but the pinned
`finality-grandpa` threshold math requires all 3 votes in that configuration.
The fork-only patch set still overlaps the exact consensus path used by the
now-landed 4-authority proof, so migration work should keep relying on real
proofs instead of intuition about quorum sizes.

### Commit Classification Refresh (2026-04-08)

Fresh classification against `upstream/stable2506` yields:

- `10` commits currently needed by myosu
- `8` commits that look subtensor-specific or safe to drop
- `3` commits that remain uncertain and need a focused migration spike

The biggest theme is that the fork is not just "subtensor baggage." Myosu's
live repo now directly consumes several forked APIs:

- `crates/myosu-chain/node/src/service.rs` calls the forked GRANDPA block import
  surface with `skip_block_justifications` and uses
  `warp_proof::HardForks::new_initial_set_id(...)`.
- `crates/myosu-chain/node/src/service.rs` also calls the forked
  `LocalKeystore::raw_public_keys()` and `key_phrase_by_type()` helpers to copy
  Aura keys into Babe keys.
- `crates/myosu-chain/node/src/command.rs` intentionally re-enters runner
  creation while switching between Aura and Babe services.
- `crates/myosu-chain/runtime/src/lib.rs` wires
  `type DispatchGuard = pallet_game_solver::CheckColdkeySwap<Runtime>;`, which
  depends on the dispatch-guard fork carried in merge commit `71629fd`.

| Commit | Area | Classification | Rationale |
|---|---|---|---|
| `4fea3a84` | GRANDPA warp proof | Needed by myosu | `service.rs` constructs `warp_proof::HardForks::new_initial_set_id(set_id)` for live/dev chains; upstream `NetworkProvider::new(...)` does not match this surface. |
| `23bad86d` | GRANDPA block import | Needed by myosu | `service.rs` passes `skip_block_justifications` into `sc_consensus_grandpa::block_import(...)`; the current node compile/runtime contract depends on this forked API and behavior. |
| `9372cfa6` | Txpool priority | Uncertain | Changes tx priority ordering by removing longevity from tie-breaking. The repo has no explicit contract for this, and local chains already default away from the fork-aware pool. |
| `285fe2d4` | GRANDPA warp proof | Needed by myosu | Myosu sets non-zero initial GRANDPA set IDs for live and development chain types; this patch fixes the `HardForks::new_initial_set_id(...)` path the node already uses. |
| `29b399f4` | CLI runner lifecycle | Needed by myosu | `command.rs` can start Aura, then restart into Babe in the same process; repeated `create_runner(...)` is part of the checked-in node behavior. |
| `35625a45` | BABE logging | Subtensor-specific | Adds trace logs only; no myosu compile or runtime contract depends on them. |
| `44b72423` | Aura try-state | Needed by myosu | The repo still carries an explicit Aura->Babe transition surface (`initial_consensus`, staged Babe constants, keystore copy). This patch keeps that path compatible with try-state/runtime-upgrade checks. |
| `291b3899` | Keystore API | Needed by myosu | `service.rs` directly calls `LocalKeystore::raw_public_keys()` and `key_phrase_by_type()` inside `copy_keys(...)`; upstream without this patch would not compile. |
| `27d5dfe7` | BABE verifier ctor | Subtensor-specific | Myosu does not construct `BabeVerifier` directly and does not carry the hybrid import queue named in the commit message. |
| `d9485da7` | BABE first-block import | Needed by myosu | The checked-in Aura->Babe transition path relies on treating the first Babe block after an Aura chain as `UnimportedGenesis`; without that, the imported path can reject the handoff. |
| `403b3707` | BABE predigest handling | Needed by myosu | Same checked-in Aura->Babe handoff path: `find_pre_digest` must tolerate a missing pre-runtime digest on the first Babe block after Aura. |
| `7d1855eb` | Pallet BABE genesis slot | Needed by myosu | Complements the first-Babe-block handoff by skipping the slot assertion when `GenesisSlot` is still uninitialized during Aura->Babe transition. |
| `81fa2c54` | Txpool logging | Subtensor-specific | Debug-level logging only; no repo contract or CLI surface depends on this verbosity change. |
| `e53b4eb9` | GRANDPA warp proof validation | Needed by myosu | Myosu enables warp sync in `service.rs`; this patch hardens the same proof-validation path used by restart/catch-up flows. |
| `467c6bf8` | Tx replacement env toggle | Subtensor-specific | The repo never sets `SUBSTRATE_TXPOOL_ENABLE_REPLACE_PREVIOUS`, and later txpool commits supersede the exact toggle shape. |
| `58add17a` | Txpool tracing | Subtensor-specific | Logging-only trace enrichment; no compile/runtime dependency in myosu. |
| `df2f9b53` | Txpool panic/log path | Subtensor-specific | Converts a panic/logging path without introducing a repo-visible contract myosu depends on. |
| `903db04e` | Tx replacement policy | Uncertain | Potentially useful behavior hardening, but the repo has no proof that it is required, and `command.rs` already forces `SingleState` txpool on local chains because the fork-aware path is not yet reconciled. |
| `a088aac8` | Txpool log level | Subtensor-specific | Escalates logging severity only; no repo contract depends on this level change. |
| `a584a577` | Txpool error wrapping | Subtensor-specific | Follow-on control-flow/logging cleanup around the same txpool replacement work; no direct myosu dependency is visible. |
| `71629fd9` | FRAME dispatch guard | Needed by myosu | `runtime/src/lib.rs` sets `DispatchGuard = pallet_game_solver::CheckColdkeySwap<Runtime>`, and the pallet tests wire the same surface; this merge commit is the forked dispatch-guard implementation that makes that compile. |

## Decision

Myosu should not attempt an upstream `polkadot-sdk` migration during stage-0
stabilization.

The active repo position is:

- keep the current opentensor fork pin at
  `71629fd93b6c12a362a5cfb6331accef9b2b2b61` for now
- treat migration to upstream as feasible in principle, but not yet justified
- reopen migration after stage-0 only as a dedicated spike, now that the
  classification work is done, and only after deciding whether Myosu still
  intends to keep the checked-in Aura->Babe transition surface and what
  transaction-pool replacement policy it wants to preserve

This is a no-go for an immediate re-pin, not a claim that the fork should be
carried forever.

## Alternatives Considered

### Option A: Hold the fork through stage-0 and audit before migrating

This wins because it matches the live repo evidence. The fork delta is moderate
enough that a future migration spike is plausible, but it touches too many
consensus-critical surfaces to swap blindly while stage-0 network proofs are
still incomplete.

### Option B: Re-pin directly to upstream `stable2506` now

This is rejected because the audit disproves the "subtensor-only divergence"
assumption. Myosu would need to either re-carry or explicitly retire 21
fork-only commits that touch GRANDPA, BABE, txpool, frame-system,
frame-support dispatch expansion, CLI startup, and keystore behavior. That is
not a safe background cleanup while `P-011` is still red.

### Option C: Treat the opentensor fork as permanent and stop auditing it

This is rejected because the divergence from `stable2506` is still small enough
to revisit later. Declaring the fork permanent now would turn a tactical carry
decision into doctrine without proving that Myosu actually needs all of the
fork-only behavior.

## Consequences

### Positive

- The repo now has a truthful answer to `F-008`: immediate migration is not the
  next stage-0 move.
- Chain work can separate two questions that would otherwise get conflated:
  "what authority count is needed for one-node-down GRANDPA tolerance?" and
  "what would break on an upstream re-pin?"
- Future migration work now has a concrete commit-level map: `10` needed
  patches, `8` safe-drop candidates, and `3` unresolved txpool deltas.

### Negative

- Myosu continues carrying the inherited fork and its update/security debt for
  now.
- The 43 upstream-only `stable2506` commits remain unapplied until a later
  migration spike.
- This ADR does not ship the multi-authority proof; it only prevents a risky
  and poorly scoped migration from being treated as obvious cleanup.

### Follow-up

- Keep the authority-count lesson from `P-011` straight: the old 3-authority
  one-node-down expectation was a contract bug, so future migration work should
  rerun the 4-authority proof instead of treating the retired 3-node stall as a
  chain defect.
- Decide whether to keep or remove the checked-in Aura->Babe transition path;
  that single choice determines whether four BABE/AURA patches stay in the
  "needed" bucket.
- Resolve the three uncertain txpool patches (`9372cfa6`, `903db04e`, and the
  surrounding policy question) before attempting any upstream re-pin.
- If a future migration spike begins, rerun the stage-0 proof set at minimum:
  `local_loop`, `two_node_sync`, the 4-authority finality proof, and the
  emission agreement proof.

## Reversibility

Moderate now, harder later.

Because stage-0 is still stabilizing, Myosu can revisit this once the now
classified fork-only patch set is either pared down or intentionally reaffirmed
and the network proofs are stronger. The decision gets harder to reverse after
public operator releases, runtime upgrades, or new features start depending on
fork-specific behavior in consensus or node startup.

The trigger to reopen this ADR is not "upstream has moved again." It is:

- the 21 fork-only commits have been triaged against upstream
- the multi-authority finality contract is still satisfied on the pinned fork
- a migration spike can run the chain proof suite without mixing dependency
  surgery into unresolved consensus debugging
- the repo has decided whether Aura->Babe transition support and the current
  txpool replacement semantics are still owned surfaces

## Validation / Evidence

- `rg -c "https://github.com/opentensor/polkadot-sdk.git" Cargo.toml crates/myosu-chain/runtime/Cargo.toml`
- `sed -n '1,220p' crates/myosu-chain/node/Cargo.toml`
- `git -C /tmp/polkadot-sdk-audit show -s --format='%H %ci %s' 71629fd93b6c12a362a5cfb6331accef9b2b2b61`
- `git -C /tmp/polkadot-sdk-audit merge-base 71629fd93b6c12a362a5cfb6331accef9b2b2b61 upstream/stable2506`
- `git -C /tmp/polkadot-sdk-audit rev-list --left-right --count 71629fd93b6c12a362a5cfb6331accef9b2b2b61...upstream/stable2506`
- `git -C /tmp/polkadot-sdk-audit diff --name-only 71629fd93b6c12a362a5cfb6331accef9b2b2b61 upstream/stable2506 | wc -l`
- `git -C /tmp/polkadot-sdk-audit log --oneline $(git -C /tmp/polkadot-sdk-audit merge-base 71629fd93b6c12a362a5cfb6331accef9b2b2b61 upstream/stable2506)..71629fd93b6c12a362a5cfb6331accef9b2b2b61`
- `git -C /tmp/polkadot-sdk-audit diff --name-only $(git -C /tmp/polkadot-sdk-audit merge-base 71629fd93b6c12a362a5cfb6331accef9b2b2b61 upstream/stable2506)..71629fd93b6c12a362a5cfb6331accef9b2b2b61`
- `git -C /tmp/polkadot-sdk-audit diff --dirstat=files,0 $(git -C /tmp/polkadot-sdk-audit merge-base 71629fd93b6c12a362a5cfb6331accef9b2b2b61 upstream/stable2506)..71629fd93b6c12a362a5cfb6331accef9b2b2b61`
- `rg -n "DispatchGuard =|copy_keys|initial_consensus|HardForks::new_initial_set_id|skip_block_justifications" crates/myosu-chain -g '!target'`
