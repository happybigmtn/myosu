# Spec & Implementation Plan Authoring Guide

Status: Active
Date: 2026-03-12
Governs: `specs/*.md`, `ralph/IMPLEMENT.md`, `loop/PROMPT_build.md`, `loop/PROMPT_plan.md`

This document is the definitive format and process reference for writing specs
and implementation plan entries. Parts 1-4 define how to write them. Part 5
defines when they are required, what governs their lifecycle, and when work is
complete. Build agents consume specs and plan entries as their primary input.
Ambiguity in these documents produces wrong code.

## Fast Quality Gate

Reject the spec immediately if any answer is "no":

- Can a new engineer explain the whole-system capability without naming files?
- Does every new truth surface, report, or registry name its first real
  consumer?
- Does every AC say what larger loop or capability it unlocks?
- Is there at least one whole-loop proof for any control-loop or operator-truth
  change?
- If the spec is a refactor, reduction, or migration, does it say what stays,
  what collapses, and what must not regress?
- Could a build agent execute the work without chat history or guessing intent?

---

## Part 1: Spec Format

Specs live in `specs/`. Each spec is a self-contained build contract: planning
context (purpose, scope, current state) and executable ACs (where, how, wiring,
tests, pass/fail) in a single file. The planning prompt reads one spec and
produces IMPLEMENT.md entries from it.

### 1.1 File Naming

- Date prefix + kebab-case: `031226-profile-loader.md`, `031226-engine-adapters.md`
- Date format is MMDDYY (creation date of the spec)
- Name describes the system, not the source
- One spec per component. If too large, split by subcomponent.

### 1.2 Header Block

```
# Specification: {Title} — {Subtitle}

Source: {What motivated this spec — a brainstorm, a plan section, an incident}
Status: Draft
Date: {YYYY-MM-DD}
Depends-on: {AC prefixes this spec requires to exist first, or "none"}
```

| Field | Required | Description |
|-------|----------|-------------|
| Title | Yes | System name matching the file name |
| Subtitle | Yes | One-phrase description of what this system does |
| Source | Yes | What motivated this spec |
| Status | Yes | `Draft` until implementation begins, then `Active` |
| Date | Yes | Creation date (YYYY-MM-DD) |
| Depends-on | Yes | Explicit AC prefixes. `none` if no dependencies. |

### 1.3 Purpose

Why this work matters. What user-visible outcome it creates.

Rules:
- Lead with the outcome, not the mechanism
- State explicitly which larger product, platform, or operating goal this spec
  advances
- Answer: "if every AC in this spec lands, what becomes possible that is not
  possible today?"
- Name the primary consumer of the outcome: user, operator, brain, runtime, or
  downstream system
- Name every subsystem this spec introduces or extends
- State the key design constraint or invariant in **bold**
- Define any term a new contributor wouldn't know
- 3-8 sentences

### 1.3.1 Whole-System Goal

Every spec must state the larger system outcome it is trying to move, not just
the local component it edits.

Required format:

```
## Whole-System Goal

Current state:
- {What the system can do today}

This spec adds:
- {What this spec contributes}

If all ACs land:
- {New capability unlocked}

Still not solved here:
- {What remains for later specs}

12-month direction:
- {What more complete system this points toward}
```

Rules:
- This section is mandatory.
- If "If all ACs land" still sounds like a small local refactor rather than a
  meaningful system capability, the spec is likely sliced too narrowly.
- "Still not solved here" must name the deliberate gap so build agents do not
  mistake one slice for the whole system.

### 1.3.2 Why This Spec Exists As One Unit

Specs should be split by coherent system capability, not by whatever file or
method was easiest to isolate.

Required format:

```
## Why This Spec Exists As One Unit

- {Why these ACs belong together}
- {Why this is not merely a rounding entry}
- {What would force a split into multiple specs}
```

Rules:
- If you cannot explain why the ACs belong together, split or rewrite the spec.
- If the section reads like "these files happen to change together," the spec
  is too implementation-shaped and not outcome-shaped.
- If the spec is likely to touch more than 8 files or introduce more than 2 new
  classes/services, justify why that breadth is still one coherent capability.

### 1.4 Scope

Explicit in/out boundary.

```
## Scope

In scope:
- {concrete deliverable}

Out of scope:
- {something someone might expect, with brief rationale}
```

### 1.5 Current State

Where things stand today. Concrete file references so a build agent knows what
exists versus what needs to be created. If greenfield, say so explicitly.

This section is a terse snapshot of the gaps and active truth surfaces. Do not
repeat the full leverage map from `## What Already Exists`; that section is for
reuse/extend/replace decisions. `## Current State` should answer "what is true
right now?" while `## What Already Exists` should answer "what can we stand on
instead of rebuilding?"

```
## Current State

- `src/profile.rs` — does not exist yet
- `project.yaml` — exists with engine config, proof_commands, workspace policy
- `Cargo.toml` — has serde_yaml dependency ready
```

### 1.5.1 What Already Exists

Every spec must map the proposed work to existing code or flows that already
solve part of the problem.

Required format:

```
## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| {thing} | {path or flow} | {reuse/extend/replace} | {rationale} |
```

Rules:
- Keep this to the minimum repo-truth snapshot a build agent needs.
- Prefer 3-8 bullets.
- Focus on missing pieces, contradictory surfaces, or the exact files that are
  currently authoritative.
- If a bullet is primarily about reuse/extend/replace, move it to
  `## What Already Exists` instead of repeating it here.
- This section is mandatory.
- At least one row must mention a real file path or runtime flow.
- If every row says "new" with no existing leverage, say so explicitly and
  explain why this is truly greenfield.
- Use this section to prevent rebuilding parallel systems accidentally.

### 1.6 Non-goals

Explicit list of what this spec does NOT cover. Each non-goal names a specific
mechanic someone might expect. Include deferred features with rationale.

Required format:

```
## Non-goals

- {Expected item this spec deliberately does not do} — {why deferred}
```

Rules:
- This section is mandatory.
- Do not write generic non-goals like "future work" or "polish."
- For refactor or reduction specs, include at least one item naming the
  contracts that must not be weakened.

### 1.7 Ownership Map

```
## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Profile struct | New | src/profile.rs |
| Engine config | New | src/profile.rs |
```

**Status values**: `New` (doesn't exist), `Extend` (modifying existing),
`Refactor` (restructuring without behavior change).

### 1.7.1 Architecture / Runtime Contract

Every non-trivial spec must describe the end-to-end loop it is changing.

Required format:

```
## Architecture / Runtime Contract

{ASCII diagram showing trigger -> processing -> persisted truth -> consumer}

Primary loop:
- Trigger: {what starts the flow}
- Source of truth: {what inputs are authoritative}
- Processing: {what transforms happen}
- Persisted truth: {what durable surfaces are written}
- Consumer: {who or what reads the result}

Failure loop:
- {What fails}
- {How the system notices}
- {What is persisted}
- {Who sees it}
```

Rules:
- This section is mandatory for runtime, orchestration, brain, control-plane,
  or operator-facing specs.
- The ASCII diagram must show one happy-path loop from input to consumer.
- If the spec creates or changes a loop and the loop is not diagrammed, the
  spec is incomplete.

### 1.7.2 Adoption / Consumption Path

Many bad specs describe a new writer, registry, report, or control surface but
never prove who actually consumes it. That creates shelfware: locally precise
work that lands cleanly and still does not move the platform.

Required format:

```
## Adoption / Consumption Path

- Producer: {What writes or emits the new thing}
- First consumer: {What reads it first}
- Operator-visible surface: {CLI/TUI/health/prompt/file/log where the effect appears}
- Why it changes behavior now: {What is different immediately if this lands}
- If not consumed yet: {Exact follow-on spec/AC required before value exists}
```

Rules:
- This section is mandatory for brain, control-plane, orchestration, runtime,
  operator-surface, or repo-truth specs.
- "Future systems will use this" is not enough. Name the first real consumer or
  explicitly say the spec is incomplete without a named follow-on AC/spec.
- If the "Why it changes behavior now" line is vague or empty, the spec is
  probably too narrow or wrongly cut.

### 1.7.3 Operational Controls

Required for:
- runtime, orchestration, brain, control-plane, operator-surface, repo-truth,
  migration, reduction, or multi-phase refactor specs

Required format:

```
## Operational Controls

Phase order:
1. {first safe phase}
2. {next safe phase}

Gate rules:
- {what must be proven before the next phase}

Failure modes:
| Codepath | Realistic failure | Test needed | Error handling needed | User-visible if broken |
|----------|-------------------|-------------|-----------------------|------------------------|
| {path} | {failure} | Yes/No | Yes/No | {clear/silent/etc.} |

Diagram maintenance:
- `{file}` — {diagram to add or update if this spec changes the flow}
```

Rules:
- This section is mandatory for the spec types listed above.
- Use it to make sequencing, rollback points, and regression risks explicit.
- If the spec deletes, collapses, or replaces behavior, at least one gate rule
  must state what proves parity before removal.
- If any failure mode would be silent with no test and no error handling, the
  spec is incomplete until that gap is addressed.

### 1.8 Acceptance Criteria (ACs)

ACs are the core of the spec. Each AC is a complete implementation contract.

#### AC Naming

- Prefix matches the component: `PL-` for profile-loader, `EA-` for engine-adapters
- Sequential, zero-padded: `PL-01`, `PL-02`, ..., `PL-09`
- Prefix is 2-5 uppercase characters

#### AC Grouping

Group ACs by subsystem when a spec has many. Section separators (`---`) between
groups:

```
## A. Core Parsing

### AC-PL-01: Parse project.yaml into Profile struct
...

---

## B. Validation

### AC-PL-03: Required field validation
```

#### AC Body — Required Fields

Every AC MUST have all of the following fields. A build agent encountering a
missing field MUST reject the task.

```
### AC-{PREFIX}-{NN}: {Title}

- Where: {file paths}
- How: {Implementation description — what to build and how}
- Whole-system effect: {What larger capability, loop, or consumer this AC unlocks}
- State: {What must be persisted. Structs, fields, enums, files.}
- Wiring contract:
  - Trigger: {What invokes this mechanic}
  - Callsite: {Exact file/function that invokes execution}
  - State effect: {Deterministic in-memory/runtime delta}
  - Persistence effect: {File write / DB row / config update}
  - Observable signal: {Log line / metric / CLI output / test assertion}
- Required tests:
  - {cargo test -p <crate> <module>::tests::<test_name>}
  - {one test per critical behavior}
- Pass/fail:
  - {Concrete scenario with exact expected behavior}
  - {Another scenario}
  - {Edge case}
- Blocking note: {Why this AC exists. Design rationale. What breaks without it.}
- Rollback condition: {What invalidates this AC and triggers replanning.}
```

##### Field-by-field rules:

**Where**: Exact file paths. `(new)` for new files, `(extend)` for mods.
Multiple paths comma-separated.

**How**: The implementation description. Rules:
- Describe the algorithm or mechanism, not just the outcome
- Include formulas with exact variable names and scales
- Reference other ACs by ID when there are dependencies
- Name struct fields, enum variants, function signatures
- For refactor, migration, or reduction work, name what is kept, what is
  collapsed, and what is deleted or deferred

**Whole-system effect**: The bridge between the local change and the spec's big
goal.
- Name which loop, consumer, or downstream capability becomes possible
- If the effect is "just cleaner code," it is not specific enough
- This field is mandatory; it prevents locally precise but globally hollow ACs

**State**: What persists across restarts. Name exact fields, structs, enum
variants, file paths. If stateless (e.g., a CLI flag parser), say "No runtime
state."

**Wiring contract**: How this AC connects to the rest of the system. Mandatory
for all runtime mechanics. This prevents building isolated components that pass
unit tests but never get called.

- `Trigger`: concrete invocation source (supervisor loop step, CLI subcommand,
  file watcher event, function call from another AC)
- `Callsite`: exact file/function that invokes this logic. Must be a real path
  a build agent can verify exists (or will exist via another AC).
- `State effect`: deterministic runtime mutation that follows invocation
- `Persistence effect`: what is durably written and where
- `Observable signal`: concrete signal proving the wiring works — a log line,
  CLI output, test assertion, file written. This is what integration tests check.
- Only non-runtime work (docs, lint, CI-only) may use `N/A` with a reason

**Required tests**: Concrete test commands a build agent can run.
- Always scoped: `cargo test -p <crate> <filter>` — never workspace-level
- Test names: `<module>::tests::<descriptive_name>`
- Must include tests at two distinct levels:
  1. **Contract tests**: forced trigger → exact effect. Proves the AC in isolation.
  2. **Lifecycle tests**: trigger fires, callsite invokes, state effect lands,
     observable signal emits. Proves the wiring contract is real.
- When an AC changes a control loop, scheduler, retry path, recurring task,
  config reload path, repo-level truth surface, prompt/queue/health projection,
  or strategy/security brain behavior, it must also include a third level:
  3. **Whole-loop proof**: one command or scenario proving the new behavior
     survives the real loop, not just the isolated helper. This can be a
     full-crate suite, a finish-gate run, or a self-hosting/dogfood scenario,
     depending on scope.
- One test per critical behavior (happy path, error case, edge case)
- 3-8 tests per AC

**Pass/fail**: Concrete scenarios with exact inputs and expected outputs.
- Use numbers, not words: `input="name: foo" → Profile { name: "foo" }` not
  "should parse correctly"
- Include at least: one happy path, one edge case, one error case
- 4-10 scenarios per AC

**Blocking note**: Why this AC exists and what depends on it.
- Name which design constraint or dependency motivated this AC
- Explain the design rationale (the "why" that code comments would carry)
- Reference other ACs or components that depend on this one
- 2-5 sentences

**Rollback condition**: What would invalidate this AC after implementation.
- State the failure condition, not the success condition
- Multiple conditions separated by commas or "or"

### 1.9 Decision Log

Record decisions made during spec writing with rationale and date.

```
## Decision Log

- 2026-03-12: Single profile.yaml instead of per-component configs — one file
  is simpler to validate and keeps the repo profile self-contained.
```

### 1.10 Milestone Verification Table

Every spec ends with a milestone verification table:

```
## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Parse minimal project.yaml with name + engine | Core parsing | PL-01 |
| 2 | Missing required field → clear error | Validation | PL-03 |
| 3 | Full project.yaml round-trip: parse → use in supervisor | End-to-end | PL-01, PL-02 |
```

Rules:
- One row per AC minimum, plus at least one end-to-end scenario
- Scenarios use the same concrete value style as pass/fail
- The end-to-end row should span multiple ACs
- At least one row must prove the capability is consumed, not merely written.
  "report persisted to file" is insufficient if no runtime, prompt, queue,
  operator, or downstream system reads it.
- For refactor, migration, or reduction specs, at least one row must prove
  parity or preserved behavior, not just successful new structure.

### 1.11 Anti-patterns

Do NOT:
- Write a spec that cannot answer "what bigger system capability exists if all
  ACs land?"
- Slice a spec by local implementation detail when the real change is an
  end-to-end loop or product behavior
- Write "brain" or "review" specs with vibes-only categories instead of
  enumerating concrete sections, registries, diagrams, and emission rules
- Use external repos as the main proof mechanism when a small local fixture
  corpus could prove the contract more cheaply
- Write specs that create a new truth surface, registry, or report without
  naming the first consumer
- Write BDD-style Given/When/Then (use concrete pass/fail instead)
- Omit the How field (agents need the algorithm, not just the outcome)
- Omit the Whole-system effect field (agents need the bridge from local change
  to platform outcome)
- Use vague pass/fail: "should work correctly" → give exact behavior
- Mix implementation and design rationale in How (rationale → Blocking note)
- Reference test files instead of test commands
- Write ACs that require reading code to verify — pass/fail must be observable
- Skip the Purpose or Current State sections (agents need context)
- Omit the Wiring contract for runtime mechanics (produces unconnected islands)
- Encode incidental scheduler timing or exact last-completed task identity as a
  product contract unless the order itself is user-visible and intentional
- Change a control loop and rely only on isolated exact tests; whole-loop proof
  is required for loop semantics
- Hand-write large structured report fixtures repeatedly when a canonical
  fixture builder could keep schema changes from breaking dozens of tests
- Introduce a new helper, service, or shared module without naming at least two
  concrete callsites or duplications it replaces
- Delete or slim an operator surface without an explicit parity check against
  CLI, state files, or another surviving consumer
- Write a reduction or migration spec that says "simplify" or "clean up"
  without naming the preserved contracts and stop conditions

---

## Part 2: Implementation Plan Entry Format

Plan entries live in `ralph/IMPLEMENT.md`. Each entry is a compressed AC —
enough for a build agent to know what to build, how to verify it, and how it
wires in. The spec is the detailed reference.

### 2.1 Plan Header

```
# Implementation Plan

Status: Active
Date: {YYYY-MM-DD}

## Spec Index

| Spec File | AC Prefix | Count |
|-----------|-----------|-------|
| 031226-profile-loader.md | PL-01..02 | 2 |
| 031226-engine-adapters.md | EA-01..04 | 4 |
```

Rules:
- One row per spec file
- Prefix shows the AC range
- Update count when specs change

### 2.2 Section Headers

Group entries by stage or component:

```
## Stage 1: Foundation
Source spec: specs/031226-profile-loader.md
```

### 2.3 Entry Format

Each entry has exactly 6 fields:

```
- [ ] **{PREFIX}-{NN}** — {Title}
  - Where: `{file paths}`
  - Tests: `{primary cargo test command}`
  - Blocking: {One sentence: why this must exist}
  - Verify: {Compressed pass/fail — key behaviors, semicolons between assertions}
  - Integration: `Trigger=...; Callsite=...; State=...; Persistence=...; Signal=...`
  - Rollback: {Compressed rollback condition}
```

**Checkbox**: `- [ ]` pending, `- [x]` complete, `- [!]` blocked.

**Title**: Same as the spec AC title. Matches exactly.

**Where**: File paths in backticks. `(new)` for new files.

**Tests**: The PRIMARY test command. The spec has the full list; the plan entry
has the entry point.

**Blocking**: One sentence. Compressed version of the spec's blocking note.

**Verify**: Key behaviors joined by semicolons. NOT exhaustive — the spec has
full scenarios. Quick-reference for the build agent.

**Integration**: One-line wiring contract. Must include: `Trigger`, `Callsite`,
`State`, `Persistence`, and `Signal`. Use `N/A` only for non-runtime tasks with
a reason.

**Rollback**: Compressed rollback condition from the spec.

#### Example:

```
- [ ] **PL-01** — Parse project.yaml into Profile struct
  - Where: `src/profile.rs (new)`
  - Tests: `cargo test -p malinka profile::tests::parse_minimal_project`
  - Blocking: Foundation for all dispatch — nothing works without a profile
  - Verify: Parses name, spec_dir, plan_file, engine config, proof_commands; errors on missing required fields
  - Integration: `Trigger=supervisor startup; Callsite=supervisor::init(); State=Profile struct populated; Persistence=read-only from project.yaml; Signal=profile::tests::parse_minimal_project passes`
  - Rollback: Missing field accepted silently or wrong default applied
```

### 2.4 Completed Entry Additions

When completed, the entry gains wiring verification, discovery, and evidence
below the rollback line:

```
- [x] **PL-01** — Parse project.yaml into Profile struct
  - Where: `src/profile.rs`
  - Tests: `cargo test -p malinka profile::tests::parse_minimal_project`
  - Blocking: Foundation for all dispatch — nothing works without a profile
  - Verify: Parses name, spec_dir, plan_file, engine config, proof_commands; errors on missing required fields
  - Integration: `Trigger=supervisor startup; Callsite=supervisor::init(); State=Profile struct populated; Persistence=read-only; Signal=test passes`
  - Rollback: Missing field accepted silently or wrong default applied
  - Trigger: {How mechanic is invoked}
  - Callsite: {Exact file/function that invokes execution}
  - State effect: {Deterministic state delta produced}
  - Persistence effect: {File update, struct mutation, or "read-only"}
  - Observable signal: {Specific test assertion, log line, or CLI output}
  - Discovery: {Unexpected findings during implementation}
  - Evidence: `profile::tests::parse_minimal_project ... ok`
```

**CRITICAL RULE:** For each `[x]` item, the five wiring fields (`Trigger`,
`Callsite`, `State effect`, `Persistence effect`, `Observable signal`) are
**mandatory**. If any are missing, empty, or unverified, the AC is incomplete
and MUST remain `[ ]`.

These fields are written BY the build agent, not by the spec author.

### 2.5 Anti-patterns

Do NOT:
- Put full spec content in plan entries (plan entries are compressed)
- Put plan entries in the spec (they go in `ralph/IMPLEMENT.md`)
- Omit the Tests field (build agents use it as backpressure)
- Use `Integration: N/A` for runtime mechanics
- Use vague Verify: "works correctly" → give the key assertion
- Write multi-paragraph Blocking notes (one sentence)
- Skip the Rollback field (it's the reopen trigger)
- Create plan entries without a corresponding spec AC

---

## Part 3: Spec-to-Plan Workflow

When creating a new spec and adding it to the implementation plan:

1. Write the spec in `specs/{date}-{name}.md` with all sections (1.2-1.10)
1a. Write `## Whole-System Goal`, `## Why This Spec Exists As One Unit`,
    `## What Already Exists`, and `## Architecture / Runtime Contract` before
    drafting ACs; add `## Operational Controls` at the same time when required
2. Add a row to the Spec Index table in `ralph/IMPLEMENT.md`
3. Add plan entries to the appropriate stage section
4. Populate each plan entry `Integration` field from the AC wiring contract
5. Update the plan header date

Before finalizing the spec, run this holism check:
- Can you explain the new system capability in one paragraph without naming
  files?
- Does every AC have a non-trivial Whole-system effect?
- Is there at least one end-to-end scenario in Milestone Verification that
  sounds like the actual product goal, not just a local helper test?
- Can you name the first consumer of every new emitted surface, report,
  registry, or projection?
- If this spec lands and no follow-on code changes, what concrete operator,
  runtime, or user-visible behavior improves immediately?
- If all ACs land, would a skeptical reviewer say "yes, that capability now
  exists"? If not, the spec is too narrow or wrongly cut.
- If the spec is a refactor, migration, or reduction, can you point to the
  exact contracts that are preserved and the gate that proves parity?

### Cross-references Between Specs

When one spec extends another spec's AC:
- Add a note ABOVE the extended AC's fields in the target spec
- Reference the extending AC by full ID: "PL-02 (specs/031226-profile-loader.md)"
- Update the target spec's date

Example:
```
**Note**: EA-01 (specs/031226-engine-adapters.md) extends this AC with engine
dispatch. Profile must expose adapter kind and model for engine selection.

### AC-PL-02: Engine configuration deserialization
...
```

---

## Part 4: Loop Contracts

The loop system consumes specs and plan entries as input.

### 4.1 Build Loop

| Loop | Plan file | Prompt file |
|------|-----------|-------------|
| `loopcodex.sh` | `ralph/IMPLEMENT.md` | `loop/PROMPT_build.md` |

Each iteration:

1. Extract next unchecked `- [ ]` task block from IMPLEMENT.md.
2. Substitute task block into PROMPT_build.md.
3. Run agent with the compiled prompt.
4. Run proof commands (tests, clippy, build gates).
5. Mark complete or blocked based on proof results.
6. Commit and push if changes exist.
7. Loop restarts with fresh context.

Build loops may be scoped to specific plan headings with
`--target "<heading substring>"`. Repeating `--target` unions multiple heading
filters.

Invariants:

- One task per iteration.
- The plan file is shared state between iterations.
- Each iteration starts from deterministic file state (prompt + specs + plan).
- Fail-closed: no structured output = incomplete turn.
- The RESULT/BLOCKED structured output line is the authoritative turn outcome.

### 4.2 Backpressure

Every loop output faces validation gates. **Scoped execution is mandatory** —
always scope to the affected crate/module: `cargo test -p malinka <test_filter>`.

| Gate | What it catches |
|------|-----------------|
| Tests | Logic errors, regressions |
| Clippy | Style violations, dead code, type drift |
| Build | Compilation, linking |

Rules:
- Every task must have `Required tests` with concrete, scoped commands.
- Tests are part of implementation scope, not optional.
- On failure, fix and retry within the same iteration. Never commit failing code.
- If a task omits required tests, reject it and proceed to next.
- If a runtime task has missing/empty wiring fields, reject completion.

---

## Part 5: Process Contract

Parts 1-4 define how to write specs and plan entries. Part 5 defines when they
are required, what governs their lifecycle, and when work is complete.

### 5.1 When Specs and Plans Are Required

Use a spec + plan when work is expected to be more than a small local edit:

- Multi-file feature work
- Architectural refactors
- Migrations or behavior-sensitive rewrites
- New subsystem introduction
- Production/readiness workflow changes

For trivial edits (single-file fixes, typo corrections, config changes), a full
spec is optional. If in doubt, write the spec — the cost of an unnecessary spec
is low; the cost of an unspecified multi-file change is high.

### 5.2 Non-Negotiable Rules

Every spec and plan entry MUST satisfy all six:

1. **Self-contained.** Define all domain terms and assumptions inline. A build
   agent or new engineer must be able to complete the work end-to-end without
   prior context or chat history.

2. **Living document.** Update progress, discoveries, and decisions as work
   evolves. A stale spec is worse than no spec — it actively misleads.

3. **Executable.** Include exact commands, file paths, and pass/fail
   verification. If a build agent cannot run the verification, the spec is
   incomplete.

4. **Outcome-focused.** Describe expected user-visible behavior, not only code
   edits. The spec answers "what does the user see?" before "what does the code
   do?"

5. **Evidence-backed completion.** No task is complete without recorded proof.
   Checked checkbox without evidence = false progress.

6. **Whole-system coherence.** Every spec must describe the larger loop or
   platform behavior it advances. If the ACs are locally precise but still do
   not add up to a meaningful capability, the spec is incomplete.

### 5.3 Spec Lifecycle

```
Draft ──▶ Active ──▶ Complete ──▶ Archived
  │                      │
  └──── Superseded ◀─────┘ (if replaced by newer spec)
```

| State | Meaning | Location |
|-------|---------|----------|
| Draft | Written, not yet being built | `specs/*.md`, Status: Draft |
| Active | Implementation in progress | `specs/*.md`, Status: Active |
| Complete | All ACs implemented and verified | `specs/*.md`, Status: Complete |
| Archived | Superseded or no longer relevant | `specs/archive/*.md` |
| Superseded | Replaced by a newer spec | Note in header, moved to archive |

Transitions:
- Draft → Active: first plan entry moves to `[x]` or implementation begins.
- Active → Complete: all plan entries `[x]`, milestone verification passes.
- Complete → Archived: spec is no longer the active reference for any work.
- Any → Superseded: a new spec explicitly replaces this one.

### 5.4 Required Sections

Every spec MUST contain all core sections. Conditional sections become
mandatory when their trigger condition applies. A build agent encountering a
spec with missing required sections MUST reject it.

Core spec sections:

| # | Section | Location | Reference |
|---|---------|----------|-----------|
| 1 | Purpose | Spec body | §1.3 |
| 2 | Whole-System Goal | Spec body | §1.3.1 |
| 3 | Why This Spec Exists As One Unit | Spec body | §1.3.2 |
| 4 | Scope | Spec body | §1.4 |
| 5 | Current State | Spec body | §1.5 |
| 6 | What Already Exists | Spec body | §1.5.1 |
| 7 | Non-goals | Spec body | §1.6 |
| 8 | Ownership Map | Spec body | §1.7 |
| 9 | Decision Log | Spec body | §1.9 |
| 10 | Milestone Verification | Spec body | §1.10 |

Conditional spec sections:

| Trigger | Required section | Reference |
|---------|------------------|-----------|
| Runtime / orchestration / brain / control-plane / operator-surface / repo-truth spec | Architecture / Runtime Contract | §1.7.1 |
| New writer / report / registry / truth surface / consumer-facing behavior | Adoption / Consumption Path | §1.7.2 |
| Runtime / control-loop / migration / reduction / multi-phase refactor spec | Operational Controls | §1.7.3 |
| Completed spec | Outcomes & Retrospective | §5.11 |

Plan-side required sections:

| Section | Location | Reference |
|---------|----------|-----------|
| Implementation Plan / Spec Index | `ralph/IMPLEMENT.md` | Part 2 |
| Progress | Checkboxes `[ ]/[x]/[!]` | §2.3, §5.5 |
| Surprises & Discoveries | Discovery field | §2.4, §5.10 |
| Validation evidence | Evidence field | §2.4, §5.7 |

### 5.5 Progress Tracking

Progress is tracked via plan entry checkboxes (`[ ]`, `[x]`, `[!]`).

Rules:
- Update progress **continuously**, not in batches. Mark each task `[x]`
  immediately upon completion, before starting the next task.
- Never batch-complete multiple tasks in one pass. Each task gets its own
  iteration with its own evidence.
- A `[x]` without the 5 wiring fields + Evidence is false progress. The
  adjudicator rejects it.
- A `[!]` must include a blocker tag explaining what is blocked and why.

### 5.6 Plan Lifecycle

```
Created ──▶ In Progress ──▶ Complete ──▶ Archived
```

A plan (`ralph/IMPLEMENT.md`) is complete only when ALL of:

1. All scoped tasks are checked complete (`[x]`).
2. Required validation commands are run and results recorded (Evidence field).
3. Downstream docs are updated (specs, README, runbooks as applicable).
4. Spec disposition is explicit — `specs/` if canonical, `specs/archive/` if
   superseded.
5. The plan is archived if no longer active.
6. Outcomes & Retrospective section is written in the spec (§5.11).

### 5.7 Validation Protocol

Validation proves the work is correct. Two levels:

**Per-task validation** (during implementation):
- Build agent runs the task's proof commands (from Tests field).
- Adjudicator independently re-runs proof commands to verify.
- Results are recorded in the Evidence field of the completed entry.

**End-of-plan validation** (after all tasks `[x]`):
- Run the Milestone Verification table scenarios (§1.10) end-to-end.
- Record pass/fail for each scenario in the spec.
- Any failure reopens the relevant task — mark it `[ ]` with a note.

### 5.7.1 Control-Loop Change Protocol

This protocol applies when work touches:
- `supervisor`, `recurring`, `queue`, `prompt`, `workflow`, `health`,
  `runtime_state`, `session`, doctrine control, operator surfaces, or
  strategy/security report contracts
- retry, cadence, prioritization, config reload, arbitration, emitted work, or
  repo-level latest-truth behavior

Required evidence for these changes:
1. Contract proof
   - The narrow helper or validator behaves correctly in isolation.
2. Lifecycle proof
   - The real trigger/callsite/persistence/observable chain works.
3. Whole-loop proof
   - One of:
   - targeted full-crate suite if scheduler semantics changed
   - `loopfinish.sh` if finish/quality-gate behavior changed
   - self-hosting or repo-dogfood pass if strategy/security/control-plane
     behavior changed
4. Invariant statement
   - The spec or AC must say which assertions are true invariants and which
     orderings are incidental.
5. Operational controls
   - The spec must include `## Operational Controls` with phase order, gate
     rules, failure modes, and diagram maintenance targets.

Rules:
- Prefer eventual invariants over exact order. Assert that the right tasks
  launch, persist, or complete; do not overfit to which task finished last
  unless that order is the actual product requirement.
- If a test must assert exact ordering, the spec must explain why the ordering
  is user-visible, operator-visible, or otherwise contractual.
- If a change introduces a richer structured report contract, prefer shared
  fixture builders over repeated hand-authored JSON snippets.
- If the full-suite or whole-loop proof disagrees with isolated exact tests,
  treat the loop-level signal as authoritative until the mismatch is explained.

Validation results are durable. They live in the plan (Evidence field) and spec
(Milestone Verification table). "I ran the tests" without output is not
evidence.

### 5.8 Directory Topology

```
repo/
├── project.yaml              # Repo profile (engine, proof gates, workspace)
├── specs/                    # Active specs
│   ├── MMDDYY-name.md       # One spec per component
│   └── archive/              # Superseded/completed specs
├── ralph/
│   ├── SPEC.md               # This document (format + process reference)
│   └── IMPLEMENT.md          # Active implementation plan
├── loop/
│   ├── PROMPT_build.md       # Build prompt template
│   ├── PROMPT_plan.md        # Planning prompt template
│   ├── loopcodex.sh          # Loop runner
│   └── _lib.sh               # Shared loop library
└── src/                      # Implementation
```

Rules:
- Active specs live in `specs/`. Completed or superseded specs move to
  `specs/archive/`.
- One plan file at a time (`ralph/IMPLEMENT.md`). Multiple concurrent plans
  create coordination problems.
- Prompt templates live in `loop/`. They consume specs and plan entries as
  input.

### 5.9 Writing Guidance

These rules govern spec and plan prose quality:

- Prefer concrete prose over terse bullet fragments when behavior is subtle.
  A build agent cannot infer nuance from abbreviations.
- Include exact commands and expected outputs for key validations. "Run the
  tests" is not actionable; `cargo test -p malinka plan::tests::parse_plan`
  is.
- Prefer synthetic local fixtures over large external-repo validation whenever
  the goal is to prove a contract rather than a downstream product.
- For control-loop changes, write tests around durable invariants first:
  emitted files, persistent state, visible prompts, queue selections,
  adjudication classes, health surfaces. Only assert exact order when order is
  the thing the product promises.
- Keep text forward-compatible: avoid references that require chat context,
  session state, or "as discussed." If context matters, inline it.
- If the approach changes mid-implementation, revise the spec and plan
  immediately before continuing. Never proceed with a plan that contradicts
  the code.
- Use the Decision Log (§1.9) to record why approaches changed, not just what
  changed.
- If a spec begins to read like a pile of ACs rather than a coherent product or
  platform bet, stop and rewrite the big-picture sections before adding more
  entries.

### 5.10 Surprises and Discovery Protocol

During implementation, build agents encounter unexpected findings: missing
dependencies, undocumented API behavior, performance cliffs, design
assumptions that don't hold.

Rules:
- Record surprises in the completed plan entry's `Discovery` field (§2.4).
- If a discovery invalidates an AC's assumptions, check the Rollback condition
  — if triggered, mark the task `[!]` and update the spec.
- If a discovery affects other ACs, add a cross-reference note (§3 workflow).
- Never silently work around a surprise. The Discovery field exists so future
  agents don't hit the same wall.

### 5.11 Outcomes & Retrospective

When all plan tasks are `[x]` and milestone verification passes, add an
`## Outcomes & Retrospective` section to the spec before archiving. This
section captures what was learned — not what was built (that's in the ACs).

Required content:

```
## Outcomes & Retrospective

### What shipped
- {User-visible outcomes, not implementation details}

### What surprised us
- {Consolidated from Discovery fields across plan entries}

### What we'd do differently
- {Process improvements, spec gaps, tooling friction}

### Unresolved debt
- {Technical debt introduced intentionally, with rationale}
- {Known limitations deferred to future specs}
```

Rules:
- Write this AFTER completion, not during. In-progress retrospectives are
  speculation.
- Reference specific Discovery fields and Decision Log entries rather than
  vague observations.
- The "What we'd do differently" section feeds future spec authoring. If a
  process rule from §5.2-5.10 was insufficient, note it here so the rule can
  be improved.
- Unresolved debt items should reference a future spec or plan entry where
  the debt will be addressed.
