# ADR 010: INV-004 Enforcement Stays in CI for Stage-0

- Status: Accepted
- Date: 2026-04-05
- Deciders: Myosu maintainers
- Consulted: `INVARIANTS.md`, `WORKLIST.md`, `nemesis/IMPLEMENTATION_PLAN.md`, `.github/workflows/ci.yml`
- Informed: miner, gameplay, CI, and release-gate contributors
- Related: `crates/myosu-play/tests/invariants.rs`, `.github/workflows/ci.yml`, `docs/operator-guide/architecture.md`

## Context

INV-004 says a gameplay bug must not corrupt training data and that there must
be no dependency path between `myosu-play` and `myosu-miner`.

The live repo already enforces that boundary in two places:

- CI runs a `cargo tree` guard in `.github/workflows/ci.yml`
- `crates/myosu-play/tests/invariants.rs` asserts that neither package depends
  on the other

Nemesis revisited whether stage-0 also needs runtime enforcement. The question
sounds reasonable at first, but the boundary in question is a workspace/package
graph property, not a dynamic runtime permission check. If `myosu-play` starts
depending on `myosu-miner`, the violation already exists at compile time. A
runtime log or assertion would only observe that mistake after the build graph
had already been allowed to drift.

## Decision

For stage-0, Myosu keeps INV-004 enforcement in CI and invariant tests. It does
not add a separate runtime assertion or runtime "boundary crossing" detector.

The authoritative enforcement surfaces are:

- the CI `cargo tree` check
- the invariant test committed in the workspace

That is the active repo position until Myosu introduces a real plugin boundary,
dynamic module loading, or another runtime composition model where package
separation is no longer enough.

## Alternatives Considered

### Option A: Keep CI and invariant-test enforcement only

This wins because it matches the actual shape of the boundary. INV-004 is about
crate graph separation, and the workspace already exposes truthful compile-time
proofs for that property.

### Option B: Add a runtime assertion inside `myosu-play` or `myosu-miner`

This is rejected because the violation would already have happened by the time
the binary starts. Runtime checks would add ceremony without creating a stronger
guarantee than the existing compile-time proof.

### Option C: Add a bespoke build script or feature-flag trap

This is rejected for stage-0 because it would duplicate CI coverage without
changing the trusted proof surface materially.

## Consequences

### Positive

- The repo keeps one clear proof story for INV-004 instead of splitting the
  guarantee across CI, runtime, and documentation.
- Contributors can validate the boundary locally with the same command CI uses
  conceptually.
- The decision matches the actual risk: dependency drift is introduced at build
  time, not through a runtime control path.

### Negative

- Local ad hoc builds that skip invariant tests can still compile unrelated
  work until CI or the invariant test is run.
- This ADR does not help if the project later adopts a runtime composition model
  where package boundaries no longer capture the real safety boundary.

### Follow-up

- Keep the CI `cargo tree` guard and the invariant test in sync if package names
  or crate layout change.
- Reopen this ADR only if stage-1 or later introduces dynamic linking,
  plugins, agent tool injection, or another runtime mechanism that weakens the
  compile-time boundary.

## Reversibility

Easy today, conditional later.

If Myosu adds runtime composition or cross-process plugin loading, this decision
can be superseded with a new ADR. Until then, the current compile-time boundary
is the truthful enforcement point.

## Validation / Evidence

- `cargo test -p myosu-play --quiet inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other`
- `.github/workflows/ci.yml` `INV-004 solver-gameplay dependency boundary`
- `crates/myosu-play/tests/invariants.rs`
