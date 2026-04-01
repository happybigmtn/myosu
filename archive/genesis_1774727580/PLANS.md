# Genesis Execution Plans

This document defines the requirements for an execution plan ("ExecPlan") within the Myosu 180-day turnaround. It is a direct copy of the root `PLANS.md` with genesis-specific conventions applied.

---

## How to Use ExecPlans and This Document

When authoring an ExecPlan, follow this document to the letter. Each ExecPlan must be fully self-contained: a novice reading only the ExecPlan file and the current working tree must be able to implement the feature end-to-end without additional context.

When implementing an ExecPlan, do not prompt for next steps. Mark the current milestone in_progress, implement it, mark it complete, and proceed to the next milestone. Keep all living sections updated.

When discussing an ExecPlan, record decisions in the Decision Log with date, rationale, and author.

---

## Required Living Sections

Every ExecPlan must maintain these four sections:

### Progress
A checklist with timestamps. Every stopping point must be documented here, even partial completions. Format:
- `- [x] (YYYY-MM-DD HH:MMZ) Description of completed step`
- `- [ ] Description of remaining step`
- `- [ ] Partial step (completed: X; remaining: Y)`

### Surprises & Discoveries
Unexpected behaviors, bugs, performance observations, or design insights discovered during implementation. Each entry must include evidence (test output, log line, compiler warning, etc.).

### Decision Log
Every non-trivial decision made while working on the plan. Format:
- `Decision: ...`
- `Rationale: ...`
- `Date/Author: YYYY-MM-DD / name`

### Outcomes & Retrospective
Written at completion of major milestones or the full plan. Summarizes what was achieved, what remains, and lessons learned.

---

## Milestone Structure

Each milestone must:
1. Describe scope and what exists at the end that did not exist before
2. Name specific files, functions, and modules by full path
3. State the exact proof command that verifies completion
4. Be independently verifiable and incrementally implement the overall goal

Milestones are narrative, not bureaucracy. Write as a story: goal, work, result, proof.

---

## Proof Command Conventions

Every milestone must include a concrete proof command. The hierarchy:

| Level | Command | What It Proves |
|-------|---------|----------------|
| **0** | `cargo check -p {crate}` | Code parses and type-checks |
| **1** | `cargo build -p {crate}` | Code compiles to artifact |
| **2** | `cargo test -p {crate}` | Unit tests pass |
| **3** | `{binary} --help` | Binary runs and produces output |
| **4** | Scenario test | Binary produces expected behavior with real inputs |

**Default to Level 2 for library crates.** Level 3-4 are required for binaries and user-facing features.

Content assertions beat presence assertions. `grep` for specific output beats `test -f` for file existence.

---

## Formatting Rules

- One ExecPlan per `.md` file in `genesis/plans/`
- Write in plain prose. Prefer sentences over lists.
- Checklists are permitted only in the `Progress` section.
- Use two newlines after every heading.
- Fenced code blocks are for commands and code snippets only — not for prose.
- Define every term of art in plain language on first use.

---

## Scope Discipline

- 3-8 milestones per plan. More = split the plan.
- Name specific files. No "the module" — write `crates/myosu-games/src/traits.rs:GameType`.
- State what does NOT belong in this plan.
- Flag one-way doors explicitly (destructive migrations, irreversible decisions).

---

## Validation Requirements

Every milestone must state:
1. **What to run** — exact command with arguments
2. **Where to run it** — working directory
3. **What to expect** — specific output, exit code, or behavior
4. **What failure looks like** — what goes wrong if the implementation is wrong

---

## Integration With Fabro/Raspberry

When a Fabro lane is the delivery mechanism:
- Reference the program manifest: `fabro/programs/myosu-{program}.yaml`
- Reference the run config: `fabro/run-configs/{family}/{name}.toml`
- Reference the curated outputs root: `outputs/{frontier}/{lane}/`
- State which Raspberry milestones are satisfied by completing this plan

When Fabro is not the delivery mechanism (e.g., chain restart):
- State the specific cargo commands and their expected outputs
- Note any preconditions from prior plans

---

## Anti-Patterns to Avoid

- "Set up X infrastructure" as a milestone — be specific about what is set up
- Milestones that assume other plans are complete without stating the dependency
- Proof commands that only check file existence
- Plans without a Decision Log
- Plans without a failure scenario in every milestone

---

*This document is the authoritative PLANS.md for the genesis turnaround. It governs all ExecPlans under `genesis/plans/`. The root `PLANS.md` governs all other plans under `plans/`.*
