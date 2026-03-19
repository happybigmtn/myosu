# Myosu Raspberry Programs

This directory holds Raspberry program manifests for Myosu.

Each manifest supervises one coherent control-plane frontier. The goal is to
avoid one giant manifest that mixes bootstrap work, chain restart work,
services, product surfaces, and recurring oversight into a single operator
surface.

## Current Programs

### `myosu-bootstrap.yaml`

Purpose: narrow bootstrap foothold.

Owns:
- `games:traits`
- `tui:shell`
- `chain:runtime`
- `chain:pallet`

Use when:
- producing the first curated lane artifacts
- proving the Fabro/Raspberry control split
- supervising the first trusted and restart lanes

### `myosu-games-traits-implementation.yaml`

Purpose: lane-scoped delivery program for the next approved `games:traits`
implementation slice.

Owns:
- `games:traits-implement`

Use when:
- `outputs/games/traits/spec.md` and `review.md` already exist
- you want a real implement/fix/verify loop over `crates/myosu-games/`

### `myosu-chain-core.yaml`

Purpose: chain restart frontier beyond bootstrap.

Owns:
- `chain:runtime`
- `chain:pallet`

Use when:
- the bootstrap surface is too broad for chain restart decisions
- you want a dedicated chain-focused control plane

### `myosu-services.yaml`

Purpose: service bringup frontier.

Owns:
- `miner:service`
- `validator:oracle`

Use when:
- chain restart work has produced enough reviewed artifacts to unblock service
  lane contracts
- you want a dedicated service-focused control plane rather than widening the
  bootstrap manifest

### `myosu-product.yaml`

Purpose: product-surface frontier.

Owns:
- `play:tui`
- `agent:experience`

Use when:
- trusted product-facing contract lanes should be supervised separately from
  bootstrap and chain restart work
- the product frontier should grow toward future play APIs and spectator lanes

### `myosu-platform.yaml`

Purpose: reusable engine and SDK frontier.

Owns:
- `games:poker-engine`
- `games:multi-game`
- `sdk:core`

Use when:
- game-engine and SDK surfaces should be supervised separately from bootstrap,
  chain restart, services, and product lanes
- the platform frontier should grow toward upstream sync and broader expansion

### `myosu-recurring.yaml`

Purpose: recurring oversight frontier.

Owns:
- `strategy:planning`
- `security:audit`
- `operations:scorecard`
- `learning:improvement`

Use when:
- recurring doctrine, audit, scorecard, and improvement lanes should be
  supervised separately from bounded delivery frontiers
- you want a dedicated recurring oversight control plane

## Planned Programs

The current frontier map is now fully seeded at the manifest level:

- `myosu-bootstrap.yaml`
- `myosu-chain-core.yaml`
- `myosu-services.yaml`
- `myosu-product.yaml`
- `myosu-platform.yaml`
- `myosu-recurring.yaml`

## Current Cross-Program Practical Rule

Raspberry does not yet have first-class manifest-to-manifest dependency
resolution. Today, cross-program readiness should be expressed through shared
output artifacts and lane checks.

Example:
- `myosu-services.yaml` waits on `outputs/chain/runtime/review.md` and
  `outputs/chain/pallet/review.md` instead of depending directly on another
  manifest object.

## Shared Rules

### Outputs

All programs share the same `outputs/` tree. Output ownership is lane-centric,
not program-centric.

Examples:
- `outputs/games/traits/`
- `outputs/tui/shell/`
- `outputs/chain/runtime/`
- `outputs/chain/pallet/`

Programs may supervise different frontiers over the same broad subsystem, but
the lane that owns an output root should stay stable.

### Milestones

Milestones are artifact-backed lifecycle checkpoints.

Current naming conventions:
- `specified` for the first accepted lane contract
- `reviewed` for spec + review artifact completion
- `implemented` for implementation artifact completion
- `verified` for implementation + verification artifact completion
- `*_reviewed` for multi-lane units where milestone names must stay lane-specific
- `service_ready` for durable service bringup
- `launch_ready` for orchestration gates

Use specific milestone names when one unit contains multiple lanes that each
produce distinct artifact contracts.

### Cross-program dependencies

Cross-program dependencies should be expressed through shared outputs and stable
milestone semantics, not through ad hoc assumptions.

Practical rule:
- if one lane depends on another lane's artifacts, model that dependency with
  the same unit/lane/milestone vocabulary in the consuming program
- avoid copying artifacts into a second output root just to satisfy a second
  program

### Frontier design

A new program manifest should exist only when the supervised frontier is
coherent to an operator. Good reasons to split a program:

- it has a different lane family mix
- it has different proof cadence or review cadence
- it would overload the current manifest with unrelated units
- it deserves a dedicated `raspberry plan/status/execute` surface
