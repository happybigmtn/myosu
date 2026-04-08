# 013 — Fabro Ghost Infrastructure Cleanup

## Objective

Resolve the discrepancy between the extensive fabro/raspberry references in AGENTS.md, OS.md, and README.md and the fact that the `fabro/` directory does not exist. Either create the minimum viable execution infrastructure or remove the references.

## Context

The repo references:
- `fabro/workflows/`
- `fabro/run-configs/bootstrap/*.toml`
- `fabro/programs/myosu-bootstrap.yaml`
- `fabro/prompts/`
- `fabro/checks/`
- `.raspberry/`

None of these paths exist. `fabro.toml` exists at root but references a MiniMax model configuration, not myosu execution infrastructure.

The AGENTS.md "Bootstrap Lanes" and "Current Expectations" sections describe a supervision model built on fabro/raspberry. The OS.md "Execution Model" section does the same.

**Decision needed:** Is the fabro/raspberry model still the intended execution substrate? If yes, create the minimum directory structure and at least one working entrypoint. If no, remove all references and use a simpler execution model (shell scripts, Makefile, or just documented cargo commands).

## Acceptance Criteria

- One of:
  - (A) `fabro/` directory exists with at least `programs/myosu-bootstrap.yaml` and one working run config, AND `fabro run <config>` actually executes something meaningful, OR
  - (B) All fabro/raspberry references in AGENTS.md, OS.md, and README.md are removed or replaced with the actual execution model (cargo commands, shell scripts, etc.)
- `fabro.toml` either points to a working configuration or is removed
- No document references a path that doesn't exist without explicitly marking it as planned

## Verification

```bash
# Option A: verify fabro works
test -d fabro && fabro run fabro/run-configs/bootstrap/game-traits.toml

# Option B: verify no ghost references
! grep -rq "fabro/" AGENTS.md OS.md README.md 2>/dev/null
```

## Dependencies

- Plan 006 (Phase 1 gate) — complete structural cleanup before deciding on execution model
