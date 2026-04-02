# 008 - Multi-Node Devnet

## Purpose / Big Picture

Stage-0 runs on a single local node. Stage-1 requires a persistent multi-node
devnet where independent operators run miners and validators. This plan builds
devnet configuration, bootnode setup, and networking infrastructure.

## Context and Orientation

Current state:
- Chain builds and produces blocks locally (single node)
- Chain specs for `devnet` and `test_finney` are placeholder quality
- Operator bundle assumes local-only operation
- No bootnode, no persistent state, no peer discovery

## Architecture

```
Bootnode (persistent)
    ├── devnet chain spec (genesis config, authorities)
    ├── P2P networking (libp2p)
    └── RPC endpoint (WebSocket)

Operator A                    Operator B
├── myosu-chain (full node)   ├── myosu-chain (full node)
├── myosu-miner               ├── myosu-miner
└── myosu-validator            └── myosu-validator
```

## Progress

- [x] (pre-satisfied) M1. Named network support in chain binary
  - Surfaces: `crates/myosu-chain/node/src/chain_spec/`
Proof command: `SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain --features fast-runtime -- build-spec --chain devnet > /dev/null`

### Milestone 2: Production-quality devnet chain spec

- [ ] M2. Write devnet chain spec with proper genesis config
  - Surfaces: `crates/myosu-chain/node/src/chain_spec/devnet.rs`
  - What exists after: Devnet spec with initial authorities, pre-funded
    accounts, subnet 7 pre-registered, bootnodes.
  - Why now: Multi-node requires real genesis configuration.
Proof command: `SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain --features fast-runtime -- build-spec --chain devnet | jq '.genesis'`
  - Tests: Chain spec parses and contains expected genesis

### Milestone 3: Bootnode deployment

- [ ] M3. Script to deploy persistent bootnode
  - Surfaces: `ops/bootnode/deploy.sh` (new), `ops/bootnode/README.md` (new)
  - What exists after: Deploy script, persistent storage, P2P and RPC ports.
  - Why now: Multi-node requires a bootnode.
Proof command: `bash ops/bootnode/deploy.sh --dry-run`
  - Tests: Dry-run completes

### Milestone 4: Two-node sync test

- [ ] M4. Test that two nodes discover each other and sync
  - Surfaces: `tests/e2e/multi_node.sh` (new)
  - What exists after: Script starts two nodes, verifies peer discovery and
    block sync.
  - Why now: Must verify networking before operator onboarding.
Proof command: `bash tests/e2e/multi_node.sh`
  - Tests: Both nodes reach height > 5 with matching hashes

### Milestone 5: Operator bundle multi-node support

- [ ] M5. Update operator bundle for devnet connection
  - Surfaces: `.github/scripts/prepare_operator_network_bundle.sh`
  - What exists after: Bundle includes bootnode addresses, `--bootnodes` flag.
  - Why now: Operators need to connect to devnet.
Proof command: `bash .github/scripts/check_operator_network_bootstrap.sh`
  - Tests: Bundle verification passes

## Surprises & Discoveries

- Substrate's libp2p networking is battle-tested but the subtensor fork may
  have custom networking code. Verify peer discovery with forked code.
- `fast-runtime` reduces block time; devnet may want normal block time.

## Decision Log

- Decision: Single persistent bootnode (not HA).
  - Why: 2--5 operators at stage-1. HA adds complexity for zero value.
  - Failure mode: Bootnode down, network stalls.
  - Mitigation: Document manual restart. Second bootnode in stage-2.
  - Reversible: yes

## Validation and Acceptance

1. Two nodes sync blocks locally.
2. Devnet chain spec has proper genesis config.
3. Operator bundle supports multi-node.

## Outcomes & Retrospective
_Updated after milestones complete._
