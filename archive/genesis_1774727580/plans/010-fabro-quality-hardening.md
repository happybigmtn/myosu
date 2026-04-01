# Quality Gate Hardening

**Plan ID:** 010
**Status:** In Progress
**Priority:** CRITICAL — the repo still has false-green completion paths

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the repo will no longer allow compile-only progress to
masquerade as completion. Every important execution path will have deterministic
quality gates covering compilation, targeted tests, and smoke behavior.

---

## Progress

- [x] (2026-03-28 17:09Z) Audit completion and verification commands for false-green patterns
- [x] (2026-03-28 17:09Z) Replace weak `myosu-games` verification references in living docs where needed
- [x] (2026-03-28 17:09Z) Add quality gates to direct execution surfaces
- [x] (2026-03-28 17:09Z) Add smoke tests to runnable binaries
- [x] (2026-03-28 17:09Z) Verify the active Genesis plans now point at behavioral proofs for `myosu-play`
- [x] (2026-03-28 17:09Z) Clean the highest-signal historical Genesis and `outputs/` artifacts that still pointed at stale proof commands
- [x] (2026-03-28 17:09Z) Clean the remaining active Genesis child-plan proof references that still used broad or stale commands
- [x] (2026-03-28 17:09Z) Add historical notes and current CLI examples to the highest-signal active specs that still used old `myosu-play` flag forms
- [x] (2026-03-28 18:02Z) Confirm that the remaining drift is doctrine-level Malinka/autodev coupling and spin out a dedicated cutover plan

---

## Surprises & Discoveries

- Discovery: The current GitHub Actions workflow was still only covering
  `myosu-games` and `myosu-tui`, even after `myosu-games-poker` and
  `myosu-play` became active execution surfaces. Evidence:
  `.github/workflows/ci.yml` only ran `cargo check -p myosu-games -p
  myosu-tui` before this change.
  Date/Author: 2026-03-28 / Codex

- Discovery: A runnable smoke gate for `myosu-play` cannot assert one fixed
  advice source anymore. On this machine the binary auto-loads the local
  codexpoker export and reports `advice_source=artifact`, while CI may still
  run the generated fallback path. The honest smoke proof is behavior:
  preflop state, `ACTION call`, then flop state.
  Evidence: `cargo run -p myosu-play --quiet -- --smoke-test` now prints
  `SMOKE myosu-play ok`, `initial_street=PREFLOP`, and `next_street=FLOP`.
  Date/Author: 2026-03-28 / Codex

- Discovery: The remaining false-green references were mostly in living docs,
  not code. `README.md`, Plan 006, and Plan 009 still surfaced weaker proof
  commands than the repo now supports, so the next hardening step was textual
  control-plane cleanup rather than more binary work.
  Evidence: `rg -n "cargo test -p myosu-games$|cargo build -p myosu-play$" README.md genesis/plans`
  found stale references before the update.
  Date/Author: 2026-03-28 / Codex

- Discovery: The most dangerous stale proof language was in "authoritative
  looking" historical artifacts, not just live plans. `GENESIS-REPORT.md`,
  `ASSESSMENT.md`, and the `outputs/play/tui` and `outputs/games/*` lane docs
  were still easy to misread as current proof doctrine until they gained
  historical notes and updated command examples.
  Evidence: a focused grep over those files for `cargo build -p myosu-play`,
  `cargo test -p myosu-games`, and `myosu-play --train` returned no matches
  after the cleanup.
  Date/Author: 2026-03-28 / Codex

- Discovery: The last stale references inside `genesis/plans/` were narrow and
  mostly operational. Plan 005 and Plan 002 still used broad `cargo test -p
  myosu-games` commands in their concrete steps, while the rest of the live
  plan set was already aligned or intentionally describing the widened CI
  command.
  Evidence: a grep over `genesis/plans/` only reported the intentional Plan 010
  note and the valid full active-crate CI command after the cleanup.
  Date/Author: 2026-03-28 / Codex

- Discovery: Active doctrine can drift independently from the plan set. The
  highest-signal remaining CLI mismatches were in the spec layer and
  agent-experience artifacts, where old `--pipe` and `--train` forms still
  appeared even after the live binary had moved to subcommands. Historical
  notes plus normalized examples were enough to make those docs safe again
  without pretending every future flag is already implemented.
  Evidence: a focused grep over the touched spec files only reports the
  intentional historical-note lines after cleanup.
  Date/Author: 2026-03-28 / Codex

- Discovery: The next blocker is no longer weak proof commands but mixed
  control-plane doctrine. Live entrypoints still advertise retired `autodev`
  and Malinka-era surfaces in places like `README.md` and Genesis docs, so the
  next honest step is doctrine cutover rather than more proof-command
  substitutions.
  Evidence: `rg -n "Malinka|malinka|autodev|raspberry autodev" OS.md README.md genesis`
  still returned live-doctrine matches after the proof cleanup.
  Date/Author: 2026-03-28 / Codex

---

## Decision Log

- Decision: A complete execution slice must satisfy compilation + tests + smoke
  behavior.
  Rationale: `cargo build` passing is not evidence that the code works. We need
  `cargo build` + `cargo test` + a smoke invocation that produces expected
  output.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: The quality gate must be deterministic and automated.
  Rationale: Manual review is not enough. The gate must run automatically and
  produce a boolean result.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: `myosu-play` should expose a dedicated `--smoke-test` path instead
  of relying on ad hoc stdin piping in CI.
  Rationale: A first-class smoke command gives the runnable surface a stable
  Level 4 proof and lets CI verify real state progression without depending on
  terminal control or machine-specific advice-source defaults.
  Date/Author: 2026-03-28 / Codex

---

## Outcomes & Retrospective

Plan 010 is now partially landed. The audit confirmed that active crate quality
gates had drifted behind the actual repo surface: CI still exercised only
`myosu-games` and `myosu-tui` even though `myosu-games-poker` and `myosu-play`
are now part of the live wedge. The first hardening step is in place: CI now
checks, tests, lints, and rustfmt-checks all four active crates, and
`myosu-play` now exposes a deterministic `--smoke-test` command that proves the
binary reaches preflop, accepts `call`, and advances to flop.

Plan 010 has reached its natural boundary. What remains is no longer primarily a
quality-gate problem; it is a doctrine-cutover problem. The next slice should
remove live Malinka/autodev coupling, refresh `OS.md` around the real
Fabro/Raspberry operator loop, and only then decide whether a fresh
synth-assisted Genesis planning pass is warranted.

---

## Milestones

### M1: Audit all active verification commands
Find every active completion command in CI, docs, scripts, and current plan
instructions. Classify each as honest or false-green.

Proof: `rg -n 'cargo (build|check|test)' .github genesis README.md crates`
identifies the current verification surfaces, and each is classified.

### M2: Replace weak `myosu-games` verification with explicit targets
If `cargo test -p myosu-games` is too broad or too weak for the real acceptance
goal, replace it with explicit test targets such as
`cargo test -p myosu-games -- serialization_roundtrip`.

Proof: `cargo test -p myosu-games -- serialization_roundtrip | rg 'test result'`
shows the expected property-test run.

### M3: Add smoke tests to runnable binaries and integration surfaces
For each runnable surface, add a smoke test that verifies it does more than just
compile or exit immediately.

Proof: each runnable surface has a documented smoke command and expected output.

### M4: Verify no active plan claims completion from compile-only checks
Review the active plan set and CI so no milestone can be considered complete
without passing its full quality gate.

Proof: CI and local verification both require targeted tests or smoke behavior
for the surfaces they claim to validate.

---

## Context and Orientation

Known false-green patterns:

| Surface | Weak Check | Problem |
|---------|------------|---------|
| `myosu-play` | `cargo build -p myosu-play` | Compile success is not a behavioral proof |
| `myosu-games` | `cargo test -p myosu-games` | May miss the property-based target we actually care about |
| any new binary | `cargo check` only | Type correctness is not runnable behavior |

The quality gate hierarchy:

```text
Level 1: cargo check      -> code parses
Level 2: cargo build      -> code compiles
Level 3: cargo test       -> targeted tests pass
Level 4: smoke test       -> runnable behavior works
Level 5: integration test -> end-to-end path works
```

Every active verification path should reach at least Level 3 with explicit test
names.

---

## Plan of Work

1. Audit all active verification commands
2. Categorize each as honest or false-green
3. Replace false-green checks with honest proof commands
4. Add smoke tests to binaries
5. Verify plan and CI behavior

---

## Concrete Steps

```bash
# Audit verification commands
rg -n 'cargo (test|build|check|clippy|fmt)' .github genesis README.md crates

# Check current myosu-games test count
cargo test -p myosu-games -- --list 2>&1 | rg 'test' | wc -l

# After implementing property tests (Plan 005):
cargo test -p myosu-games -- serialization_roundtrip -- --list 2>&1 | rg 'test' | wc -l

# Smoke test pattern
cargo run -p myosu-play -- --smoke-test 2>&1

# Verify completion quality
# Check that plans and CI require:
# cargo test -p myosu-tui PASSING
# and/or a real smoke test
# not just: cargo build -p myosu-play PASSING
```

---

## Validation

- Every active verification path reaches Level 3 or higher
- Every runnable implementation surface has a smoke test
- `cargo test -p myosu-games -- serialization_roundtrip` shows the property-based target
- `cargo test -p myosu-tui` shows the TUI suite
- No active plan claims completion without its full proof chain
