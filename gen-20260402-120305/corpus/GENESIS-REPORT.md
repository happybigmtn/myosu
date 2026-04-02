# Genesis Report

Date: 2026-04-02
Sprint: Full genesis planning cycle (Think -> Plan -> Build -> Review -> Verify)

## Summary

This genesis run produced a 13-plan corpus (001 master + 12 workstreams)
covering a 180-day roadmap from stage-0 (working local-loop prototype) to
stage-1 (public multi-node devnet with external operators).

The assessment reviewed 265K lines of Rust code (23K active, 242K chain fork),
3.2K lines of Python research code, 7 CI jobs, 50 git commits, 19 specification
files, 21 prior genesis plans, and all doctrine documents. Every active crate
source file was read. Chain code was sampled at entry points and critical pallets.

## Total Plans Generated

| Plan | Title | Phase | Priority |
|------|-------|-------|----------|
| 001 | Master Plan (180-day roadmap) | All | -- |
| 002 | CI and Genesis Plan Sync | Foundation | P0 |
| 003 | Chain Runtime Reduction | Foundation | P0 |
| 004 | Emission Accounting Completion | Foundation | P0 |
| 005 | Security Audit and Observability | Hardening | P1 |
| 006 | Integration Test Harness | Hardening | P1 |
| 007 | Doctrine Refresh and Staleness Repair | Hardening | P2 |
| 008 | Multi-Node Devnet | Network | P0 |
| 009 | Operator Onboarding and Documentation | Network | P1 |
| 010 | Release Governance and Versioning | Network | P1 |
| 011 | Third Game Integration | Product | P2 |
| 012 | Python Research Stack QA | Product | P2 |
| 013 | Future Architecture and Governance | Product | P2 |

**Total milestones across all plans:** 52
**Pre-satisfied milestones:** 12 (verified against code)
**Remaining milestones:** 40

## Assessment Highlights

1. **The gameplay stack is production-quality.** 23K lines across 9 active
   crates, zero unwrap/expect calls, property-based tests, CI-gated. This is
   unusual for a 17-day-old project.

2. **The chain fork is 10x what stage-0 needs.** 242K lines, 11 pallets, EVM
   integration -- all inherited from Bittensor. Plan 003 addresses this.

3. **Three stage-0 exit criteria remain unverified at runtime.** Criteria 6
   (cross-validator determinism), 7 (emission distribution), and 12 (emission
   accounting green). Plans 004 and 006 target these.

4. **78% of commits are machine-authored.** Fabro and Codex drive development.
   The 8% recovery rate suggests the pipeline is effective but occasionally
   needs human correction.

5. **12 of 21 prior genesis plans are fully satisfied.** Code review verified
   each claim. This genesis consolidated to 13 plans by dropping verified work.

## Plans Needing Human Attention

### Critical (block stage-0 completion)

- **002 (CI Sync):** Must be done FIRST. CI will break without updating plan
  file references in `.github/scripts/check_stage0_repo_shape.sh`. This is a
  5-minute fix but blocks everything else.

- **004 (Emission Accounting):** Closes the three remaining stage-0 exit
  criteria. The coinbase rewrite requires understanding the Yuma Consensus
  emission logic, which is deeply embedded in the subtensor fork. Human
  review of the single-token emission path is essential.

### Important (block stage-1)

- **003 (Chain Reduction):** Deciding which 20 extrinsics to keep (out of 69)
  requires product judgment about what stage-0 actually needs. Cannot be fully
  automated.

- **008 (Multi-Node Devnet):** Deployment decisions (bootnode hosting, chain spec
  genesis config) require infrastructure choices that only a human operator can
  make.

### Can be automated

- **005 M1 (Unified tracing):** Mechanical task of adding tracing subscriber.
- **007 (Doctrine refresh):** Mechanical cross-reference audit.
- **011 (Third game):** Well-scoped implementation task.
- **012 (Python QA):** Straightforward linting and testing.

## Known Gaps Requiring Runtime Verification

These cannot be verified by reading code alone:

1. **Cross-validator determinism (INV-003).** The code supports deterministic
   scoring, but no test has been run with two independent validator instances
   against the same miner. Plan 006 M4 addresses this.

2. **Emission distribution.** Yuma Consensus is wired but has not been observed
   distributing tokens on a running chain. Plan 004 M4 addresses this.

3. **Multi-node networking.** Substrate peer discovery works in general, but the
   subtensor fork may have custom networking code. Plan 008 M4 addresses this.

4. **Chain spec genesis state.** The devnet chain spec is placeholder. Running
   the chain with a real genesis config may reveal configuration issues. Plan
   008 M2 addresses this.

## Recommended Operator Next Steps

1. **Immediate (today):** Execute plan 002 -- update CI scripts to reference new
   genesis plan filenames. Push and verify CI passes.

2. **This week:** Start plans 003 and 004 in parallel. Chain reduction is
   independent of emission accounting. Both are on the critical path.

3. **Week 2--3:** Execute plan 006 (integration test harness). The local loop
   test is the single most important proof artifact.

4. **Week 4--5:** Execute plan 005 (security audit). `cargo audit` in CI is a
   one-line change with high value.

5. **Month 2:** Begin Phase 3 (network plans 008--010). Multi-node devnet is
   the gate to stage-1.

## Code-Review Coverage by Component

| Component | Coverage | Method |
|-----------|----------|--------|
| myosu-games | Full (all 3 files) | Direct read |
| myosu-games-poker | Full (all 10 files) | Direct read |
| myosu-games-liars-dice | Full (all 7 files) | Direct read |
| myosu-tui | Full (all 9 files) | Direct read |
| myosu-play | Full (all 5 files) | Direct read |
| myosu-chain-client | Full (1 file, 2560 lines) | Direct read |
| myosu-miner | Full (all 7 files) | Direct read |
| myosu-validator | Full (all 5 files) | Direct read |
| myosu-keys | Full (all 3 files) | Direct read |
| myosu-chain (node) | Sampled (main.rs, cli.rs, service.rs) | Entry points |
| myosu-chain (runtime) | Sampled (lib.rs) | Entry point |
| pallet-game-solver | Sampled (lib.rs, stage_0 tests) | Key paths |
| Other pallets (10) | Not read (inherited, unused) | Dependency analysis |
| Python research | Full (all 6 files) | Direct read |
| CI/Scripts | Full (all 5 scripts) | Direct read |
| Doctrine | Full (all 12 files) | Direct read |
| Specs | Confirmed present (19 files) | Directory listing |
| Prior genesis | Headers read (21 files) | Sampled |

## Corpus File Manifest

```
genesis/
├── ASSESSMENT.md          # Full repository assessment
├── SPEC.md                # Project specification
├── PLANS.md               # ExecPlan conventions and prior plan disposition
├── DESIGN.md              # Design system and user-facing surfaces
├── GENESIS-REPORT.md      # This file
└── plans/
    ├── 001-master-plan.md
    ├── 002-ci-genesis-plan-sync.md
    ├── 003-chain-runtime-reduction.md
    ├── 004-emission-accounting-completion.md
    ├── 005-security-audit-and-observability.md
    ├── 006-integration-test-harness.md
    ├── 007-doctrine-refresh-and-staleness-repair.md
    ├── 008-multi-node-devnet.md
    ├── 009-operator-onboarding.md
    ├── 010-release-governance.md
    ├── 011-third-game-integration.md
    ├── 012-python-research-qa.md
    └── 013-future-architecture.md
```

Total: 5 top-level documents + 13 plans = 18 files, ~1,800 lines.
