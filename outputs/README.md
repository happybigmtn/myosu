# Myosu Curated Outputs

This directory holds the durable, lane-level artifacts that Raspberry should
use for milestone evaluation.

These files are not Fabro's internal run state. They are the curated
deliverables that survive across runs and are meant to be reviewed by humans
and consumed by the control plane.

Current bootstrap convention:

- each active lane owns one output root
- `spec.md` captures the current lane contract and next slices
- `review.md` captures trust assessment and keep/reopen/reset judgments

Implementation-lane convention:

- `implementation.md` records the concrete slice the lane changed
- `verification.md` records the proof result, residual risks, and next slice

Fabro run branches and `.fabro` runtime state remain execution-plane details.
`outputs/` is the first durable control-plane artifact surface.

## Current Bootstrap Roots

The currently promoted bootstrap program is
`fabro/programs/myosu-bootstrap.yaml`. Its output roots are:

- `outputs/games/traits/`
- `outputs/tui/shell/`
- `outputs/chain/runtime/`
- `outputs/chain/pallet/`

Those four roots are the current bootstrap-first deliverables referenced by the
live operator docs.

## Secondary Roots

Other roots under `outputs/` are still meaningful curated artifacts, but they
belong to broader secondary program manifests or follow-on work rather than the
current bootstrap entrypoint. Examples:

- `outputs/play/tui/`
- `outputs/agent/experience/`
- `outputs/games/poker-engine/`
- `outputs/games/multi-game/`
- `outputs/sdk/core/`
- `outputs/miner/service/`
- `outputs/validator/oracle/`
- `outputs/security/audit/`
- `outputs/operations/scorecard/`
- `outputs/learning/improvement/`
- `outputs/strategy/planning/`

Treat those as secondary portfolio surfaces unless and until the operator
explicitly widens the promoted control plane.
