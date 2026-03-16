# Myosu Invariants

Hard rules for the myosu game-solving subnet chain.

## INV-001: Structured Closure Honesty

- Statement: No dispatched turn may be treated as complete unless it ends in a
  trusted structured `RESULT:` or `BLOCKED:` outcome or fails closed.
- Why: Truthful unattended execution is the platform's core promise.
- Enforcement: adjudicator, supervisor, and the delivery contract in `WORKFLOW.md`.
- Measurement: count turns that mutate plan or land trunk without a trusted
  terminal outcome.
- No-ship rule: any confirmed violation is at least `S1`.
- Fallback mode: mark the turn incomplete, preserve evidence, repair closure.

## INV-002: Proof Honesty

- Statement: Named proof commands must actually execute and must never be used
  as false-green placeholders.
- Why: A completion claim without honest proof destroys trust.
- Enforcement: proof gating in adjudicator and plan/task proof command contracts.
- Measurement: `false_green_proof_count`.
- No-ship rule: any confirmed false-green proof is at least `S1`.
- Fallback mode: reopen the work, fix the proof surface, rerun honestly.

## INV-003: Game Verification Determinism

- Statement: For any given game state and strategy profile, the exploitability
  score computed by any validator must produce the same result within floating
  point tolerance (epsilon < 1e-6).
- Why: The subnet's incentive mechanism depends on objective, reproducible
  quality measurement. Non-deterministic validation breaks Yuma Consensus.
- Enforcement: deterministic PRNG seeding, canonical game state serialization,
  exact-arithmetic exploitability computation where feasible.
- Measurement: max divergence across validator scores for identical inputs.
- No-ship rule: validator disagreement above epsilon on identical inputs is `S0`.
- Fallback mode: freeze solver emissions until determinism is restored.

## INV-004: Solver-Gameplay Separation

- Statement: The solver layer (miners competing on strategy quality) and the
  gameplay layer (humans playing against bots) must share game engine code but
  never share runtime state or trust boundaries.
- Why: A compromised solver must not affect live gameplay fairness. A gameplay
  bug must not corrupt training data.
- Enforcement: separate crates (`myosu-solver`, `myosu-play`), shared
  dependency on game engine crates, no direct imports between solver and play.
- Measurement: `cargo tree` dependency check — no path from play → solver or
  solver → play.
- No-ship rule: direct dependency between solver and play crates is `S1`.
- Fallback mode: revert the offending dependency, refactor shared code into
  the engine crate.

## INV-005: Plan And Land Coherence

- Statement: `IMPLEMENT.md` truth, git land behavior, and task runtime truth
  must not drift apart.
- Why: A trustworthy platform cannot report a task complete when git state,
  plan state, and runtime evidence disagree.
- Enforcement: rollback-on-land-failure behavior, plan mutation rules.
- Measurement: count of land attempts that leave plan/git/runtime in divergent
  states after recovery.
- No-ship rule: unresolved divergence after recovery is at least `S1`.
- Fallback mode: roll back plan status, preserve evidence, treat as incomplete.

## INV-006: Robopoker Fork Coherence

- Statement: The robopoker fork (`happybigmtn/robopoker`) must track v1.0.0 as
  its baseline. Changes must be documented in CHANGELOG.md with rationale.
  Core MCCFR algorithm changes require review.
- Why: We own the fork but the v1.0.0 MCCFR engine is proven. Diverging from
  core algorithm correctness risks solver quality.
- Enforcement: `Cargo.toml` git dependency pinned to fork branch/tag,
  CHANGELOG.md in fork documents all changes from v1.0.0 baseline.
- Measurement: diff between fork and v1.0.0 tag is documented and intentional.
- No-ship rule: undocumented algorithm changes are `S2`.
- Fallback mode: document the change, review for correctness.
