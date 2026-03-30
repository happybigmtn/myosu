# Fabro/Raspberry Bootstrap Control Plane Cutover

Status: Completed locally on 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The repo already has a real bootstrap control plane: `fabro/programs/myosu-bootstrap.yaml`, curated `outputs/`, and Raspberry commands that supervise those lanes. The problem is not absence; the problem is drift and mixed messaging. This plan makes the bootstrap manifest the unambiguous active entrypoint and narrows the surrounding doctrine to match it.

After this plan, all live supervisory docs point first to the bootstrap manifest and its current lanes. Broader program manifests may still exist, but they will be clearly marked as secondary or planned rather than implied as current operator truth.

## Progress

- [x] (2026-03-28) Confirmed that the bootstrap manifest exists and matches the narrow lane set described in `AGENTS.md`.
- [x] (2026-03-30) Confirmed the bootstrap-first story is now aligned across
  `README.md`, `OS.md`, `AGENTS.md`, and the Genesis docs that define the
  doctrine stack.
- [x] (2026-03-30) Marked the non-bootstrap Fabro program manifests as
  secondary in `fabro/programs/README.md`, with `myosu-bootstrap.yaml` called
  out as the only promoted operational entrypoint.
- [x] (2026-03-30) Rewrote `outputs/README.md` so the promoted bootstrap roots
  are explicit and the broader portfolio roots are labeled as secondary rather
  than implied as equally current.

## Surprises & Discoveries

- Observation: The generated corpus acknowledged Fabro/Raspberry, but still demoted parts of the control-plane work as if it were not current.
  Evidence: The dropped synthesized plan for Raspberry program decomposition framed that work as premature despite the repo already treating `fabro/programs/` as active doctrine.
- Observation: The repo has multiple program manifests today, but only one is clearly current.
  Evidence: `fabro/programs/myosu-bootstrap.yaml` is the current bootstrap entrypoint; the other program manifests exist but are not equally trusted.

## Decision Log

- Decision: `myosu-bootstrap.yaml` is the only promoted operational entrypoint until an explicit operator decision broadens the live program set.
  Rationale: `AGENTS.md` already says not to widen the bootstrap manifest until doctrine cutover is complete.
  Date/Author: 2026-03-28 / Codex

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Doc alignment | Multiple manifests still look equally current | Add explicit active/secondary labels in manifest README and doctrine docs |
| Output promotion | `outputs/` roots imply lanes that are not active | Distinguish bootstrap outputs from secondary outputs in docs |

## Outcomes & Retrospective

The useful `016` move was to avoid inventing more metadata and instead make the
existing checked-in control plane speak more clearly. `myosu-bootstrap.yaml`
was already the real entrypoint; the missing piece was explicit labeling around
the broader manifest set and the broader `outputs/` tree.

## Context and Orientation

Primary live surface:
- `fabro/programs/myosu-bootstrap.yaml`

Secondary surfaces that must be classified:
- `fabro/programs/myosu.yaml`
- `fabro/programs/myosu-chain-core.yaml`
- `fabro/programs/myosu-product.yaml`
- `fabro/programs/myosu-platform.yaml`
- `fabro/programs/myosu-services.yaml`
- `fabro/programs/myosu-recurring.yaml`

## Milestones

### Milestone 1: Bootstrap-first docs

Rewrite doctrine and operator docs so they point first to `fabro/programs/myosu-bootstrap.yaml`.

Proof command:

    rg -n "myosu-bootstrap" README.md OS.md AGENTS.md genesis fabro/programs/README.md

### Milestone 2: Secondary manifest labeling

Mark the non-bootstrap manifests as secondary, planned, or historical so readers do not confuse them with the active loop.

Proof command:

    sed -n '1,200p' fabro/programs/README.md

### Milestone 3: Output-root truth

Explain which `outputs/` roots are current bootstrap deliverables and which are broader portfolio or future surfaces.

Proof command:

    find outputs -maxdepth 2 -type d | sort
    rg -n "outputs/" README.md OS.md AGENTS.md fabro/programs/README.md genesis || true

## Plan of Work

1. Align bootstrap references.
2. Label secondary manifests.
3. Sync `outputs/` story to the bootstrap lanes.

## Concrete Steps

From `/home/r/coding/myosu`:

    ls fabro/programs
    sed -n '1,160p' fabro/programs/myosu-bootstrap.yaml
    find outputs -maxdepth 2 -type d | sort

## Validation and Acceptance

Accepted when:
- The bootstrap manifest is the unambiguous primary entrypoint in all live docs.
- Secondary manifests are clearly labeled.
- `outputs/` no longer implies a broader current lane set than the bootstrap program supports.

## Idempotence and Recovery

This is documentation and metadata alignment work. If a manifest classification changes later, update the labels rather than reintroducing ambiguity.

## Interfaces and Dependencies

Depends on: 014 and 015.
Blocks: 017 and 018.

```text
myosu-bootstrap.yaml
      |
      +--> live docs
      +--> outputs/ bootstrap roots
      `--> secondary manifests labeled clearly
```
