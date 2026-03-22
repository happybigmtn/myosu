# Implement validator registration with chain Lane — Review

Review only the current slice for `validator-binary-validator-registration`.

Current Slice Contract:
Plan file:
- `genesis/plans/012-validator-binary.md`

Child work item: `validator-binary-validator-registration`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Validator Binary Implementation

**Plan ID:** 012
**Status:** New
**Priority:** HIGH — core chain participant

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the validator binary (`myosu-validator`) will connect to the local devnet, register as a validator, score miner strategy submissions via exploitability computation, and submit weights to the chain. Validators are the verification layer — they ensure miners are producing genuinely better strategies.

---

## Progress

- [ ] Create `crates/myosu-validator/` crate scaffold
- [ ] Implement validator registration with chain
- [ ] Implement exploitability scoring using `myosu-games`
- [ ] Implement weight verification and submission
- [ ] Implement hotkey management
- [ ] Test end-to-end: validator registers → scores miners → submits weights

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Validators compute exploitability, not correctness.
  Rationale: Per INV-003, validators must agree within epsilon (1e-6) on exploitability scores. They verify that a strategy is near-Nash, not that it plays optimally.
  Date/Author: 2026-03-21 / Interim CEO

---

## Milestones

### M1: Create `crates/myosu-validator/` scaffold
Set up the crate with CLI interface.

Proof: `cargo build -p myosu-validator` produces a binary that responds to `--help`.

### M2: Implement validator registration
Register the validator on the target subnet.

Proof: `cargo run -p myosu-validator -- --subnet 1 --register` succeeds and the chain shows the validator registered.

### M3: Implement exploitability scoring
For each miner submission, compute exploitability using `myosu-games::remote_poker_exploitability()`.

Proof: `cargo run -p myosu-validator -- --subnet 1 --score` produces a numeric exploitability value.

### M4: Implement weight verification and submission
Verify miner weights satisfy Yuma Consensus, submit validator weight set.

Proof: After scoring, `cargo run -p myosu-validator -- --subnet 1 --submit-weights` succeeds.

### M5: End-to-end validator workflow
Register → receive miner weights → score → submit weights → receive emission.

Proof: A validator that scores miners and submits weights receives a positive emission within one epoch.

---

## Context and Orientation

The validator binary pattern:
```
myosu-validator
├── CLI: --subnet, --hotkey, --register, --score, --submit
├── Validator registration via subtensor RPC
├── Exploitability scoring via myosu-games
└── Weight submission via game-solver pallet tx
```

Per `specs/031626-02b-poker-engine.md`, the key function is `remote_poker_exploitability(profile, game_state) -> f64`. This must be deterministic across validators.

---

## Validation

- `cargo build -p myosu-validator` passes
- `cargo run -p myosu-validator -- --help` shows all CLI options
- `cargo run -p myosu-validator -- --subnet 1 --register` succeeds
- `cargo run -p myosu-validator -- --subnet 1 --score --miner <uid>` produces exploitability value
- Two validators scoring the same miner produce values within epsilon (1e-6)


Workflow archetype: implement

Review profile: hardened

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Validator registration module in myosu-validator
- How: Register the validator on a target subnet via subtensor RPC
- Required tests: cargo test -p myosu-validator -- registration
- Verification plan: Running --register succeeds and the chain shows the validator registered
- Rollback condition: Registration fails or chain does not reflect validator

Proof commands:
- `cargo build -p myosu-validator`
- `cargo run -p myosu-validator -- --subnet 1 --register`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check state transitions that affect balances, commitments, randomness, payout safety, or replayability
- check secret handling, capability scoping, pairing/idempotence behavior, and privilege escalation paths

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful
