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
older stable branches:

| Upstream branch | Fork-only commits | Upstream-only commits |
|---|---:|---:|
| `stable2506` | 21 | 43 |
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

This matters because the repo already has a live unresolved multi-node
consensus problem. `P-011` and `WORKLIST.md` `NET-FINALITY-001` record that a
3-authority `devnet` stalls GRANDPA finality at `#2` after one authority is
stopped, even though best blocks keep importing on the remaining authorities.
The fork-only patch set overlaps the exact consensus path currently under
investigation.

## Decision

Myosu should not attempt an upstream `polkadot-sdk` migration during stage-0
stabilization.

The active repo position is:

- keep the current opentensor fork pin at
  `71629fd93b6c12a362a5cfb6331accef9b2b2b61` for now
- treat migration to upstream as feasible in principle, but not yet justified
- reopen migration only after the 21 fork-only commits are classified and the
  current multi-node finality stall is understood well enough that a re-pin
  would not blur cause and effect

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
  "why is 3-node finality stalling?" and "what would break on an upstream
  re-pin?"
- Future migration work has a concrete starting point: classify the 21
  fork-only commits instead of re-auditing the whole SDK from scratch.

### Negative

- Myosu continues carrying the inherited fork and its update/security debt for
  now.
- The 43 upstream-only `stable2506` commits remain unapplied until a later
  migration spike.
- This ADR does not solve the live GRANDPA stall; it only prevents a risky and
  poorly scoped migration from being treated as obvious cleanup.

### Follow-up

- Classify each of the 21 fork-only commits as `required`, `replaceable`, or
  `drop` against `stable2506` or the next candidate upstream line.
- Resolve or at least root-cause the `P-011` / `NET-FINALITY-001` 3-node
  finality stall before using migration as a remedy.
- If a future migration spike begins, rerun the stage-0 proof set at minimum:
  `local_loop`, `two_node_sync`, the eventual 3-node finality proof, and the
  emission agreement proof.

## Reversibility

Moderate now, harder later.

Because stage-0 is still stabilizing, Myosu can revisit this once the fork-only
patch set is understood and the network proofs are stronger. The decision gets
harder to reverse after public operator releases, runtime upgrades, or new
features start depending on fork-specific behavior in consensus or node startup.

The trigger to reopen this ADR is not "upstream has moved again." It is:

- the 21 fork-only commits have been triaged against upstream
- the current finality stall has a root cause or a bounded reproduction story
- a migration spike can run the chain proof suite without mixing dependency
  surgery into unresolved consensus debugging

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
