# OS Refresh and Operator Docs

Status: Completed locally on 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

`OS.md` is supposed to be the repo's live operating system, but right now it mixes durable doctrine, stage-0 ambitions, and surfaces that are not yet truly operator-ready. This plan rewrites `OS.md` so it becomes the reliable top-level handbook, then syncs `README.md` and the durable docs around it.

After this plan, a newcomer can open `OS.md` and understand the live doctrine hierarchy, the narrow Fabro/Raspberry bootstrap loop, what is current versus aspirational, and where to go next. `README.md` and durable docs will point at that same truth instead of widening the story.

## Progress

- [x] (2026-03-28) Confirmed that the generated Genesis corpus treated `OS.md` as already comprehensive even though the active objective is to build it out.
- [x] (2026-03-30) Rewrote `OS.md` around the actual stage-0 doctrine: mission,
  stage-0 exit meaning, doctrine hierarchy, the narrow Fabro/Raspberry
  bootstrap control plane, and the current operator loop now have explicit
  sections instead of being buried in a market thesis / roadmap hybrid.
- [x] (2026-03-30) Synced `README.md` to the refreshed `OS.md` story. The repo
  entrypoint now points at the operating-system document, the bootstrap
  manifest, the execution playbooks, and the current proof commands instead of
  duplicating a broader or stale operator narrative.
- [x] (2026-03-30) Added and updated durable docs for the currently runnable
  local advisor and bootstrap supervision path. The execution-playbook index
  now elevates bootstrap, local advisor, and the stage-0 local loop as the
  current truthful surfaces.
- [x] (2026-03-30) Removed or demoted documentation that implied miner,
  validator, or devnet operator workflows were already broader first-class
  products. The stage-0 loop playbook now centers the node-owned proof, and the
  services playbook is explicitly diagnostic rather than aspirational.

## Surprises & Discoveries

- Observation: The generated documentation plan assumed `OS.md` was complete enough to be treated as a reference only.
  Evidence: The generated plan described `OS.md` as "comprehensive, current" even though the current objective is to expand it.
- Observation: The repo already has a clear bootstrap control plane in `AGENTS.md`, but `OS.md` does not yet carry that same narrowness cleanly.
  Evidence: `AGENTS.md` names `fabro/programs/myosu-bootstrap.yaml` as the current bootstrap entrypoint and says not to widen it until doctrine cutover is complete.

## Decision Log

- Decision: `OS.md` is the primary document to deepen; supporting docs must conform to it rather than invent parallel narratives.
  Rationale: The operator asked to build out `OS.md`, and the repo doctrine already says `OS.md` explains how the system decides.
  Date/Author: 2026-03-28 / Codex

- Decision: Only document currently truthful operator flows as first-class runbooks.
  Rationale: `myosu-play` and the bootstrap supervision loop are real today; miner/validator and devnet operations still belong to planned work unless proven otherwise.
  Inversion: Writing polished runbooks for not-yet-runnable surfaces would recreate the same documentation drift this cleanup is trying to remove.
  Date/Author: 2026-03-28 / Codex

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| `OS.md` rewrite | Durable doctrine gets replaced with a transient status note | Keep doctrine hierarchy, mission, and current operator loop as stable sections; move ephemeral detail to Genesis or runbooks |
| README sync | README becomes a second operating system | Make README point to `OS.md` and bootstrap entrypoints instead of duplicating every rule |
| Runbook scope | Docs overstate not-yet-live services | Mark planned surfaces as planned, or omit them until proven |

## Outcomes & Retrospective

This plan closed cleanly once the repo stopped trying to make `OS.md` do every
job at once. The useful move was subtraction: trim `OS.md` down to doctrine,
operator truth, and proof surfaces; make `README.md` point at it; and make the
playbooks reflect the current node-owned and local-advisor realities instead of
generic future service stories.

## Context and Orientation

Relevant live surfaces:
- `OS.md` -- live operating system for the repo
- `AGENTS.md` -- active doctrine hierarchy plus Fabro/Raspberry execution model
- `README.md` -- newcomer entrypoint
- `fabro/programs/myosu-bootstrap.yaml` -- current bootstrap manifest
- `outputs/` -- curated lane artifacts
- `docs/` and `docs/execution-playbooks/` -- durable documentation surfaces

## Milestones

### Milestone 1: Rewrite `OS.md` around live doctrine

Rewrite `OS.md` so it clearly separates:
- live doctrine and active control-plane truth
- stage-0 exit criteria and planned architecture
- historical or deferred surfaces

Proof command:

    rg -n "myosu-bootstrap|doctrine hierarchy|historical" OS.md

### Milestone 2: Sync repo entrypoints

Update `README.md` and any durable architecture docs so they point at the refreshed `OS.md`, the bootstrap manifest, and the currently truthful operator loop.

Proof command:

    rg -n "OS.md|myosu-bootstrap|outputs/" README.md docs || true

### Milestone 3: Keep runbooks honest

Document the local advisor and bootstrap supervision flow as current. Remove or demote runbooks that present miner/validator or devnet operator stories as if they are already first-class.

Proof command:

    test -f fabro/programs/myosu-bootstrap.yaml
    rg -n "myosu-play|raspberry plan|raspberry status|raspberry execute" README.md docs docs/execution-playbooks || true

## Plan of Work

1. Rewrite `OS.md`.
2. Sync `README.md`.
3. Update durable docs and runbooks for truthful current flows only.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '244,332p' AGENTS.md
    sed -n '1,260p' OS.md
    sed -n '1,120p' README.md

## Validation and Acceptance

Accepted when:
- `OS.md` explicitly describes the live bootstrap control plane and doctrine hierarchy.
- `README.md` matches that same story.
- Durable docs do not imply a broader operator surface than the repo currently supports.

## Idempotence and Recovery

Documentation rewrites are overwrite-safe. If a statement cannot be proven from the current repo, remove it or mark it planned rather than preserving it for completeness.

## Interfaces and Dependencies

Depends on: none.
Blocks: 015 and 016, because cutover work needs a stable `OS.md` target.

```text
OS.md
  |
  +--> README.md
  +--> docs/
  `--> docs/execution-playbooks/
```
