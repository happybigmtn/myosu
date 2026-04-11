# Specification: Developer and Operator Experience

## Objective

Define the current developer experience (DX) for contributors and operators interacting with the myosu project: onboarding paths, first-success expectations, error handling patterns, documentation structure, and the operating model (Fabro/Raspberry). This spec captures the operational surface truthfully enough that a future worker can improve it without guessing.

## Evidence Status

### Verified facts (code-grounded)

#### Onboarding paths

- **Fastest first success:** `cargo run -p myosu-play -- --smoke-test` — validates preflop→flop progression without chain or keys — README.md, AGENTS.md
- **Interactive local play:** `cargo run -p myosu-play -- train` — TUI mode with blueprint bot — README.md
- **Agent protocol test:** `printf 'quit\n' | cargo run -p myosu-play -- pipe` — validates pipe mode — README.md:82, `crates/myosu-play/src/main.rs:460-483`
- **Kuhn smoke test:** `cargo run -p myosu-play -- --game kuhn --smoke-test` — second game proof — CI job
- **Unit test suite:** `SKIP_WASM_BUILD=1 cargo test -p myosu-games --quiet` — no chain compilation needed
- **Full proof surface:** `SKIP_WASM_BUILD=1 cargo test` on 11 active crates — tests all game crates, TUI, miner, validator, play

#### Prerequisites and friction

- `SKIP_WASM_BUILD=1` required for all non-chain builds to avoid 5+ minute WASM compilation — `ci.yml:83`
- `wasm32-unknown-unknown` and/or `wasm32v1-none` Rust targets needed for chain compilation — `ci.yml:95,284`
- `protobuf-compiler` system package needed for chain crates — `ci.yml:238-239`
- Python research QA requires `numpy`, `pytest`, `ruff` installed via pip (no pinned versions) — `ci.yml:68-70`
- Devnet startup takes >60 seconds for JSON-RPC availability — AGENTS.md
- WASM cache at `target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm` must be pre-built for chain operations — AGENTS.md
- Critical caveats are distributed across README.md, AGENTS.md, and OS.md instead of concentrated in one contributor quickstart — README.md:78-115, AGENTS.md:296-346, OS.md

#### Documentation structure (doctrine hierarchy)

1. `SPEC.md` / `specs/` / `plans/` / `fabro/programs/` — what system must become (priority 1)
2. `INVARIANTS.md` — what must never be violated (priority 2)
3. `OS.md` — how the system decides (priority 3)
4. `outputs/` / `ops/` — durable lane artifacts (priority 4)
5. `specsarchive/` — historical only (priority 5)
6. `.raspberry/` / Fabro run state — runtime truth (priority 6)

Sources: AGENTS.md, OS.md

#### Key documentation files

| File | Purpose | Lines |
|------|---------|-------|
| README.md | Orientation, quick start, architecture diagram | ~142 |
| AGENTS.md | System architecture, priorities, engineering decisions, bootstrap doctrine | ~378 |
| OS.md | Operating system/decision logic, stage-0 exit criteria, execution model | ~378 |
| INVARIANTS.md | 6 hard no-ship invariants | — |
| SPEC.md | Spec type definitions and writing rules | — |
| IMPLEMENTATION_PLAN.md | Development roadmap (generated 2026-04-05) | ~153 |
| ARCHIVED.md | Historical review passes with commit hashes | ~74 |

#### Execution model

- **Fabro:** Execution substrate (workflow runner) — OS.md
- **Raspberry:** Control plane (supervision) — OS.md
- **Operator loop:** Three proof surfaces: bootstrap supervision, node-owned stage-0 proof, gameplay/advisor proof — OS.md
- Primary commands: `fabro run` on specific configs — AGENTS.md

#### Error handling patterns

- `KeyError` enum: 15+ variants with specific error contexts — `crates/myosu-keys/src/lib.rs:22-91`
- `MinerBootstrapError`: covers probe, registration, axon, training failures — `crates/myosu-miner/src/main.rs`
- `ChainClientError`: 10+ variants for RPC failures — `crates/myosu-chain-client/src/lib.rs:55-130`
- `CanonicalTruthError`: game-level validation errors — `crates/myosu-games/`
- `TrainingBootstrapError`: encoder loading, artifact completeness checks — `crates/myosu-miner/src/training.rs:76-100`
- `PortfolioEngineError`: portfolio solver failures — `crates/myosu-games-portfolio/`
- Machine-readable report prefixes for each miner/validator bootstrap stage — miner/validator binaries
- Operator binaries print text report blocks directly and expose no `--json`/`--output-format` CLI flag in their current Clap definitions — `crates/myosu-miner/src/main.rs:36-77`, `crates/myosu-validator/src/main.rs:34-83`, `crates/myosu-miner/src/cli.rs`, `crates/myosu-validator/src/cli.rs`

#### Stage-0 exit criteria (from OS.md)

14 must-pass conditions including:
- Chain produces blocks
- Pallet at index 7
- Miner produces MCCFR profile
- Validator computes exploitability
- Two validators identical scores (INV-003)
- Yuma distributes emissions
- Human plays poker against bot
- Training mode works offline
- Solver advisor shows action distribution
- Liar's Dice validates multi-game
- INV-004 (no play→miner dependency)
- Emission accounting sum match
- All 6 invariants pass

### Recommendations (intended system)

- Concentrate critical caveats (WASM cache, sparse artifacts, devnet timing) into a single operator quickstart
- Add typed JSON output mode for miner and validator reports
- Add `--help` output examples to documentation
- Python research dependencies should be managed via `uv` with pinned versions

### Hypotheses / unresolved questions

- Whether Fabro/Raspberry tooling is documented enough for new contributors to use
- Whether the quickstart path works on macOS/Windows or is Linux-only
- Whether there are undocumented environment variables beyond `SKIP_WASM_BUILD` and `MYOSU_KEY_PASSWORD`

## Acceptance Criteria

- `cargo run -p myosu-play -- --smoke-test` succeeds on a fresh clone with `SKIP_WASM_BUILD=1` (no chain, no keys, no artifacts needed)
- `cargo run -p myosu-play -- --game kuhn --smoke-test` succeeds similarly
- `SKIP_WASM_BUILD=1 cargo test -p myosu-games --quiet` runs without compilation errors on a fresh clone with stable Rust
- Error types in all crates provide specific, actionable messages (not generic "something went wrong")
- Miner and validator produce recognizable report prefixes for each bootstrap stage
- README.md links to the correct entry points (OS.md, SPEC.md, INVARIANTS.md)
- All 6 invariants are documented in INVARIANTS.md with enforcement mechanism and no-ship level
- Doctrine hierarchy is consistent between AGENTS.md and OS.md

## Verification

```bash
# First-success path (no chain needed)
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test

# Kuhn first-success
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test

# Test suite without chain
SKIP_WASM_BUILD=1 cargo test -p myosu-games --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-games-kuhn --quiet

# Check documentation files exist
test -f README.md && echo "README.md exists"
test -f AGENTS.md && echo "AGENTS.md exists"
test -f OS.md && echo "OS.md exists"
test -f INVARIANTS.md && echo "INVARIANTS.md exists"
test -f SPEC.md && echo "SPEC.md exists"

# Verify repo shape
bash .github/scripts/check_stage0_repo_shape.sh
```

## Open Questions

1. **Cross-platform support:** All verification commands assume Linux with bash. Does the developer experience work on macOS? The chain compilation requires `protobuf-compiler` which has different install paths.
2. **Fabro/Raspberry documentation:** The execution model references Fabro and Raspberry, but their documentation is not in the repo. Where should new contributors learn these tools?
3. **Caveat consolidation:** Critical operational caveats (WASM cache requirement, sparse artifact limitation, devnet timing, Python dependency management) are scattered. Should there be a single `QUICKSTART.md` or `CONTRIBUTING.md`?
4. **Stage-0 exit tracking:** 14 exit criteria are documented in OS.md. Is there a dashboard or tracking mechanism showing which ones are met vs. pending?
5. **Environment variable inventory:** `SKIP_WASM_BUILD` and `MYOSU_KEY_PASSWORD` are known. Are there other environment variables that affect behavior? A canonical list would help onboarding.
6. **Deleted control surfaces:** AGENTS.md notes that `project.yaml` and `WORKFLOW.md` were deleted (Malinka-only surfaces, do not recreate). Are there other historical files whose absence might confuse new contributors?
