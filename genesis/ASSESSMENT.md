# Assessment

Generated: 2026-04-11
Codebase snapshot: `trunk @ 4e0b37fbaa` plus working tree

## Inspection Scope

This pass was based on direct repo inspection only.

### Files and surfaces inspected directly

- repo control/docs: `AGENTS.md`, `README.md`, `PLANS.md`, `WORKLIST.md`,
  `IMPLEMENTATION_PLAN.md`, `docs/operator-guide/quickstart.md`
- current corpus: `genesis/*.md`, `genesis/plans/*.md`
- core runtime/chain: `Cargo.toml`, `crates/myosu-chain/runtime/src/lib.rs`,
  `crates/myosu-chain/node/src/chain_spec/mod.rs`, `command.rs`
- current gameplay/UX: `crates/myosu-play/src/cli.rs`,
  `crates/myosu-play/src/blueprint.rs`, `crates/myosu-tui/src/events.rs`,
  `crates/myosu-tui/src/renderer.rs`
- solver/validation paths: `crates/myosu-miner/src/training.rs`,
  `crates/myosu-validator/src/validation.rs`,
  `crates/myosu-games-canonical/src/lib.rs`,
  `crates/myosu-games-portfolio/src/lib.rs`,
  `crates/myosu-games-portfolio/src/game.rs`
- security/tooling: `.github/workflows/ci.yml`,
  `.github/scripts/check_plan_quality.sh`

### Lightweight commands run

- corpus/file layout: `find genesis ...`, `rg` over repo and corpus headings
- plan and spec verification: `rg` for ExecPlan sections, runtime index, audit
  ignores, wire-codec usage, research-game counts, and current CLI/report
  strings
- repo metadata: `git rev-parse --short=10 HEAD`, `git status --short`

### Not done in this pass

- no long cargo builds or E2E suites
- no sibling `../bitino/` repo inspection
- no assumption of truth from generated corpus prose

## Verified Current State

| Area | Verified fact | Evidence |
|------|---------------|----------|
| Runtime | `SubtensorModule` sits at runtime index `7` | `crates/myosu-chain/runtime/src/lib.rs` |
| Validator | Current score path compares against validator-loaded checkpoint expectation | `crates/myosu-validator/src/validation.rs` |
| Poker training | Positive-iteration poker training rejects `postflop_complete=false` artifact sets | `crates/myosu-miner/src/training.rs` |
| Poker artifacts | Checked-in summaries still expose sampled postflop coverage rather than full trainable coverage | `crates/myosu-games-poker/src/artifacts.rs` via `rg` |
| Portfolio corpus | `ALL_RESEARCH_GAMES.len() == 22`, `ALL_PORTFOLIO_ROUTED_GAMES.len() == 20` | `crates/myosu-games-portfolio/src/lib.rs` |
| Canonical crate | `myosu-games-canonical` currently owns canonical-ten/playtrace surfaces; no `policy.rs` exists yet | `crates/myosu-games-canonical/src/lib.rs` |
| Gameplay UX | `myosu-play` supports smoke-test, train, and pipe modes | `crates/myosu-play/src/cli.rs` |
| TUI shell | Shared shell has explicit `Neutral/Loading/Empty/Partial/Error/Success` states | `crates/myosu-tui/src/events.rs` |
| Security gate | CI suppresses 19 audit IDs today, while `WORKLIST.md` tracks 12 active `SEC-001` items | `.github/workflows/ci.yml`, `WORKLIST.md` |
| Plan tooling | Live repo plan check still expects `Acceptance/Gate Criteria` and `Verification` headings | `.github/scripts/check_plan_quality.sh` |

## What Works

- The repo has a coherent stage-0 operator stack: chain, miner, validator,
  gameplay, and key management are all represented as checked-in crates and
  docs.
- The terminal product surface is credible. `myosu-play`, `myosu-miner`,
  `myosu-validator`, and `myosu-keys` all expose structured, non-placeholder
  operator or player flows.
- The research-game portfolio routing is real and code-backed, not just
  speculative documentation.
- The canonical layer already has enough shape to justify adding policy-bundle
  types there rather than inventing another crate immediately.

## What Is Incomplete or Risky

| Surface | Repo-grounded status | Why it matters |
|---------|----------------------|----------------|
| NLHE promotion | Blocked on stronger benchmark/provenance than the sparse checked-in artifacts | Prevents honest `promotable_local` claims |
| Validator quality story | Same-checkpoint self-check only | Yuma-style quality claims outrun current proof strength |
| Security debt | 19 ignored audit advisories in CI, with direct `bincode` usage still live | Supply-chain and codec trust risk |
| Same-TUI downstream work | External dependency not inspected in this pass | Cannot be an honest executable Myosu plan yet |
| Canonical policy bundle | Not implemented | Blocks the whole promotion stream |

## Security and Test Risk

### Security

- The highest-signal owned-code issue is still direct `bincode 1.3.3` usage in
  the game wire/checkpoint paths.
- Most of the remaining ignored advisories appear inherited from the chain fork
  or adjacent tooling, but they are still part of the live audit surface and
  should not be hand-waved as “already handled.”

### Test risk

- This review did not re-run the long proof commands, so harness references were
  treated as repo claims unless backed by the code paths directly inspected.
- The validator happy path is tested, but it is a determinism test shape rather
  than an independent-quality test shape.
- The generated corpus originally under-described the risk that NLHE promotion
  could appear green while still depending on non-trainable sparse artifacts.

## Documentation and Tooling Staleness

| Surface | Staleness / mismatch | Why it matters |
|---------|----------------------|----------------|
| `AGENTS.md` | References `@RTK.md`, but no repo-root `RTK.md` exists | Control-surface ambiguity |
| Generated corpus | Claimed agent-based review work and direct Bitino implementation planning | Overstated inspection scope |
| Plan tooling | Repo CI still checks old plan headings while corpus used only the new ones | Docs/tooling drift that would make the corpus look invalid |
| Product docs | Quickstart is honest overall, but still relies on operators knowing sparse-artifact and slow-local-devnet caveats from deeper docs | DX friction |

## Assumption Ledger

| ID | Assumption | Status | Notes |
|----|------------|--------|-------|
| A1 | The repo's long E2E proofs are currently green | Not re-run in this pass | Treated as repo claim, not freshly verified |
| A2 | Bitino can consume Myosu policy bundles with limited adaptation | Unverified in this review pass | Later grounded by the active root master plan, not by this document alone |
| A3 | NLHE can reach `promotable_local` without a large checked-in artifact set | Plausible but unproven | Requires the pinned external-dossier path to land first |
| A4 | Cribbage is still the best first portfolio target | Reasonable default | No contrary code evidence found in this pass |

## Focus Response

The user focus was “what we need to do to make the `001-master-plan` come into
reality.” The repo supports that direction, but with two important corrections:

1. the next honest work starts with policy/promotion grounding before any
   sibling-repo adapter work
2. NLHE promotion must depend on stronger dossier evidence than the sparse
   checked-in artifact path

Those are not stylistic differences. They are the difference between a corpus
that sounds plausible and one that is actually grounded.

## Opportunity Framing

### Recommended direction

Follow the revised plan queue:

1. policy bundle contract
2. promotion ledger
3. policy/ledger checkpoint
4. NLHE dossier unblock
5. NLHE promotion
6. Liar's Dice promotion
7. dedicated-game checkpoint
8. parallel security triage
9. cribbage deepening
10. Bitino local adapter and same-TUI pilot
11. same-TUI pilot checkpoint

### Rejected direction 1: Treat ungrounded direct Bitino implementation as part of this review pass

Rejected because this review did not inspect the sibling repo and would have
made the plans look executable while depending on unreviewed code. That
sequence constraint does not invalidate the later grounded Bitino milestone in
the active root master plan.

### Rejected direction 2: Promote NLHE using only the checked-in sparse artifacts

Rejected because the repo itself documents and enforces that those artifacts are
not a truthful positive-iteration training surface.

### Rejected direction 3: Put all effort into security before the promotion work starts

Rejected as the only ordering, not as unimportant work. Security triage should
run in parallel, but the next product-learning loop is still promotion
credibility.

## DX Assessment

### First-run experience

The repo has a real “time to first success” path:

- `cargo run -p myosu-play -- --smoke-test`

That is a legitimate strength. It keeps the repo from feeling purely
infrastructural.

### Main DX friction points

- WASM-target and `SKIP_WASM_BUILD` knowledge are still prerequisite lore.
- The quickstart is honest, but critical caveats are distributed across README,
  quickstart, and `AGENTS.md` instead of being concentrated where operators
  first fail.
- There is still no JSON output mode for the major operator CLIs.

### DX conclusion

This is a developer- and operator-facing repo with a meaningful learn-by-doing
path. The work it needs most is not a new onboarding document; it is stronger
contract surfaces so docs, plans, and tooling stop drifting apart.
