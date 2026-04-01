# Fabro Workflow Library

This library defines the workflow families referenced by the Genesis plans. The
files here are checked-in contracts, not legacy executor prompts.

## Classification

| Family | Status | Why |
|---|---|---|
| `bootstrap/` | active | This is the current promoted control-plane family used by `myosu-bootstrap.yaml`. |
| `implement/` | secondary | Useful for direct feature work, but not part of the promoted bootstrap loop. |
| `services/` | secondary | Useful for miner/validator follow-on work beyond bootstrap proof. |
| `review-promote/` | secondary | Useful for broader portfolio promotion after a lane already exists. |
| `maintenance/` | secondary | Useful recurring evidence surfaces, but not a primary operator entrypoint today. |
| `planning/` | secondary | Useful for plan generation, not part of the promoted bootstrap loop. |

No workflow family is currently labeled experimental or historical. The current
question is promotion level, not legitimacy.

## Bootstrap

Use [bootstrap.fabro](/home/r/coding/myosu/fabro/workflows/bootstrap/bootstrap.fabro)
when a lane must inventory current source, review existing curated outputs, and
refresh stale artifacts before the lane can be trusted.

Status: active.
Example: [bootstrap.fabro](/home/r/coding/myosu/fabro/workflows/bootstrap/bootstrap.fabro)
paired with `fabro/run-configs/bootstrap/chain-runtime-restart.toml`.

## Implement

Use [workflow.fabro](/home/r/coding/myosu/fabro/workflows/implement/workflow.fabro)
for direct feature or bugfix work that must move through spec, implementation,
verification, and promotion.

Status: secondary.
Example: [workflow.fabro](/home/r/coding/myosu/fabro/workflows/implement/workflow.fabro)
paired with `fabro/run-configs/implement/game-traits.toml`.

## Services

Use [workflow.fabro](/home/r/coding/myosu/fabro/workflows/services/workflow.fabro)
for miner and validator surfaces where bring-up and integration verification are
the main concerns.

Status: secondary.
Example: [workflow.fabro](/home/r/coding/myosu/fabro/workflows/services/workflow.fabro)
paired with `fabro/run-configs/services/miner-service.toml`.

## Maintenance

Use [workflow.fabro](/home/r/coding/myosu/fabro/workflows/maintenance/workflow.fabro)
for recurring security, operations, learning, and strategy surfaces that update
evidence rather than ship product features.

Status: secondary.
Example: [workflow.fabro](/home/r/coding/myosu/fabro/workflows/maintenance/workflow.fabro)
paired with `fabro/run-configs/maintenance/security-audit.toml`.

## Planning

Use [workflow.fabro](/home/r/coding/myosu/fabro/workflows/planning/workflow.fabro)
when the output is an implementation-ready plan or dependency map rather than
code.

Status: secondary.
Example: [workflow.fabro](/home/r/coding/myosu/fabro/workflows/planning/workflow.fabro)
paired with `fabro/run-configs/planning/architecture.toml`.

## Review Promote

Use [workflow.fabro](/home/r/coding/myosu/fabro/workflows/review-promote/workflow.fabro)
when a lane mainly needs verification and artifact promotion, not new feature
generation.

Status: secondary.
Example: [workflow.fabro](/home/r/coding/myosu/fabro/workflows/review-promote/workflow.fabro)
paired with `fabro/run-configs/review-promote/play-tui.toml`.

## Deletion Policy

No checked-in workflow family is misleading enough to delete right now.
Classification is sufficient for this phase; delete or archive only after a
future pass proves a family is not defensible even as a secondary surface.
