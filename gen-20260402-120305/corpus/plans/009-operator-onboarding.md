# 009 - Operator Onboarding and Documentation

## Purpose / Big Picture

Before external operators run miners and validators, they need clear
documentation, streamlined setup, and troubleshooting guides. This plan builds
the operator onboarding path from zero to running on devnet.

## Context and Orientation

Current operator experience:
- `myosu-keys print-bootstrap` generates miner/validator commands (CI-verified)
- Operator bundle has startup scripts, chain specs, verification
- Execution playbooks in `docs/execution-playbooks/` are partially stale
- No quickstart guide, no troubleshooting FAQ

## Architecture

```
docs/operator-guide/
├── quickstart.md          # Zero to running in 30 minutes
├── architecture.md        # What each component does
├── troubleshooting.md     # Common issues and fixes
└── monitoring.md          # What to watch
```

## Progress

- [x] (pre-satisfied) M1. Operator bootstrap command exists
  - Surfaces: `crates/myosu-keys/src/main.rs`
Proof command: `MYOSU_KEY_PASSWORD=test cargo run -p myosu-keys --quiet -- create --config-dir /tmp/test-keys --network devnet`

### Milestone 2: Operator quickstart guide

- [ ] M2. Write zero-to-running guide for miner+validator
  - Surfaces: `docs/operator-guide/quickstart.md` (new)
  - What exists after: Prerequisites, key creation, chain sync, miner
    registration, validator staking. Every command works when copy-pasted.
  - Why now: First external operators need this.
Proof command: `test -s docs/operator-guide/quickstart.md`
  - Tests: Manual walkthrough by non-contributor

### Milestone 3: Architecture overview for operators

- [ ] M3. Write non-technical component interaction explanation
  - Surfaces: `docs/operator-guide/architecture.md` (new)
  - What exists after: Diagram-heavy explanation for operators.
  - Why now: Operators need context about what they run.
Proof command: `test -s docs/operator-guide/architecture.md`
  - Tests: Non-developer review

### Milestone 4: Troubleshooting guide

- [ ] M4. Document common failure modes and resolutions
  - Surfaces: `docs/operator-guide/troubleshooting.md` (new)
  - What exists after: FAQ with symptom/cause/resolution for top 10 failures.
  - Why now: Self-service troubleshooting reduces support burden.
Proof command: `test -s docs/operator-guide/troubleshooting.md`
  - Tests: Each failure mode reproduced and resolved

## Surprises & Discoveries

- `print-bootstrap` already generates formatted instructions. Quickstart should
  build on this output, not duplicate it.

## Decision Log

- Decision: Markdown docs (not docs site).
  - Why: <5 operators. Docs site adds overhead for zero benefit now.
  - Failure mode: Markdown gets stale.
  - Mitigation: CI check verifying commands in quickstart.md.
  - Reversible: yes

## Validation and Acceptance

1. Quickstart enables zero-to-running for a new operator.
2. Architecture doc understandable by non-developer.
3. Troubleshooting covers top 10 failure modes.

## Outcomes & Retrospective
_Updated after milestones complete._
