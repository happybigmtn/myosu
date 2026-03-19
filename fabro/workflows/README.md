# Myosu Fabro Workflow Library

This directory is the beginning of Myosu's Fabro workflow library.

The goal is not to force every lane through one universal graph. Different
Myosu lane types need different workflow families because they carry different
risk profiles, proof shapes, and operator expectations.

## Workflow Families

### `bootstrap/`

Purpose: narrow entry workflows that establish lane contracts and curated
artifacts with minimal orchestration.

Use for:
- initial lane specification and trust review
- proving the execution/control-plane split
- extremely small bootstrap slices where richer workflow machinery would be
  premature

Current examples:
- `bootstrap/game-traits.fabro`
- `bootstrap/tui-shell.fabro`
- `bootstrap/chain-pallet-restart.fabro`

### `implement/`

Purpose: bounded implementation loops with preflight, code change, verification,
fixup, and artifact audit.

Use for:
- trusted leaf crates
- small-to-medium code slices with a clear proof command
- lanes where the main risk is implementation quality, not architecture

Current examples:
- `implement/game-traits.fabro`

### `restart/`

Purpose: phased restart or rebuild work where an existing implementation base
is not trustworthy and the lane must move layer by layer with explicit
validation between phases.

Use for:
- chain/runtime restart work
- major subsystem reductions
- cases where "build it all in one shot" would hide architectural mistakes

Current examples:
- `restart/chain-runtime.fabro`

### Future families

These are not all checked in yet, but they are the current target library
shape:

- `conformance/` for definition-of-done and spec-audit loops
- `orchestration/` for launch/devnet and stack-level coordination

### `services/`

Purpose: service-lane contract and bringup work where readiness, health, and
proof posture matter more than a pure compile/test loop.

Use for:
- miner/validator service bootstrap
- service bringup and stabilization
- lanes that need explicit readiness and health semantics

Current examples:
- `services/miner-service.fabro`
- `services/validator-oracle.fabro`

### `maintenance/`

Purpose: recurring oversight and backlog-processing work where the lane should
continuously inspect doctrine, outputs, or upstream change streams and emit
durable review artifacts.

Use for:
- strategy/security/operations/learning lanes
- future semantic-port or upstream-sync loops
- recurring contract and audit surfaces

Current examples:
- `maintenance/strategy-planning.fabro`
- `maintenance/security-audit.fabro`
- `maintenance/operations-scorecard.fabro`
- `maintenance/learning-improvement.fabro`

## Lane-Type to Workflow-Family Map

| Myosu lane type | Recommended family | Why |
|---|---|---|
| Trusted leaf crate continuation (`games:traits`, `tui:shell`) | `implement/` | These lanes already have trusted code bases and clear proof commands. |
| Chain restart (`chain:runtime`, `chain:pallet`) | `restart/` | These lanes need phased rebuilds and explicit restart boundaries. |
| Spec-backed audit or acceptance sweep | `conformance/` | Best fit for definition-of-done or NL-spec compliance loops. |
| Long-lived service bringup (`miner:service`, `validator:oracle`) | `services/` | Needs setup, health checks, and bounded repair loops rather than a pure build flow. |
| Launch/devnet lane | `orchestration/` | Needs environment readiness, sequencing, and multi-lane coordination. |
| Upstream tracking (`robopoker`, `subtensor`) | `maintenance/` | Semantic-port style recurring backlog loop is the right shape. |
| Recurring security/ops/strategy review | `maintenance/` or `conformance/` | Depends on whether the lane is backlog-driven or checklist/audit-driven. |

## Current Guidance

- Reuse workflow families before inventing one-off graphs.
- Keep Myosu-specific logic in run configs, prompts, checks, and program
  manifests where possible.
- Prefer promoting a lane into a richer family only when its risk profile or
  proof shape clearly demands it.
