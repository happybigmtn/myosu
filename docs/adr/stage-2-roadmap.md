# Stage-2 Architecture Roadmap

- Status: Draft support document
- Date: 2026-04-02
- Related: `specs/040226-11-architecture-decision-records.md`, `docs/adr/001-single-token-emission.md`, `docs/adr/002-substrate-fork-strategy.md`, `docs/adr/005-swap-interface-abstraction.md`, `docs/adr/006-commit-reveal-v2.md`, `OS.md`, `THEORY.MD`, `ops/decision_log.md`, `gen-20260402-120305/corpus/plans/013-future-architecture.md`

## Purpose

This document maps the major architectural decisions Myosu is expected to face
after stage 1. It does not make those decisions. Its job is to identify the
decision points, the evidence required before reopening them, the plausible
options, the main risks, and the reversibility of each path.

The intended use is narrow:

- keep stage-2 discussion tied to current repo truth instead of abstract future
  product narratives
- force irreversible decisions to wait for concrete prerequisites
- show which decisions can move independently and which ones should not be
  separated

## Current Baseline

Current accepted architectural truth is still stage-0 and early stage-1 shaped:

- Myosu is single-token in live runtime behavior.
- The chain is intentionally stripped of Frontier/EVM baggage.
- The swap seam is kept only as a stage-0 internal compatibility contract.
- Hash-based commit-reveal v2 is the only live weight-hiding mechanism.
- Local proof already covers poker plus a second game, but not public
  multi-subnet economics or open admission.

That means stage 2 is not "turn everything back on." It is a sequence of
explicit choices about which complexity to reintroduce, in what order, and only
after the current proof surfaces are stable enough to carry the cost.

## Stage-2 Entry Gate

No stage-2 architectural decision should be accepted before all of the
following are true:

- stage-0 exit criteria in `OS.md` are satisfied by executable proof
- multi-node devnet and operator onboarding surfaces from
  `specs/040226-06-multi-node-devnet.md` and
  `specs/040226-07-operator-onboarding.md` are stable enough for non-authors to
  reproduce
- release governance from `specs/040226-08-release-governance.md` is live and
  used for at least one operator-facing release cycle
- the cross-validator determinism and emission accounting proofs remain green
- future-facing specs that expand the game surface are at least directionally
  current, especially `031626-14`, `031626-16`, `031626-18`, and `031626-19`

If those gates are not met, stage-2 discussion should stay in plan/spec form
rather than becoming accepted ADRs.

## Recommended Decision Order

The decisions below are intentionally ordered.

1. Upgrade authority and governance model
2. Validator/miner admission model
3. Multi-subnet emission and routing model
4. Programmable contract surface
5. Dual-token economics

This order keeps the hardest-to-reverse choices last. Governance and admission
define who can change the protocol and who can participate. Multi-subnet routing
defines what the network is optimizing across. Programmable contracts and
dual-token economics should wait until those earlier control surfaces are
proven, because both expand the blast radius of mistakes.

## Decision Map

| ID | Decision point | Earliest honest trigger | Reversibility |
|---|---|---|---|
| `S2-01` | Upgrade authority and governance model | first public operator upgrade or state migration | Moderate |
| `S2-02` | Validator/miner admission model | when curated operator coordination stops scaling | Moderate |
| `S2-03` | Multi-subnet emission and routing model | when more than one subnet needs live economic competition | Moderate |
| `S2-04` | Programmable contract surface | when third parties need chain-resident extensibility | Moderate to hard |
| `S2-05` | Dual-token economics | when single-token incentives cannot support subnet-local markets | Hard |

## S2-01: Upgrade Authority And Governance Model

### Decision question

Who is allowed to approve runtime-changing upgrades once the network has real
external operators, and what proof is required before those changes ship?

### Prerequisites

- `specs/040226-08-release-governance.md` is fully exercised, not just written
- `specs/040226-07-operator-onboarding.md` produces repeatable operator bring-up
- `specs/031626-15-key-management.md` has enough implementation to support
  non-author signing and recovery
- at least one release has crossed the multi-node devnet with truthful upgrade
  notes and rollback procedure

### Options to evaluate

1. Maintainer-led release authority with documented operator consent window
2. Small operator multisig or constitutional committee
3. On-chain governance with voting and timed enactment

### Principal risks

- maintainer-only control does not scale to public trust
- committee governance can deadlock urgent fixes
- on-chain governance before the network has stable social norms can turn
  routine upgrades into protocol theater

### Reversibility

Moderate.

The governance path can change later, but every additional operator and every
live release makes the social migration harder. This decision should be
reopened only if the current upgrade authority repeatedly fails either trust or
speed requirements in live operator use.

## S2-02: Validator And Miner Admission Model

### Decision question

When should Myosu move from curated or bootstrap-oriented participation to open
validator and miner registration, and what anti-sybil controls belong at that
boundary?

### Prerequisites

- `INV-003` determinism proofs stay green under independent operators
- operational RPCs from `specs/031626-18-operational-rpcs.md` exist for
  validator divergence, subnet inspection, and emission visibility
- security audit process from `specs/040226-04-security-audit-process.md` is
  live in CI and operator docs
- key-management and operator docs are good enough that joining does not require
  tribal knowledge

### Options to evaluate

1. Continue curated admission while operator count is small
2. Open admission gated by stake and rate limits
3. Open admission plus additional identity, reputation, or slashing controls

### Principal risks

- open admission too early invites spam, copycat validation, and noisy network
  behavior that stage-0 tooling cannot yet explain
- curated admission too long undermines the core market claim
- slashing or reputation systems added prematurely create governance burden
  before the base scoring loop is socially legible

### Reversibility

Moderate.

A curated network can usually be opened later, but reopening after abuse is
harder. The decision should be revisited only after operator tooling and
divergence visibility are strong enough to debug bad participants without
hand-holding from maintainers.

## S2-03: Multi-Subnet Emission And Routing Model

### Decision question

How should emissions, scoring, and discovery behave once more than one subnet
is economically meaningful at the same time?

### Prerequisites

- the second-game and third-game proofs from `specs/031626-06` and
  `specs/040226-09-third-game-extensibility-proof.md` are live, not only
  architectural
- cross-game scoring work in `specs/031626-16-cross-game-scoring.md` has a
  concrete normalization story
- operational RPCs can expose subnet-local and network-wide state without
  operator archaeology
- release governance can carry storage or runtime-API migrations safely

### Options to evaluate

1. Independent subnet-local emissions with no shared routing logic
2. Shared global emission budget with explicit routing rules across subnets
3. Hybrid model with fixed subnet floors plus performance-based shared upside

### Principal risks

- routing emissions before score normalization is ready can reward the wrong
  games for structural reasons rather than solver quality
- forcing subnet competition too early can make smaller games economically
  impossible to bootstrap
- independent subnet budgets may fragment the network into isolated mini-chains
  in all but name

### Reversibility

Moderate.

Emission weights and routing formulas can be changed, but once operators choose
subnets based on expected rewards, frequent churn damages trust. This decision
should only move once multiple games have live operator demand and the network
can explain why one subnet earns more than another.

## S2-04: Programmable Contract Surface

### Decision question

Should Myosu reintroduce a chain-resident programmable surface for third-party
games, integrations, or settlement logic, and if so, which surface?

### Prerequisites

- the stripped runtime remains stable without Frontier regressions
- `specs/031626-19-game-engine-sdk.md` or an equivalent third-party developer
  story exists off-chain first
- security audit and release governance are mature enough to absorb a much
  larger attack surface
- operator docs and observability can explain failures in the added execution
  environment

### Options to evaluate

1. Stay native-only and keep third-party extensibility off-chain
2. Reintroduce EVM/Frontier for contract compatibility
3. Add a narrower WASM/native plugin surface instead of full EVM restoration

### Principal risks

- EVM restoration reimports a large dependency and security surface that stage 0
  deliberately removed
- native-only extensibility may block useful third-party integrations
- a custom plugin model can be safer than EVM but may strand developers on a
  one-off execution environment

### Reversibility

Moderate to hard.

Adding a programmable surface is easier than removing it once contracts or
plugins hold user expectations and protocol state. This decision should reopen
only when Myosu has clear third-party demand that cannot be served through
runtime APIs, SDK tooling, or off-chain services alone.

## S2-05: Dual-Token Economics

### Decision question

Does Myosu eventually need a second token or subnet-local asset model to align
incentives across multiple games and subnet owners, or should it stay
single-token longer?

### Prerequisites

- ADR 001 remains truthful for the current network and its limits are
  measurable, not speculative
- multi-subnet routing has already been defined well enough to expose the real
  economic pressure points
- release governance can execute storage migrations and operator communication
  safely
- operator and gameplay demand show that single-token incentives are constraining
  actual product or subnet growth rather than theoretical future flexibility

### Options to evaluate

1. Keep the single-token model and delay any second asset
2. Reintroduce a subtensor-like dual-token design with subnet-local economics
3. Design a narrower second asset specifically for subnet funding or routing
   rather than recreating the full inherited Alpha/TAO stack

### Principal risks

- dual-token economics are the easiest way to reimport unnecessary AMM and
  accounting complexity
- staying single-token too long may make independent subnet ownership or
  subnet-local market signaling impossible
- a bespoke second asset can split the difference technically while still
  confusing operators and users

### Reversibility

Hard.

This is the most expensive stage-2 decision to unwind. It affects storage,
staking, runtime APIs, operator mental models, and potentially every emission
proof. It should be the last of these roadmap decisions to move from roadmap to
accepted ADR, and only after the network can point to specific single-token
failures that cannot be addressed with routing or governance changes alone.

## What This Roadmap Deliberately Does Not Decide

- it does not commit Myosu to stage-2 timing
- it does not choose EVM over other programmable surfaces
- it does not promise dual-token economics
- it does not define a final governance constitution
- it does not turn future specs into implied implementation approval

## Review Triggers

Review this roadmap when any of the following become true:

- stage-0 exit criteria are satisfied and a public stage transition is being
  proposed
- a release requires a storage migration or governance exception larger than the
  current process comfortably allows
- more than one subnet has sustained operator demand
- third-party developers ask for on-chain extensibility instead of off-chain SDK
  support
- single-token economics are blamed for a concrete operator or subnet failure
  mode more than once
