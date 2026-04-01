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
