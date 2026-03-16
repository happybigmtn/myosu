# Myosu Risk Register

Last updated: 2026-03-16

## Active Risks

### R-001: Subtensor Fork Complexity
- Severity: S2
- Likelihood: High
- Impact: Substrate runtime has thousands of files; extracting game-solving
  pallets while keeping chain functional may be harder than expected.
- Mitigation: start with minimal runtime (no EVM, no swap pallet, no root
  network). Add complexity only when needed.
- Owner: Execution / Dev

### R-002: Robopoker API Stability
- Severity: S2
- Likelihood: Medium
- Impact: v1.0.0 is the first stable release. API may still have rough edges
  that require patching for our use case.
- Mitigation: pin to git tag, document all local patches (INV-006), contribute
  fixes upstream.
- Owner: Execution / Dev

### R-003: Exploitability Computation Cost
- Severity: S2
- Likelihood: Medium
- Impact: full best-response computation for NLHE is expensive. If it can't
  run within a tempo period, validators can't score miners.
- Mitigation: sample-based exploitability (not full game tree traversal),
  configurable sample count, benchmark early.
- Owner: Execution / Dev

### R-004: Validator Determinism
- Severity: S1
- Likelihood: Medium
- Impact: if validators compute different exploitability scores for the same
  strategy, Yuma Consensus produces nonsensical results (INV-003).
- Mitigation: deterministic PRNG seeding, canonical serialization, fixed-point
  arithmetic where possible.
- Owner: Security / Risk

### R-005: Miner Gaming
- Severity: S2
- Likelihood: High
- Impact: miners may memorize common test positions instead of learning
  general strategies.
- Mitigation: commit-reveal test positions, large and varied test sets,
  validators sample from continuous distribution.
- Owner: Strategy
