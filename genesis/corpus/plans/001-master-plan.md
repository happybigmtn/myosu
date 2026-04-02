# 001 - Stage-0 to Stage-1 Master Plan (180-Day Roadmap)

## Purpose / Big Picture

This plan orders the workstreams that take myosu from a working local-loop
prototype (stage-0) to a public multi-node devnet with external operators
(stage-1). The goal is to close the remaining stage-0 exit criteria (INV-003
cross-validator determinism, emission accounting, runtime verification), harden
the foundation, and then open the network.

## Context and Orientation

As of 2026-04-02, myosu has:
- 23K lines of production-quality active crates, CI-gated with 7 jobs.
- 242K lines of inherited chain fork, mostly unused baggage.
- Working local loop: chain -> miner -> validator -> play.
- Multi-game proof: Poker (subnet 2) and Liar's Dice (subnet 3).
- No external users, no deployments, no releases.

The master plan is organized into four phases:

1. **Foundation (Days 1--30):** Close stage-0 gaps, fix CI, reduce chain surface.
2. **Hardening (Days 31--75):** Integration tests, security, observability.
3. **Network (Days 76--120):** Multi-node devnet, operator onboarding.
4. **Product (Days 121--180):** External access, additional games, documentation.

## Architecture

```
Phase 1: Foundation          Phase 2: Hardening
+--002--+  +--003--+         +--005--+  +--006--+
| CI    |  | Chain |         | Secur |  | Integ |
| Sync  |  | Redux |         | ity   |  | Tests |
+-------+  +-------+         +-------+  +-------+
      \      /                      \      /
       +--004--+                     +--007--+
       | Emiss |                     | Doctr |
       | ions  |                     | ine   |
       +-------+                     +-------+

Phase 3: Network             Phase 4: Product
+--008--+  +--009--+         +--011--+  +--012--+
| Multi |  | Oper  |         | Third |  | Pyth  |
| Node  |  | Onbrd |         | Game  |  | on QA |
+-------+  +-------+         +-------+  +-------+
      \      /                      \      /
       +--010--+                     +--013--+
       | Relea |                     | Futur |
       | se    |                     | e     |
       +-------+                     +-------+
```

## Workstream Schedule

### Phase 1: Foundation (Days 1--30)

| Plan | Title | Dependencies | Priority |
|------|-------|-------------|----------|
| 002 | CI and Genesis Plan Sync | None | P0 |
| 003 | Chain Runtime Reduction | None | P0 |
| 004 | Emission Accounting Completion | 003 | P0 |

**Exit gate:** All stage-0 exit criteria verifiable on local devnet. CI green
with updated genesis plan references.

### Phase 2: Hardening (Days 31--75)

| Plan | Title | Dependencies | Priority |
|------|-------|-------------|----------|
| 005 | Security Audit and Observability | 003, 004 | P1 |
| 006 | Integration Test Harness | 004 | P1 |
| 007 | Doctrine Refresh and Staleness Repair | 002 | P2 |

**Exit gate:** End-to-end integration test proves the full loop. Security review
complete with no S0/S1 findings. All doctrine documents current.

### Phase 3: Network (Days 76--120)

| Plan | Title | Dependencies | Priority |
|------|-------|-------------|----------|
| 008 | Multi-Node Devnet | 004, 006 | P0 |
| 009 | Operator Onboarding and Documentation | 007, 008 | P1 |
| 010 | Release Governance and Versioning | 008 | P1 |

**Exit gate:** Two independent operators can run miner + validator against a
persistent devnet. Release process defined and tested.

### Phase 4: Product (Days 121--180)

| Plan | Title | Dependencies | Priority |
|------|-------|-------------|----------|
| 011 | Third Game Integration | 008 | P2 |
| 012 | Python Research Stack QA | None | P2 |
| 013 | Future Architecture and Governance | 010 | P2 |

**Exit gate:** Third game proves continued extensibility. Research stack has
basic CI. Governance process documented.

## Progress
- [x] (pre-satisfied) M1. Local loop proof (chain -> miner -> validator -> play)
  - Surfaces: `crates/myosu-play/`, `crates/myosu-miner/`, `crates/myosu-validator/`
  - Evidence: `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`
- [x] (pre-satisfied) M2. Multi-game architecture proof
  - Surfaces: `crates/myosu-games-liars-dice/`
  - Evidence: `cargo test -p myosu-games-liars-dice --quiet`
- [x] (pre-satisfied) M3. CI pipeline with 7 jobs
  - Surfaces: `.github/workflows/ci.yml`, `.github/scripts/`
  - Evidence: CI passes on trunk
- [x] (pre-satisfied) M4. Operator bootstrap bundle
  - Surfaces: `.github/scripts/prepare_operator_network_bundle.sh`
  - Evidence: `bash .github/scripts/check_operator_network_bootstrap.sh`
- [ ] M5. Phase 1 Foundation complete (all stage-0 exit criteria verifiable)
- [ ] M6. Phase 2 Hardening complete (integration tests, security review)
- [ ] M7. Phase 3 Network complete (multi-node devnet with external operators)
- [ ] M8. Phase 4 Product complete (third game, research QA, governance)

## Surprises & Discoveries

1. Prior genesis produced 21 plans; 12 are fully satisfied by current code.
   This genesis consolidates to 13 plans (001 master + 12 workstreams).
2. CI hardcodes genesis plan filenames -- must be updated before renumbering.
3. The Python research stack is a separate project sharing the repo.

## Decision Log

- Decision: Consolidate 21 prior plans down to 13.
  - Why: 12 prior plans are fully verified against code.
  - Failure mode: Dropping a plan that still has unfinished work.
  - Mitigation: Implementation-status table in ASSESSMENT.md.
  - Reversible: yes

- Decision: Phase chain reduction before emission completion.
  - Why: Emissions depend on runtime configuration. Reducing first prevents
    building on a shifting foundation.
  - Failure mode: Chain reduction takes longer than expected.
  - Mitigation: Time-box to 20 days. If blocked, stub-only and proceed.
  - Reversible: yes

## Validation and Acceptance

The master plan is validated when:
1. Each phase exit gate is met.
2. All numbered plans reach their acceptance criteria.
3. Stage-1 definition is met: public devnet with 2+ external operators.

## Outcomes & Retrospective
_Updated after milestones complete._
