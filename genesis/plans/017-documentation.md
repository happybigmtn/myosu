# Documentation Sprint

**Plan ID:** 017
**Status:** New
**Priority:** HIGH — reduces onboarding friction

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Myosu will have a complete developer documentation set: a README that accurately describes the current state (not the aspirational state), API docs for `myosu-games` and `myosu-sdk`, an architecture overview, and a developer onboarding guide.

---

## Progress

- [ ] Audit current README for accuracy — update to reflect current state
- [ ] Write architecture overview with ASCII diagrams
- [ ] Write developer onboarding guide
- [ ] Write API docs for `myosu-games`
- [ ] Write developer guide for SDK ("30-Minute Game")
- [ ] Verify docs build with `cargo doc`

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: README describes current state, not aspirational state.
  Rationale: A README that says "run the full devnet" but the full devnet doesn't build is worse than no README.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Audit and update root README
Read the current README. Verify every claim about what works. Update or remove anything that doesn't match reality.

Proof: `rg 'cargo run|cargo test' README.md` references only commands that actually work.

### M2: Write architecture overview
ASCII diagram of the full stack + description of each layer.

Proof: `test -f docs/architecture.md` with ASCII diagram showing all components.

### M3: Write developer onboarding guide
Step-by-step: clone → build → test → run → contribute.

Proof: `test -f docs/onboarding.md`; following it produces a working local environment.

### M4: Write API docs for `myosu-games`
`cargo doc -p myosu-games` produces complete API documentation.

Proof: `cargo doc -p myosu-games --no-deps && test -f target/doc/myosu_games/index.html`.

### M5: Verify `cargo doc` passes
All public items have doc comments.

Proof: `cargo doc --all-features --no-deps` produces no warnings.

---

## Validation

- `cargo doc -p myosu-games --no-deps` produces docs with no warnings
- `cargo doc -p myosu-tui --no-deps` produces docs with no warnings
- `test -f docs/architecture.md` with ASCII diagram
- `test -f docs/onboarding.md` with step-by-step guide
- `test -f docs/developer-guide.md` with "30-Minute Game" tutorial
