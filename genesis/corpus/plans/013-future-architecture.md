# 013 - Future Architecture and Governance

## Purpose / Big Picture

As myosu transitions from stage-0 to stage-1 and beyond, architectural decisions
made now will constrain future options. This plan captures the architectural
roadmap beyond stage-1 and establishes a governance process for making
irreversible decisions.

## Context and Orientation

Key architectural questions for post-stage-1:
- When to add a second token (Alpha/TAO dual-token model)?
- When to enable EVM/smart contracts for third-party game developers?
- How to handle chain upgrades with live operators?
- When to move from single-subnet to multi-subnet with cross-subnet emissions?
- How to govern protocol changes (on-chain governance vs. operator consensus)?

Current decisions documented in `THEORY.MD`:
- Single-token model chosen for stage-0 simplicity
- EVM stripped but source preserved behind feature flag (per plan 003)
- Commit-reveal v2 (hash-based) only; v3 (timelock) deferred

## Architecture

```
Stage-0 (now)    Stage-1 (devnet)    Stage-2 (public)
Single token     Single token        Dual token?
1 subnet         2-3 subnets         N subnets
Local only       2-5 operators       Open registration
No governance    Operator consensus  On-chain governance?
```

## Progress

### Milestone 1: Architecture decision record template

- [ ] M1. Create ADR template and process
  - Surfaces: `docs/adr/000-template.md` (new), `docs/adr/README.md` (new)
  - What exists after: Template for recording architectural decisions with
    context, options, consequences. Process for proposing and approving ADRs.
  - Why now: Decisions made in stage-0 are becoming load-bearing. Must record
    rationale before it is lost.
Proof command: `test -s docs/adr/000-template.md`
  - Tests: Template exists and is non-empty

### Milestone 2: Record existing architectural decisions

- [ ] M2. Document decisions already made as ADRs
  - Surfaces: `docs/adr/001-*.md` through `docs/adr/NNN-*.md` (new)
  - What exists after: ADRs for: single-token model, Substrate fork, robopoker
    fork, enum dispatch, SwapInterface stub, commit-reveal v2, checkpoint
    versioning.
  - Why now: These decisions exist in THEORY.MD prose but are not structured for
    reference. New contributors need to find them quickly.
Proof command: `ls docs/adr/0*.md | wc -l` returns >= 7
  - Tests: Each ADR has context, decision, and consequences sections

### Milestone 3: Draft stage-2 architecture roadmap

- [ ] M3. Write roadmap for dual-token, multi-subnet, and governance
  - Surfaces: `docs/adr/stage-2-roadmap.md` (new)
  - What exists after: Document describing when and how to transition to
    dual-token, multi-subnet, and on-chain governance. Includes prerequisites,
    risks, and reversibility assessment for each transition.
  - Why now: Operators and investors need to see the path beyond stage-1.
Proof command: `test -s docs/adr/stage-2-roadmap.md`
  - Tests: Document exists with section for each major transition

## Surprises & Discoveries

- The `THEORY.MD` file is 97K lines and contains most architectural rationale
  in narrative form. Extracting structured ADRs from it will be the primary work.
- Some decisions (like the subtensor fork) may not be reversible. ADRs should
  clearly mark irreversibility.

## Decision Log

- Decision: ADR format (not RFC or spec) for architectural decisions.
  - Why: ADRs are lightweight, have established tooling, and focus on recording
    decisions rather than proposing processes. Good fit for a small team.
  - Failure mode: ADRs pile up without being maintained.
  - Mitigation: Link ADRs from relevant code files. Review ADRs quarterly.
  - Reversible: yes

## Validation and Acceptance

1. ADR template and process exist.
2. >= 7 existing decisions recorded as ADRs.
3. Stage-2 roadmap drafted with prerequisites and risks.

## Outcomes & Retrospective
_Updated after milestones complete._
