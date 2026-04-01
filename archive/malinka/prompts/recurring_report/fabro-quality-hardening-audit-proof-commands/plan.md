# Audit all Fabro proof commands for false-green patterns Lane — Plan

Lane: `fabro-quality-hardening-audit-proof-commands`

Goal:
- Audit all Fabro proof commands for false-green patterns

Child work item of plan: Fabro Quality Hardening

Objective:
Classify every proof command as honest (tests real behavior) or false-green (tests only compilation)

Owned surfaces:
- `fabro/run-configs/`
- `fabro/checks/`

Proof commands:
- `rg -l 'cargo (build|check)' fabro/run-configs/ fabro/checks/`

Required durable artifacts:
- `spec.md`
- `review.md`

Context:
- Plan file:
- `genesis/plans/010-fabro-quality-hardening.md`

Child work item: `fabro-quality-hardening-audit-proof-commands`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Fabro Quality Hardening

**Plan ID:** 010
**Status:** New
**Priority:** CRITICAL — autodev currently produces fake completions

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the Fabro autodev loop will no longer produce fake completions. Every lane that reports "complete" will have actually passed deterministic quality gates — not just `cargo build`. The proof commands in every run config will assert real behavior, not just compilation success.

---

## Progress

- [ ] Audit all Fabro proof commands for false-green patterns
- [ ] Replace `cargo test -p myosu-games` with explicit test names
- [ ] Add quality gate to all implementation-family programs
- [ ] Add smoke test to all generated programs
- [ ] Verify autodev no longer produces compile-only completions

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: A "complete" Fabro lane must satisfy: compilation + tests + smoke behavior.
  Rationale: Currently, `cargo build` passing is treated as "the code works." This is insufficient. We need `cargo build` + `cargo test` + a smoke invocation that produces expected output.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: The quality gate must be deterministic and automated.
  Rationale: Manual review is not scalable across all the generated implementation programs. The gate must run automatically and produce a boolean result.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Audit all Fabro proof commands
Find every proof command in `fabro/run-configs/` and `fabro/checks/`. Classify each as: honest (tests real behavior) or false-green (tests only compilation).

Proof: `rg -l 'cargo (build|check)' fabro/run-configs/ fabro/checks/` returns all files; each is annotated as honest/false-green.

### M2: Replace `cargo test -p myosu-games` with explicit test names
The current proof command `cargo test -p myosu-games` runs only doc tests (9 tests). Replace with `cargo test -p myosu-games -- serialization_roundtrip` which runs 100+ property tests.

Proof: `cargo test -p myosu-games -- serialization_roundtrip | rg 'test result'` shows ≥ 100 tests passing.

### M3: Add smoke test to all generated implementation programs
For each generated implementation program (play-tui, poker-engine, multi-game, sdk-core), add a smoke test that verifies the binary does more than just exit(0).

Proof: `cargo run -p myosu-play -- train --smoke-test` exits with code 1 if the binary just exits, not 0.

### M4: Verify autodev no longer produces fake completions
Run `raspberry autodev` on a fresh set of implementation programs. Verify that no program reports "complete" without passing all quality gates.

Proof: After autodev, every program marked "complete" in Raspberry has `cargo test` passing and smoke test passing, not just `cargo build`.

---

## Context and Orientation

Known false-green proof commands:

| Program | Current Proof | Problem |
|---------|--------------|---------|
| `play-tui-implementation` | `cargo build -p myosu-play` | Binary exits without loop |
| `poker-engine-implementation` | `cargo test -p myosu-games` | Only runs 9 doc tests |
| `multi-game-implementation` | `cargo test -p myosu-games` | Same as above |
| `sdk-core-implementation` | `cargo check -p myosu-sdk` | Only checks types |

The quality gate hierarchy:
```
Level 1: cargo check     → code parses
Level 2: cargo build     → code compiles
Level 3: cargo test      → tests pass (MUST specify test name, not all)
Level 4: smoke test      → binary produces expected behavior
Level 5: integration test → full pipeline works
```

Every Fabro lane should reach at least Level 3 with explicit test names.

---

## Plan of Work

1. Audit all Fabro proof commands
2. Categorize each as honest/false-green
3. Replace false-green with honest proof commands
4. Add smoke tests to binaries
5. Verify autodev behavior

---

## Concrete Steps

```bash
# Audit all proof commands
for f in $(find fabro/run-configs fabro/checks -type f); do
  echo "=== $f ==="
  rg 'cargo (test|build|check)' "$f"
done

# Check current myosu-games test count
cargo test -p myosu-games -- --list 2>&1 | rg 'test' | wc -l
# Expected: 9 (doc tests only — insufficient)

# After implementing property tests (Plan 005):
cargo test -p myosu-games -- serialization_roundtrip -- --list 2>&1 | rg 'test' | wc -l
# Expected: ≥ 100

# Smoke test pattern
cargo run -p myosu-play -- --smoke-test 2>&1
# Expected: exits with 0 if binary works; exits with 1 if binary just returns

# Verify autodev quality
# Run autodev on a fresh implementation program
raspberry execute --manifest fabro/programs/myysu-play-tui-implementation.yaml
# Check that completion requires: cargo test -p myosu-tui PASSING
# NOT just: cargo build -p myosu-play PASSING
```

---

## Validation

- Every `fabro/run-configs/**/*.toml` has proof commands at Level 3 or higher
- Every generated implementation program has a smoke test
- `cargo test -p myosu-games -- serialization_roundtrip` shows ≥ 100 tests
- `cargo test -p myosu-tui` shows ≥ 20 tests
- After autodev run, no program reports "complete" without passing its full proof chain


Workflow archetype: report

Review profile: hardened

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: All Fabro run-config TOML files and check scripts that contain proof commands
- How: Classify every proof command as honest (tests real behavior) or false-green (tests only compilation)
- Required tests: rg -c 'cargo (build|check)' fabro/run-configs/ fabro/checks/
- Verification plan: Every proof command in run-configs and checks is annotated as honest or false-green; no unclassified commands remain
- Rollback condition: New run-configs or checks are added with unclassified proof commands

Proof commands:
- `rg -l 'cargo (build|check)' fabro/run-configs/ fabro/checks/`

Artifacts to write:
- `spec.md`
- `review.md`
