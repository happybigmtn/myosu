# Fabro Run Configs

This directory defines the checked-in run-config families that pair with the
Fabro workflow families. These files are part of the execution substrate, but
they are not all equally promoted in the live operator loop.

## Classification

| Family | Status | Why |
|---|---|---|
| `bootstrap/` | active | These configs power the promoted `myosu-bootstrap.yaml` entrypoint. |
| `implement/` | secondary | Direct implementation support beyond the promoted bootstrap loop. |
| `services/` | secondary | Miner/validator service bring-up beyond the node-owned proof. |
| `review-promote/` | secondary | Broader portfolio promotion surfaces after bootstrap. |
| `maintenance/` | secondary | Recurring evidence and supervision surfaces. |
| `planning/` | secondary | Planning-only surfaces, not current operator entrypoints. |

No checked-in run-config family is currently labeled experimental or
historical. The present distinction is promotion level, not validity.

## Active Family

`bootstrap/` is the only active run-config family in the promoted operator
loop today.

Current bootstrap configs:
- `bootstrap/game-traits.toml`
- `bootstrap/tui-shell.toml`
- `bootstrap/chain-runtime-restart.toml`
- `bootstrap/chain-pallet-restart.toml`

## Secondary Families

- `implement/`
  Secondary. Example: `implement/game-traits.toml`
- `services/`
  Secondary. Example: `services/miner-service.toml`
- `review-promote/`
  Secondary. Example: `review-promote/play-tui.toml`
- `maintenance/`
  Secondary. Example: `maintenance/security-audit.toml`
- `planning/`
  Secondary. Example: `planning/architecture.toml`

## Deletion Policy

No checked-in run-config family is currently misleading enough to delete.
Classification is the right move for this phase; archive or delete only after a
later pass proves a family is no longer defensible even as a secondary surface.
