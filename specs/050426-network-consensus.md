# Specification: Network Model and Consensus

## Objective

Define the network model, consensus mechanism, and cross-node agreement
properties for the myosu chain. This spec covers multi-node behavior from
what is proven today (two-node block sync) through the planned multi-node
finality and emission agreement proofs required for stage-0 exit.

## Evidence Status

### Proven

- **Consensus mechanism**: Aura (block authoring) + GRANDPA (finality),
  wired in `crates/myosu-chain/node/src/service.rs`. GRANDPA justification
  period is 512 blocks.
- **Chain specs**: Four variants exist --
  `localnet` (single or multi-authority dev seeds Alice/Bob/Charlie),
  `devnet` (four named authorities derived from `//myosu//devnet//authority-{1,2,3,4}`),
  `testnet` (three-authority Alice/Bob/Charlie with separate chain type),
  `finney` (production placeholder).
  Source: `crates/myosu-chain/node/src/chain_spec/`.
- **Two-node block sync**: `tests/e2e/two_node_sync.sh` starts a bootnode
  authority and a sync peer on `--chain devnet`. It proves:
  - Bootnode produces blocks with authority keys from `MYOSU_NODE_AUTHORITY_SURI`.
  - Peer discovers bootnode via libp2p multiaddr.
  - Peer syncs to the same best block number as the bootnode.
  - Both nodes report >= 1 peer.
- **Devnet genesis state**: The devnet chain spec bootstraps subnet 7 with a
  pre-registered owner, pool balances, and storage items. This is the starting
  state all devnet nodes share.
- **Fixed-point math**: Emission accounting uses `substrate_fixed` types
  (I32F32, I64F64, I96F32, U64F64) from the encointer fork. These types
  produce bit-identical results for the same inputs regardless of execution
  context, which is a prerequisite for cross-node emission agreement.

### Not Proven

- **GRANDPA finality across multiple nodes**: The two-node sync test checks
  best block agreement but does not verify finalized block advancement.
  GRANDPA finality has not been tested with 4 authorities.
- **Node restart resilience**: No test stops a node and verifies it catches up
  without forking the chain.
- **Cross-node emission agreement**: Emission distribution has been tested in
  single-node unit and E2E tests only. No multi-node test compares emission
  state across independent node processes.
- **Network partition tolerance**: No test simulates network splits or delayed
  message delivery between validators.
- **Four-authority automated devnet**: The devnet chain spec should define four
  authority keys so the proof can survive one authority loss, but no test
  script currently starts all four as separate processes.

## Architecture

### Consensus Layers

1. **Block authoring (Aura)**: Round-robin slot assignment among registered
   authorities. Each authority produces blocks in its assigned slot. Slot
   duration is configured in the runtime. Block production continues as long
   as at least one authority is online.

2. **Finality (GRANDPA)**: Byzantine fault tolerant finality gadget. The
   pinned `finality-grandpa` implementation computes the vote threshold as
   `total_weight - floor((total_weight - 1) / 3)`. With three equal-weight
   authorities the threshold is 3 of 3, so one-node-down tolerance is
   impossible. With four equal-weight authorities the threshold is 3 of 4,
   which is the minimum stage-0 configuration that can truthfully prove
   one-node-down resilience. Finalized blocks are irreversible.

3. **Chain selection (LongestChain)**: Fork choice rule selects the longest
   valid chain. GRANDPA finality prevents permanent forks by anchoring the
   canonical chain at the last finalized block.

### Authority Configuration

The devnet chain spec defines authorities as Aura + GRANDPA key pairs:

| Authority | Aura key source | GRANDPA key source |
|---|---|---|
| authority-1 | `//myosu//devnet//authority-1` (sr25519) | same URI (ed25519) |
| authority-2 | `//myosu//devnet//authority-2` (sr25519) | same URI (ed25519) |
| authority-3 | `//myosu//devnet//authority-3` (sr25519) | same URI (ed25519) |
| authority-4 | `//myosu//devnet//authority-4` (sr25519) | same URI (ed25519) |

Authority keys are injected at node startup via the `MYOSU_NODE_AUTHORITY_SURI`
environment variable, which the node reads to populate its local keystore.

### Network Topology

Stage-0 operates as a local devnet. Nodes discover each other via explicit
`--bootnodes` multiaddr arguments. There is no DHT-based peer discovery or
public bootstrap node list.

## Acceptance Criteria

### Phase 1: Multi-node finality (Plans 006, 007)

- Four authority nodes start on isolated ports, each with a distinct
  authority key, all using `--chain devnet`.
- All four nodes produce blocks and the finalized block number advances
  on each node.
- Stopping one authority does not halt block production or finality
  because the surviving 3 of 4 still meet GRANDPA threshold.
- A restarted authority syncs to the current chain tip and resumes
  participation in finality.
- All live nodes agree on the finalized block hash at any given
  finalized block number.

### Phase 2: Cross-node emission agreement (Plan 008)

- After multiple epoch transitions on the four-authority devnet, all live
  nodes report identical emission distributions when queried via RPC.
- The accounting invariant (total emission equals sum of individual
   distributions) holds independently on each node.
- Comparison tolerance for fixed-point values is zero (bit-identical).

### Phase 3: Decision gate (Plan 009)

- All six gate criteria pass before operator packaging proceeds:
   dead code removed, emission invariant proven, test suite clean,
   storage audited, multi-node finality proven, cross-node emission agreed.

## Verification

### Currently runnable

```bash
# Two-node sync (proven)
bash tests/e2e/two_node_sync.sh
```

### Planned verification scripts

```bash
# Four-authority devnet finality
bash tests/e2e/four_node_finality.sh

# Consensus resilience under node restart (Plan 007)
bash tests/e2e/consensus_resilience.sh

# Cross-node emission agreement (Plan 008)
bash tests/e2e/cross_node_emission.sh
```

Each planned script should exit nonzero on any assertion failure and clean up
all spawned node processes on exit (including on failure), matching the pattern
established by `two_node_sync.sh`.

## Open Questions

1. **GRANDPA with the opentensor fork**: The polkadot-sdk fork used by myosu
   may carry patches that affect GRANDPA behavior. After the threshold math
   correction, the remaining question is whether a four-authority proof behaves
   exactly as upstream GRANDPA would.

2. **Fixed-point determinism across architectures**: `substrate_fixed` produces
   bit-identical results on the same architecture. If devnet nodes run on
   different CPU architectures (x86 vs ARM), does determinism still hold?
   Stage-0 is single-architecture, but this matters for future operator
   deployments.

3. **Minimum authority count for meaningful proof**: Four equal-weight
   authorities is the minimum configuration that preserves one-node-down
   tolerance under the pinned GRANDPA threshold math. Is four sufficient for
   the stage-0 exit gate, or should the proof target a higher count to surface
   timing-dependent bugs?

4. **Warp sync vs full sync**: The service code references `WarpSyncProvider`.
   Should new nodes joining the devnet use warp sync to catch up, or is full
   sync sufficient and simpler for the proof?

5. **Finality stall recovery**: If all three nodes restart simultaneously
   (e.g., power failure), can they recover finality from on-disk state without
   manual intervention? This is not covered by the single-node restart test in
   Plan 007.

6. **Emission query RPC availability**: Cross-node emission comparison
   (Plan 008) requires querying emission state via RPC. The necessary RPC
   endpoints must exist and return comparable data structures. Are the current
   `pallet-game-solver` RPCs sufficient, or do new endpoints need to be added?
