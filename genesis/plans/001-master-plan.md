# OS-Driven Stage-0 Master Plan

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

`OS.md` says Myosu is building a decentralized game-solving chain, not a
planning framework. The repo drifted toward doctrine cleanup and Genesis
adjudication work, but the kernel doctrine is clear: stage 0 does not end until
the chain produces blocks, the game-solving pallet runs Yuma, a miner and
validator can participate, and one human can play against the resulting bot.

After this master plan is complete, the active next-step stack will again point
at chain execution. A newcomer should be able to open this file, see the
critical path from runtime strip-down to local devnet to miner/validator loop,
and know which plans are active now versus merely useful later.

## Progress

- [x] (2026-03-28) Re-read `OS.md` and `AGENTS.md` to re-anchor the repo on the
  stage-0 chain mission.
- [x] (2026-03-28) Verified that `cargo check -p myosu-chain-runtime` succeeds
  today, which means the bottleneck is no longer "make anything compile" but
  "strip the chain to the stage-0 architecture we actually want."
- [x] (2026-03-28) Determined that the existing doctrine-cutover master plan was
  over-prioritizing meta work relative to the chain objective in `OS.md`.
- [x] (2026-03-28) Started plan 003 execution by removing Drand, Crowdloan, the
  commitments pallet, and the self-contained extrinsic wrapper from the live
  runtime path while keeping `cargo check -p myosu-chain-runtime` green.
- [x] (2026-03-28) Landed the first reproducible local devnet proof for plan
  004: `myosu-chain --smoke-test` now boots the stripped node, imports blocks,
  and observes local finality.
- [x] (2026-03-28) Verified the live local RPC surface for plan 004:
  `system_health` responds on `127.0.0.1:9944` and
  `neuronInfo_getNeuronsLite` returns data from the running node.
- [x] (2026-03-28) Started plan 005 reduction by removing the live
  crowdloan/leasing and timelocked-weights extrinsics from
  `pallet-game-solver` while keeping `cargo check` and `cargo clippy` green.
- [x] (2026-03-28) Landed the first honest pallet proof for plan 005:
  `cargo test -p pallet-game-solver stage_0_flow --quiet` now passes against a
  reduced default test surface that keeps the stage-0 harness active and parks
  stale subtensor-only unit modules behind `legacy-subtensor-tests`.
- [x] (2026-03-29) Completed the active chain execution stack: plans 003, 004,
  005, and 007 now all have executable proof paths and completed statuses.
- [x] (2026-03-29) Resumed the retained downstream proof plans only after the
  chain loop was materially alive, then closed both 012 and 013 on honest
  executable surfaces.
- [x] (2026-03-30) Closed doctrine/governance cleanup plans 014 through 018
  locally, including the Genesis corpus sync that rewrote the report and
  tightened stale assessment statements to the current adjudicated reality.
- [x] (2026-03-30) Closed plan 019 locally by writing future Genesis synth
  governance into `genesis/PLANS.md`, including launch procedure, provider
  policy, and adjudication-before-merge discipline.

## Surprises & Discoveries

- Observation: The runtime compiles even though it still carries the very
  surfaces the doctrine says must be stripped.
  Evidence: `cargo check -p myosu-chain-runtime` succeeds, but
  `crates/myosu-chain/runtime/src/lib.rs` still wires `pallet_drand`,
  `pallet_crowdloan`, `fp_self_contained`, Frontier/EVM, and the swap pallet.

- Observation: The earlier audit's exact first blocker has shifted.
  Evidence: `crates/myosu-chain/pallets/game-solver/src/macros/config.rs`
  no longer requires `pallet_drand::Config + pallet_crowdloan::Config`; the
  surviving blocker is now the runtime and pallet surface that still expose
  drand, crowdloan, CRV3 timelock, and swap-heavy paths.

- Observation: We do not need a fresh plan namespace to refocus.
  Evidence: Existing plans 003, 004, 005, and 007 already correspond to the
  runtime, node, pallet, and miner/validator layers named in `OS.md`; they
  simply needed to be rewritten to match the real code and current priorities.

## Decision Log

- Decision: Reuse the existing chain plan slots instead of inventing more
  meta-plans.
  Rationale: `003`, `004`, `005`, and `007` already map cleanly to the real
  chain execution path, so a truthful rewrite is better than adding more plan
  surface.
  Date/Author: 2026-03-28 / Codex

- Decision: The active stack is now chain-first, not doctrine-first.
  Rationale: `OS.md` and `AGENTS.md` both define stage 0 in terms of chain,
  miner, validator, and gameplay exit criteria.
  Inversion: Continuing to lead with documentation or Genesis governance work
  would keep burning cycles above the actual product bottleneck.
  Date/Author: 2026-03-28 / Codex

- Decision: Runtime strip-down is destructive, not feature-gated by default.
  Rationale: The repo doctrine says "replace, don't deprecate" and the stage-0
  target is a smaller fork, not a permanently dual-path runtime.
  Inversion: Carrying a reversible "full runtime" mode would preserve exactly
  the complexity the chain fork is supposed to remove.
  Date/Author: 2026-03-28 / Codex

## Outcomes & Retrospective

The reset succeeded. The chain-first stack was executed instead of collapsing
back into doctrine churn: runtime reduction (003), node/devnet proof (004),
pallet reduction (005), and miner/validator/bootstrap proof (007) all closed
on executable evidence. Once that core loop was materially alive, the retained
downstream proof work also moved from aspirational to real: the multi-game
additive proof (012) and node-owned integration harness (013) are now closed
on honest branch surfaces.

## Context and Orientation

The controlling documents are:

- `OS.md`, which says the product is a decentralized game-solving chain and
  describes the four-layer architecture: chain, solvers, validators, gameplay.
- `AGENTS.md`, which names the stage-0 work inventory and the chain fork
  critical path: strip the runtime, minimize the node, finish the game-solving
  pallet, then bring up miner and validator.
- `crates/myosu-chain/runtime/src/lib.rs`, which still includes Drand,
  Crowdloan, swap, Frontier/EVM, and self-contained Ethereum extrinsics.
- `crates/myosu-chain/pallets/game-solver/`, which is the live pallet fork that
  already dropped the old `pallet_drand`/`pallet_crowdloan` supertrait but
  still carries CRV3 timelock and swap-era logic internally.
- `crates/myosu-chain/node/`, which must become a minimal Aura/Grandpa devnet
  node.
- `Cargo.toml`, where `myosu-miner` and `myosu-validator` are active workspace
  members and part of the current stage-0 proof surface.

The completed and next-phase stack is now:

```text
COMPLETED

003  Strip runtime to the stage-0 chain core
004  Minimize node and prove local devnet block production
005  Reduce pallet-game-solver to the stage-0 Yuma/staking/weights surface
006  Game traits and poker boundaries
007  Bring up myosu-chain-client, myosu-miner, and myosu-validator
008  Artifact / wire / checkpoint hardening
009  Productize the poker play/TUI surface
002  Spec corpus normalization
012  Prove additive multi-game architecture with Liar's Dice
013  Add a node-owned end-to-end stage-0 integration harness
014  Refresh OS/operator docs around the truthful bootstrap loop
015  Retire Malinka from the active control plane
016  Cut over the bootstrap-first Fabro/Raspberry control plane
017  Rationalize Fabro workflow/program/run-config surfaces
018  Adjudicate the Genesis corpus to the current completed/active/queued truth
020  Prove a full second-game subnet execution path with Liar's Dice

ACTIVE NEXT PHASE

010  Expand CI proof gates across chain and doctrine surfaces
hosted Actions now exist and the remote-drift blocker is cleared on the draft
PR branch; the first full hosted run exposed concrete fixable blockers instead
of vague pending work: `myosu-play` startup-state tests that depended on local
artifact discovery, plus missing `protoc` and deny-Clippy cleanup in the chain
lane. The next step is publish those fixes and capture one hosted green timing
run

NEXT QUEUED DOCTRINE

019  Future synth genesis governance
```

## Completed Plans

| # | Plan | Role | Depends On |
|---|------|------|------------|
| 002 | Spec Corpus Normalization | Clean the active spec namespace, restore empty canonicals, and sync stale review/control-plane docs to current code truth. | none |
| 003 | Strip Runtime to Stage-0 Chain Core | Remove drand, crowdloan, Frontier/EVM, and swap-heavy runtime baggage while keeping `GameSolver` at runtime index 7. | none |
| 004 | Minimize Node for Working Devnet | Build a node that starts, authors blocks, and serves game-solver RPC on a local devnet. | 003 |
| 005 | Reduce Pallet Game-Solver to Stage-0 Surface | Keep Yuma, staking, registration, serving, and commit-reveal v2; remove CRV3 timelock and AMM-era baggage. | 003 |
| 006 | Harden Game Traits and Poker Engine Boundaries | Enforce the literal gameplay/miner boundary, lock the public growth seam, and prove additive custom-game extensibility. | none |
| 007 | Bootstrap Miner, Validator, and Shared Chain Client | Bring up the first actual off-chain participants against the stripped chain. | 003, 004, 005 |
| 008 | Artifact / Wire / Checkpoint Hardening | Harden artifact loading, bounded decode, mmap validation, and checkpoint/header trust boundaries. | 006 |
| 009 | Productize Play + TUI Experience | Turn the local poker surface into a resilient stage-0 product with explicit startup states, onboarding, responsive layout tiers, and keyboard-usable shell behavior. | 006, 008 |
| 011 | Security Audit, Observability, and Release Governance | Add grounded stage-0 audit doctrine, release gating, service timing logs, and chain/node health summaries. | 008, 010 |
| 012 | Multi-Game Architecture Proof | Prove a second game can land additively through the shared game seam without reaching into poker code. | 006 |
| 013 | Integration Test Harness | Turn the owned stage-0 smoke into a cargo-managed contract with cheap fixture regressions and an ignored live wrapper. | 007 |
| 014 | OS Refresh and Operator Docs | Rewrite `OS.md`, sync `README.md`, and tighten durable playbooks around the truthful bootstrap, local advisor, and node-owned stage-0 loop. | none |
| 015 | Retire Malinka and Cut Over to No-Autodev Doctrine | Archive the remaining root-level legacy executor surface and scrub active references so the live repo no longer advertises that execution model. | 014 |
| 016 | Fabro / Raspberry Bootstrap Control-Plane Cutover | Make `myosu-bootstrap.yaml` the explicit primary entrypoint in control-plane docs and distinguish bootstrap `outputs/` roots from secondary portfolio roots. | 014, 015 |
| 017 | Rationalize Fabro Workflow and Program Surfaces | Classify the checked-in Fabro programs, workflows, and run-config families so the execution substrate reads as intentional instead of uneven. | 016 |
| 018 | Genesis Corpus Adjudication and Downstream Selection | Rewrite the Genesis report and stale assessment language so the corpus reflects the current completed/active/queued doctrine. | 014, 015, 016, 017 |
| 019 | Future Synth Genesis Governance | Document the launch procedure, provider order, fallback posture, and adjudication-before-merge rule for all future `fabro synth genesis` runs. | 018 |
| 020 | Second-Game Subnet Execution Proof | Extend the additive Liar's Dice seam into a full second-game miner/validator/play proof with a passing owned two-subnet coexistence harness. | 011, 012 |

## Active Next-Step Plans

| # | Plan | Why now |
|---|------|---------|
| 010 | Expand CI Proof Gates to Chain and Doctrine Surfaces | The code and spec surfaces are now broad enough that CI blind spots are a bigger risk than feasibility gaps. |

## Next Queued Plan

No queued follow-on plan is being promoted right now. Once the remote-only
`010` hosted proof lands against the current repo surface, reprioritize from
the then-current repo state instead of auto-promoting another dormant plan
slot.

Plan 020 is now complete locally. The second game has wire transport, bounded
miner/validator execution, a local play surface in `myosu-play`, and a passing
owned two-subnet coexistence proof on the local chain.

## Deferred Plans

Plans `014` through `019` are now complete locally. No additional
doctrine/governance plan is currently being promoted ahead of the remote-only
`010` closure check.

## Milestones

### Milestone 1: Strip the chain to what stage 0 actually needs

Goal: `myosu-chain-runtime` and `pallet-game-solver` stop pretending to be a
general subtensor fork and become a stage-0 game-solving chain.

Plans: 003 and 005.

Exit proof:

    cargo check -p myosu-chain-runtime
    cargo test -p pallet-game-solver stage_0_flow --quiet
    rg -n "pallet_drand|pallet_crowdloan|fp_self_contained" crates/myosu-chain/runtime/src/lib.rs

Status:
- Completed on 2026-03-29.

### Milestone 2: Produce blocks on a minimal local devnet

Goal: the node starts in local dev mode, authors blocks, and serves the
game-solver RPC surface needed by the next layer.

Plan: 004.

Exit proof:

    cargo run -p myosu-chain -- --smoke-test
    curl -s -H "Content-Type: application/json" \
      -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
      http://localhost:9944

Status:
- Completed on 2026-03-29.

### Milestone 3: Bring up the first real network participants

Goal: one miner can train and serve strategy, one validator can score it and
submit weights, and the repo begins behaving like a network rather than a local
demo.

Plan: 007.

Exit proof:

    cargo check -p myosu-chain-client -p myosu-miner -p myosu-validator
    cargo test -p myosu-validator inv_003_determinism --quiet

Status:
- Completed on 2026-03-29.

### Milestone 4: Prove the stage-0 loop and additive multi-game seam

Goal: turn the live local loop into a cargo-owned integration contract and
prove that a second game crate can land additively through the shared game
boundary.

Plans: 012 and 013.

Exit proof:

    cargo test -p myosu-games-liars-dice --quiet
    cargo test -p myosu-chain --test stage0_local_loop --quiet

Status:
- Completed on 2026-03-29 on the honest branch surface. On this machine the
  node test uses the same `SKIP_WASM_BUILD=1` proof path as the stripped
  runtime checks because `wasm32-unknown-unknown` is not installed.

### Milestone 5: Close downstream hardening and corpus drift

Goal: finish the remaining non-core-but-still-live hardening work so the code,
spec corpus, and review surfaces stop drifting apart.

Plans: 006, 008, and 002.

Exit proof:

    cargo test -p myosu-games --quiet
    cargo test -p myosu-games-poker --quiet
    SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
    python - <<'PY'
    from pathlib import Path
    import re
    specs = sorted(p.name for p in Path('specs').glob('031626-*.md'))
    index = Path('specs/031626-00-master-index.md').read_text()
    refs = sorted(set(re.findall(r'031626-\d{2}[a-z]?-[a-z-]+\.md', index)))
    allowed = set(refs) | {'031626-00-master-index.md'}
    extra = sorted(set(specs) - allowed)
    missing = sorted(allowed - set(specs))
    print('EXTRA', extra)
    print('MISSING', missing)
    PY

Status:
- Completed on 2026-03-29. The stronger consistency rule is now "every active
  `031626-*` file in `specs/` is backed by the master index, and every
  master-index target is present and non-empty."

### Milestone 6: Productize poker and harden the ship path

Goal: turn the working poker stage-0 surface into a more resilient and
operator-friendly product surface, then add CI and release discipline around
it before moving deeper into the multi-game thesis.

Plans: 009, 010, and 011.

Exit proof:

    cargo test -p myosu-play -p myosu-tui --quiet
    test -s ops/security-audit-stage0.md
    test -s ops/release-gate-stage0.md
    grep -q "chain-core\\|Doctrine Integrity\\|Plan Quality" .github/workflows/ci.yml .github/workflows/*.yml

Status:
- 009 completed on 2026-03-29.
- 011 completed on 2026-03-29.
- 010 remains the only active hardening plan, pending GitHub-hosted timing
  proof. The workflow is published and the current repo shape now reaches the
  long hosted lanes. The remaining blockers are concrete and fixable:
  environment-dependent `myosu-play` startup tests plus missing `protoc` and
  node deny-Clippy cleanup in the chain lane.
- 020 completed locally on 2026-03-30, including the owned two-subnet
  coexistence proof for poker and Liar's Dice.

## Plan of Work

1. Keep the completed chain-first stack documented as finished, not merely
   assumed.
2. Finish the remaining 010 timing proof so the hardening phase is actually
   closed, not just locally green.
3. Treat 020 and 019 as finished locally and keep the repo focused on the
   remote-only 010 closure check until a new explicit reprioritization happens.
4. Do not quietly smuggle deferred work back into the active stack without an
   explicit reprioritization decision.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '111,220p' OS.md
    sed -n '131,239p' AGENTS.md
    cargo check -p myosu-chain-runtime
    rg -n "pallet_drand|pallet_crowdloan|fp_self_contained|Swap: pallet_subtensor_swap" \
      crates/myosu-chain/runtime/src/lib.rs

## Validation and Acceptance

This master plan is accepted when:

- `001` reflects the completed plans truthfully instead of presenting them as
  still active.
- The chain plan files themselves no longer describe completed work as active.
- 010 is explicitly surfaced as the only remaining active hardening plan, with
  second-game deepening next after it.
- Deferred doctrine/governance work is explicitly separated from both the
  closed stage-0 stack and the chosen next phase.

## Idempotence and Recovery

This rewrite is document-only and safe to rerun. If the operator later chooses
to reopen doctrine or Genesis governance work, that should appear as an
explicit master-plan decision rather than ambient drift.

## Interfaces and Dependencies

```text
003 runtime strip-down  --->  004 node devnet
        |                          |
        v                          |
005 pallet stage-0 surface --------+
        |
        v
007 chain client + miner + validator
        |
        +--> 013 integration harness
        |
        +--> 006 game-boundary hardening ---> 012 additive second-game proof
        |
        +--> 008 artifact/wire hardening
        |
        +--> 002 spec/review corpus normalization
```
