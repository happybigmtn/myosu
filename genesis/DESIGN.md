# Myosu Design

Generated: 2026-04-07
Source of truth: trunk @ 4e0b37f

---

## User-Facing Surfaces

Myosu has four distinct user-facing surfaces today, each at a different maturity level.

### 1. Operator CLI Surface

**Maturity: Functional, docs exist, packaging missing**

Operators interact with myosu through five binaries:

```
myosu-chain     — Substrate node (block production, RPC, consensus)
myosu-miner     — Strategy training and serving
myosu-validator  — Quality scoring and weight submission
myosu-play      — Gameplay against trained bots
myosu-keys      — Cryptographic key management
```

Each binary produces structured operator-facing reports to stdout:

```
MINER myosu-miner bootstrap ok
chain_endpoint=ws://127.0.0.1:9944
subnet=7
peers=0
...
```

This report format is a stable contract — tests assert against the prefix strings. New operators can parse stdout to confirm each lifecycle step succeeded.

**Current friction:** No pre-built binaries. No Docker images. Operators must compile from source with Rust nightly, WASM target, and protoc installed. Cold compilation takes 10-30 minutes depending on machine.

### 2. TUI Gameplay Surface

**Maturity: Functional, multi-game**

The TUI (`myosu-play train` or `myosu-play`) launches a ratatui-based terminal interface:

```
┌─ Game Selection ────────────────────────────┐
│ Poker (NLHE Heads-Up)                       │
│ Kuhn Poker                                  │
│ Liar's Dice                                 │
└─────────────────────────────────────────────┘
```

**Architecture:**
- `myosu-tui` crate owns shell state, screen management, input, rendering, themes
- Game-specific renderers implement `GameRenderer` trait per game
- Blueprint files provide bot strategy (mmap'd, <1us lookup)
- Pipe mode (`myosu-play pipe`) exposes structured JSON for agent consumption

**Screen state machine:**
```
Welcome → GameSelect → Playing → HandResult → (next hand or quit)
```

The TUI uses crossterm for terminal events and ratatui for rendering. Theme tokens provide consistent styling. Shell state tracks game state, hand history, and player decisions.

**Design decision:** Solver advisor (showing optimal action distributions during play) is intended ON by default in training mode, OFF by default in chain mode (to protect miner strategy privacy). The TUI shell infrastructure supports this but the advisor integration is not independently verified as working end-to-end.

### 3. Pipe / Agent Surface

**Maturity: Functional, protocol stable**

The pipe surface (`myosu-play pipe`) accepts newline-delimited commands on stdin and produces structured JSON on stdout:

```
→ stdin:  deal
← stdout: {"type":"deal","cards":["Ah","Ks"],"pot":3,"stack":98}
→ stdin:  raise 10
← stdout: {"type":"action","player":"hero","action":"raise","amount":10}
← stdout: {"type":"action","player":"villain","action":"call","amount":10}
...
→ stdin:  quit
```

This surface is designed for AI agents and automated testing. The smoke test (`--smoke-test`) exercises this protocol end-to-end.

**Design constraint:** Pipe output must be parseable by a stateless consumer. Each line is a complete JSON object. No multi-line messages. No binary framing.

### 4. Chain RPC Surface

**Maturity: Functional, Substrate-standard**

The chain node exposes standard Substrate JSON-RPC on port 9944 (configurable). Custom RPC methods are inherited from subtensor:

- `neuronInfo_getNeuronsLite` — lightweight neuron info for all UIDs
- `subnetInfo_getSubnetInfo` — subnet metadata
- `subnetInfo_getSubnetHyperparams` — subnet configuration
- `stakeInfo_getStakeInfoForColdkey` — stake queries
- `delegateInfo_getDelegates` — delegation info

The `myosu-chain-client` crate wraps these into typed Rust methods, but operators can also use any Substrate-compatible client (polkadot.js, subxt).

## Information Architecture

### What operators need to know at each stage

```
T0 (First contact)
├── README.md → "What is this?"
├── docs/operator-guide/quickstart.md → "How do I run it?"
└── docs/operator-guide/architecture.md → "How do the pieces fit?"

T1 (Running a node)
├── myosu-chain --dev → Single authority
├── Build-spec for devnet/testnet → Named multi-authority
└── docs/operator-guide/troubleshooting.md → "What went wrong?"

T2 (Running miner/validator)
├── myosu-keys create → Generate identity
├── myosu-miner → Register, train, serve
├── myosu-validator → Score, submit weights
└── CHANGELOG.md → "What changed since last release?"

T3 (Understanding the protocol)
├── INVARIANTS.md → Hard rules
├── specs/ → Design decisions
├── docs/adr/ → Architecture decision records
└── AGENTS.md → System architecture
```

### Documentation gaps in the current T0 path

1. README does not list prerequisites (Rust, WASM target, protoc)
2. README references `fabro run` commands that don't work
3. The fastest meaningful test (`cargo test -p myosu-games-kuhn`) is not documented
4. No "hello world" path that works in under 60 seconds

## Visual Design

The TUI uses a simple theme system (`myosu-tui/src/theme.rs`):

- Consistent color palette across all game renderers
- Card rendering with suit symbols (unicode)
- Box-drawing characters for table layout
- Status bar with game state and hand counter

No web frontend exists. No design system beyond TUI theme tokens.

## Key Interaction Flows

### Miner Registration and Training

```
Operator                    Chain                   Miner Binary
   │                          │                         │
   ├─ myosu-keys create ──────┤                         │
   │  (generates keypair)     │                         │
   │                          │                         │
   ├─ myosu-miner ────────────┤                         │
   │                          │◄── probe (system_health)│
   │                          │──► health response      │
   │                          │                         │
   │                          │◄── register (burned_register)
   │                          │──► uid assigned         │
   │                          │                         │
   │                          │◄── serve_axon (set_axon_info)
   │                          │──► axon published       │
   │                          │                         │
   │                          │         [training loop] │
   │                          │         │ load encoder  │
   │                          │         │ MCCFR step()  │
   │                          │         │ checkpoint()  │
   │                          │         └───────────────│
   │                          │                         │
   │◄─── TRAINING report ─────┤                         │
   │   (epochs, exploitability)                         │
```

### Validator Scoring and Weight Submission

```
Validator Binary            Chain              Miner (via file or HTTP)
   │                          │                         │
   ├── probe ─────────────────┤                         │
   │                          │                         │
   ├── register + stake ──────┤                         │
   │                          │                         │
   ├── query miner ───────────┤─────────────────────────►│
   │                          │                         │
   │◄─ strategy response ─────┤◄────────────────────────│
   │                          │                         │
   │  [local scoring]         │                         │
   │  load checkpoint         │                         │
   │  decode query/response   │                         │
   │  compute expected dist.  │                         │
   │  L1 distance → score     │                         │
   │                          │                         │
   ├── submit weights ────────┤                         │
   │                          │                         │
   │◄── VALIDATION report ────│                         │
```

### Emission Distribution

```
Block N authored
   │
   ├── on_finalize()
   │   ├── run_coinbase(block_emission)
   │   │   ├── get_subnets_to_emit_to()
   │   │   ├── get_subnet_block_emissions()     ← split by weight
   │   │   ├── emit_to_subnets()                ← accumulate PendingEmission
   │   │   ├── drain_pending()                  ← if tempo fires
   │   │   │   ├── epoch() or epoch_dense()     ← Yuma Consensus
   │   │   │   │   ├── compute incentives
   │   │   │   │   ├── compute dividends
   │   │   │   │   └── compute emissions
   │   │   │   ├── owner_cut
   │   │   │   ├── server_distribution          ← to miners by incentive
   │   │   │   └── validator_distribution       ← to validators by dividend
   │   │   └── increase TotalIssuance
   │   └── done
   │
   Block N+1 authored...
```

## Crate Dependency Graph

```
myosu-games (traits, GameType, registry)
   ├── myosu-games-poker (robopoker wrapper)
   ├── myosu-games-kuhn (native MCCFR)
   └── myosu-games-liars-dice (native MCCFR)

myosu-chain-client (RPC wrapper)
   └── depends on: runtime-common types

myosu-miner
   ├── myosu-games, myosu-games-poker, myosu-games-kuhn, myosu-games-liars-dice
   ├── myosu-chain-client
   └── myosu-keys

myosu-validator
   ├── myosu-games, myosu-games-poker, myosu-games-kuhn, myosu-games-liars-dice
   ├── myosu-chain-client
   └── myosu-keys

myosu-play (gameplay)     ← NO dependency on myosu-miner (INV-004)
   ├── myosu-games, myosu-games-poker, myosu-games-kuhn, myosu-games-liars-dice
   └── myosu-tui

myosu-tui (shell, rendering)
   ├── ratatui, crossterm
   └── myosu-games (traits only)

myosu-keys (key management)
   └── sp-core, sp-keyring (Substrate crypto)
```

## Design Decisions Worth Noting

| Decision | Rationale |
|----------|-----------|
| Enum dispatch, not trait objects | CFR traits require Copy+Sized. No dyn dispatch possible. `GameType` enum switches at runtime boundaries. |
| bincode for wire codec | Fast serialization. Direct dependency on unmaintained `bincode 1.x` accepted as known debt. |
| 1 MiB decode budget | Prevents memory exhaustion from malformed payloads. Hardened from original 256 MiB. |
| mmap for blueprint files | 50MB+ profiles stay on disk. Sub-microsecond lookup via page faults. |
| Hyperbolic scoring | `1/(1+d)` maps L1 distance to [0,1] score. Smooth, bounded, differentiable. |
| File-based validator path | Alongside HTTP. Supports games without HTTP axon (Liar's Dice, Kuhn). |
| Network-namespaced key storage | Prevents devnet/testnet key confusion. Each network gets its own directory. |
| Structured stdout reports | Machine-parseable operator output. Tests assert against prefix strings. |
