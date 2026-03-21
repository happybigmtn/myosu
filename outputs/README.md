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

Integration-decision convention:

- `*-adapter.md` records how already-reviewed lane artifacts should be
  consumed by the next frontier or workflow family
- `review.md` records the keep/reopen/reset judgment for that integration
  decision and states whether the next move is implementation or another
  unblock

Fabro run branches and `.fabro` runtime state remain execution-plane details.
`outputs/` is the first durable control-plane artifact surface.
