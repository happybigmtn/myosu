# Write the demo script Lane — Fixup

Fix only the current slice for `e2e-demo-demo-script`.

Current Slice Contract:
Plan file:
- `genesis/plans/016-e2e-demo.md`

Child work item: `e2e-demo-demo-script`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# End-to-End Demo

**Plan ID:** 016
**Status:** New
**Priority:** CRITICAL — this is the single measure of turnaround success

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the full Myosu pipeline will work end-to-end: miner solves a poker situation → submits weights to chain → validators score the submission → exploitability is computed → chain state reflects the update → TUI shows the updated strategy. A quantitative poker researcher can run this locally and see the decentralized game-solving market in action.

---

## Progress

- [ ] Wire miner → chain submission
- [ ] Wire chain → validator scoring
- [ ] Wire validator → chain weights
- [ ] Wire chain state → TUI display
- [ ] Write the demo script (step-by-step narrative)
- [ ] Run the full demo end-to-end
- [ ] Verify: no panics, no deadlocks, no silent failures

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: The demo runs on local devnet, not testnet or mainnet.
  Rationale: No real tokens or infrastructure exist. The demo is for demonstration purposes only.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Wire miner → chain submission
Miner submits strategy weights to the game-solver pallet.

Proof: `cargo run -p myosu-miner -- --subnet 1 --solve --submit`; chain RPC shows `gameSolver.setWeights` event.

### M2: Wire validator → scoring and submission
Validators score miner submissions and submit weights.

Proof: `cargo run -p myosu-validator -- --subnet 1 --score --submit`; chain shows validator weight submission.

### M3: Wire chain state → TUI display
TUI reads chain state via RPC and displays current best strategy.

Proof: `cargo run -p myosu-play -- --sync-chain` shows the current best bot strategy from the chain.

### M4: Write the demo script
A step-by-step script that a poker researcher can follow in under 10 minutes.

Proof: `cat demo/README.md` documents the full narrative: start chain → register miner → solve → submit → validate → play.

### M5: Run the full demo end-to-end
Follow the demo script from start to finish. No manual interventions. No panics.

Proof: The demo script completes in < 10 minutes and produces a narrative output showing all pipeline steps succeeded.

### M6: Verify INV-003 (deterministic validation)
Two validators scoring the same miner submission produce exploitability scores within epsilon (1e-6).

Proof: Run validator A and validator B on the same miner submission. `diff <(echo $score_a) <(echo $score_b)` shows delta < 1e-6.

---

## Context and Orientation

Full pipeline flow:
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ myosu-miner  │────►│ game-solver │◄────│myosu-validator│
│ (solve +     │     │   pallet    │     │ (score +     │
│  submit)     │     │ (consensus) │     │  submit)     │
└──────────────┘     └──────┬───────┘     └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  myosu-play  │
                    │  (TUI sync)  │
                    └──────────────┘
```

Demo environment:
- Local devnet: `./target/debug/myosu-chain --dev`
- Miner: `cargo run -p myosu-miner`
- Validator: `cargo run -p myosu-validator`
- TUI: `cargo run -p myosu-play -- train`
- RPC: `ws://localhost:9944`

---

## Plan of Work

1. Verify each component works in isolation (P007, P008, P009, P011, P012 complete)
2. Write the demo script with step-by-step commands
3. Run the full pipeline
4. Identify and fix integration failures
5. Time the demo — must complete in < 10 minutes
6. Verify INV-003

---

## Concrete Steps

```bash
# Terminal 1: start devnet
./target/debug/myosu-chain --dev --rpc-port 9933 --ws-port 9944

# Terminal 2: register miner
cargo run -p myosu-miner -- --subnet 1 --register --rpc ws://localhost:9944

# Terminal 2: register validator
cargo run -p myosu-validator -- --subnet 1 --register --rpc ws://localhost:9944

# Terminal 2: miner solves and submits
cargo run -p myosu-miner -- --subnet 1 --solve --iterations 10000 --submit --rpc ws://localhost:9944

# Terminal 3: validator scores and submits
cargo run -p myosu-validator -- --subnet 1 --score --submit --rpc ws://localhost:9944

# Terminal 4: TUI syncs with chain
cargo run -p myosu-play -- train --sync-chain

# Observe: TUI shows the updated best bot strategy
```

---

## Validation

- Demo completes end-to-end without panics or manual intervention
- Miner weight submission appears on chain
- Validator scoring appears on chain
- TUI reflects current chain state
- INV-003 satisfied: validator scores within epsilon
- Demo completes in < 10 minutes


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Step-by-step demo script a poker researcher can follow in under 10 minutes
- How: Document the full narrative: start chain, register miner, solve, submit, validate, play
- Required tests: test -f demo/README.md
- Verification plan: demo/README.md exists and contains step-by-step commands covering the full pipeline
- Rollback condition: Demo script is missing or commands in the script no longer match actual CLI interfaces

Proof commands:
- `test -f demo/README.md`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
