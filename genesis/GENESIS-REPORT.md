# Genesis Report

Date: 2026-04-05
Commit: `b1be135` on `trunk`

---

## Corpus Summary

This genesis refresh produced 6 documents and 14 numbered plans:

| File | Purpose |
|------|---------|
| [ASSESSMENT.md](ASSESSMENT.md) | What the project is, what works, what's broken, gaps |
| [SPEC.md](SPEC.md) | System specification grounded in code |
| [DESIGN.md](DESIGN.md) | User-facing surfaces: TUI, pipe, operator CLI, RPC |
| [PLANS.md](PLANS.md) | Plan index with sequencing rationale |
| [GENESIS-REPORT.md](GENESIS-REPORT.md) | This file |
| [plans/001-master-plan.md](plans/001-master-plan.md) | Master plan and phase structure |
| [plans/002-014](plans/) | 13 implementation and research plans |

---

## Major Findings

### 1. The Local Loop Is Proven

The repo's central claim -- that a stripped chain, miner, validator, and gameplay
can connect as one honest local proof -- is **verified against code**. The E2E
test suite (`local_loop.sh`, `validator_determinism.sh`, `emission_flow.sh`)
and pallet-level tests (`stage_0_flow.rs`) prove the full path. Multi-game
architecture is proven with Liar's Dice and Kuhn poker requiring zero changes
to existing code.

### 2. Inherited Complexity Is the Primary Risk

The `pallet-game-solver` pallet carries ~200K lines of code inherited from
Bittensor's subtensor, including:
- A complete copy of the original `pallet-subtensor` (with 44 duplicated test files)
- ~194 storage items (only ~80 needed)
- AMM/dual-token/root-network logic behind an identity-swap stub
- Epoch/coinbase code that is 4x more complex than the stage-0 model requires

This inherited mass is not broken -- it compiles, tests pass -- but it creates:
- Audit burden (which pallet is authoritative?)
- CI slowness (duplicated test suites)
- Confusion for new contributors
- False attack surface (dormant storage items in metadata)

### 3. Emission Accounting Is the Weakest Proven Surface

The emission invariant (`sum(distributions) == block_emission * epochs`) is
asserted in `stage_0_flow.rs` but the assertion surface is thinner than the
coinbase code complexity suggests it should be. The `emission_flow.sh` E2E
test exists but exercises a narrow path. This is the highest-priority
hardening target.

### 4. Multi-Node Behavior Is Partially Proven

Two-node sync and peer discovery are proven (`two_node_sync.sh`). GRANDPA
finality across 3+ nodes, node restart resilience, and cross-node emission
agreement are not yet tested. These are the bridge from "local proof" to
"operator-ready network."

### 5. Python Research Layer Is Disconnected

The 125K-line Python layer (`main.py`, `methods.py`, etc.) is a research/simulation
tool with CI coverage (ruff + pytest) but no connection to the Rust codebase.
It has no dependency management. It is not blocking stage-0 but adds repo
size and maintenance surface without clear integration path.

---

## Recommended Direction

**Close stage-0 by reducing inherited complexity, hardening emission accounting,
and proving multi-node behavior. Then package for operators.**

The four-phase approach:

1. **Reduce and Harden** (Plans 002-005): Delete the pallet copy, harden emissions,
   clean up tests, audit storage. This is 2-3 focused sessions of work.

2. **Network Proof** (Plans 006-008): Three-node devnet with finality and
   cross-node emission agreement. This is 2-3 more sessions.

3. **Decision Gate** (Plan 009): Evaluate results before packaging.

4. **Operator Packaging** (Plans 010-012): Containers, documentation, release process.

Research gates (Plans 013-014) run in parallel and inform post-stage-0 direction
without blocking stage-0 exit.

---

## Top Next Priorities

### Immediate (start now)

1. **Plan 002: Delete pallet-subtensor copy** -- Highest leverage single action.
   Removes ~150K lines of duplicated source, unblocks all subsequent chain work.

2. **Plan 003: Emission accounting hardening** -- The weakest proven surface.
   Add the accounting invariant test, gate dead coinbase paths.

### Near-term (after 002-003)

3. **Plan 004: Test deduplication** -- Clean the test suite to match the reduced surface.

4. **Plan 006: Multi-node devnet** -- Can start in parallel with 004-005 if
   a second worker is available.

### Background (parallel)

5. **Plan 013: Token economics research** -- Does not block code work.

---

## Not Doing List

These items are explicitly out of scope for the current planning horizon:

| Item | Reason |
|------|--------|
| **Game portfolio expansion** | Three games (poker, Liar's Dice, Kuhn) are sufficient proof of multi-game architecture |
| **Web/mobile gameplay** | TUI and pipe serve stage-0 users. Web is a product decision for post-stage-0 |
| **Full AMM token economics** | The identity swap is the correct stage-0 model. Research gate (013) covers future direction |
| **Production deployment** | Stage-0 is devnet-only |
| **Upstream polkadot-sdk migration** | Research gate (014) only. Not an implementation commitment |
| **Governance mechanisms** | Post-stage-0 concern |
| **Runtime upgrade/migration paths** | Not needed until production |
| **Benchmarking and weight calibration** | Not needed until production |
| **Public testnet operations** | Devnet-only until stage-0 exits |
| **Python research integration** | The Python layer serves a different purpose; no forced integration |
| **Fabro/Raspberry dependency** | Plans do not assume or require these tools |
| **Hardware optimization** | Full encoder (7-11 GB) requirements are documented, not optimized |

---

## Comparison with Previous Genesis Snapshot

The previous planning snapshot (archived at `.auto/fresh-input/genesis-previous-20260405-064750`)
proposed plans through 021 (operator hardening and network packaging). Key differences:

| Area | Previous | Current |
|------|----------|---------|
| Stage-0 local loop | Claimed proven | **Verified against code** |
| Multi-game proof | Claimed proven | **Verified: Liar's Dice + Kuhn** |
| Emission hardening | Identified as gap | **Promoted to Plan 003 with concrete acceptance criteria** |
| Dead code removal | Not explicitly planned | **New Plan 002: delete pallet-subtensor copy** |
| Multi-node devnet | In plan 008 | **Plans 006-008 with explicit gate** |
| Container packaging | Not planned | **New Plan 010** |
| Token economics | Noted as future | **Explicit research gate (Plan 013)** |
| SDK migration | Not discussed | **Explicit research gate (Plan 014)** |
| Decision gates | None | **Plan 009 as explicit checkpoint** |

The current corpus is more conservative in scope (14 plans vs 21+) but more
explicit about verification, dependencies, and decision points. Every plan
has concrete acceptance criteria and verification commands.

---

## Risk Summary

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Emission accounting breaks at network scale | Medium | High | Plan 003 (local proof) + Plan 008 (cross-node proof) |
| GRANDPA finality issues with 3+ nodes | Low | High | Plan 007 (explicit finality proof) |
| Inherited code has undiscovered dependencies | Low | Medium | Plan 002 verification via `cargo check --workspace` |
| Identity swap stub masks economic bugs | Medium | Medium | Plan 013 (research gate) addresses future model |
| Single developer velocity | High | Medium | Plans are independently closeable, enabling parallel work |
| Substrate build times slow iteration | High | Low | Container caching, SKIP_WASM_BUILD for off-chain crates |
