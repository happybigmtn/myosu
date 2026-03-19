# Myosu Specifications

`PLANS.md` governs executable implementation plans. This file governs the
documents that come before plans: decision specs, migration specs, and
capability specs. A spec explains what durable behavior or boundary we want and
why it matters. A plan explains how to implement the next slice of that work.

For small, local changes, a plan is often enough. For architectural changes,
repo strategy changes, migration programs, or any work that introduces a new
stable boundary, write a spec first and then write a plan for the first
implementation slice.

## Relationship to `PLANS.md`

Use a spec when the main problem is choosing or preserving a system boundary.
Use an ExecPlan when the main problem is carrying out a bounded change. Large
programs often need both:

- the spec captures the durable decision, target architecture, invariants, and
  migration phases
- the plan captures the current implementation slice and stays live while work
  proceeds

If the work would still be understandable six months from now without the
implementation checklist, it belongs in a spec. If the work would become stale
unless someone updates progress, discoveries, and next steps as they code, it
belongs in a plan.

## Spec Types

### Decision Spec

Use this when choosing an architectural direction, ownership boundary, or repo
strategy.

Good examples:

- adopting Fabro-first as the primary execution substrate
- deciding whether a capability belongs in generic Fabro core or in a
  Myosu-specific operational layer
- choosing whether a migration is additive, bridging, or cut-over

Required sections:

- title and status
- decision
- why now
- alternatives considered
- consequences
- what is now superseded

Decision specs should be short and decisive. They are not living checklists.

### Migration / Port Spec

Use this when moving a capability from one system, repo, or architecture into
another.

Required sections:

- title and status
- purpose / user-visible outcome
- whole-system goal
- current state
- target architecture
- what ports directly
- what ports selectively
- what does not port
- transitional bridges
- migration phases
- parity gates
- non-goals

Migration specs define what survives the move and what should be deleted rather
than preserved.

### Capability Spec

Use this when adding or changing a durable capability inside the current
architecture.

Required sections:

- title and status
- purpose / user-visible outcome
- whole-system goal
- scope
- current state
- architecture / runtime contract
- adoption path
- acceptance criteria
- failure handling

Capability specs can be followed by one or more ExecPlans.

## Writing Rules

Every spec must be self-contained for a reader who only has this repository.
Do not rely on chat history. Do not rely on another repository being checked
out nearby. If a migration originates in another repo, describe the source
concepts in plain language here rather than pointing to file paths that do not
exist in this tree.

Define terms of art immediately. If you use a phrase like "control plane",
"lane", "milestone", or "draft artifact", say what it means in this
repository. Name the first concrete consumer of every new durable surface.
Describe what becomes possible after the spec lands, not just which files
change.

Prefer prose over giant taxonomies. Use lists when they clarify the port
inventory, migration phases, or non-goals. Use tables only when they compress
important comparisons.

## Acceptance Criteria

Specs should carry acceptance criteria only when they describe a durable
capability or migration phase that can be proven. Keep them outcome-shaped:

- what someone can run
- what behavior they should see
- what parity gate proves the old path can freeze

Do not turn a spec into a second copy of an ExecPlan. Leave day-to-day progress
tracking to the plan.

## Supersession

When a new spec replaces the repo strategy of an older one, say so explicitly
in the header or first paragraphs. The old spec may remain as historical
context, but the new spec becomes the document that future plans should follow.

## Default Workflow

For any non-trivial change, use this sequence:

1. Write or update a decision spec if the architecture or ownership boundary is
   changing.
2. Write or update a migration or capability spec that defines the durable
   target.
3. Write the first ExecPlan in `plans/` for the next implementation slice.
4. Keep the plan live while coding. Update the spec only when the durable
   decision or target architecture changes.

## Canonical Locations

- specs live in `specs/`
- executable plans live in `plans/`
- this guide lives at `SPEC.md`
- execution-plan rules live at `PLANS.md`

If a contributor is unsure whether they need a spec, the bias should be:

- plan only for small local work
- spec plus plan for migration, architectural, or multi-slice work
