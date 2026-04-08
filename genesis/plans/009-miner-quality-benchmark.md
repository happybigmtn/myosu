# 009 — Miner Quality Benchmark Surface

## Objective

Create a truthful benchmark surface that measures miner strategy quality independently of the current self-scoring validator path. This unblocks the miner convergence research gate (F-007) and provides operators with quality guidance.

## Context

The current validator scoring path (`score_response()` in `validation.rs`) compares the observed miner response against `solver.answer(query)` from the checkpoint supplied on the validator CLI. When the same checkpoint is used for both miner and validator, the result is always `exact_match=true, score=1.0`. This is a determinism proof, not a quality benchmark.

Additionally, the checked-in poker bootstrap artifacts are intentionally sparse — any positive-iteration poker training fails upstream with `isomorphism not found`.

WORKLIST.md `MINER-QUAL-001` and IMPLEMENTATION_PLAN.md `F-007` both document this blocker.

**Two paths forward:**

1. **Exploitability-based benchmark** — Use robopoker's `Profile::exploitability()` method directly. This computes the exact exploitability of a trained profile. Requires a full encoder (not the sparse bootstrap). Works for any game that implements `Profile`.

2. **Independent reference checkpoint** — Train a reference checkpoint offline, ship it as a test fixture, and score miners against it. This only proves relative quality, not absolute.

Option 1 is more truthful but requires either shipping richer encoder artifacts or documenting the hardware requirements (7-11 GB RAM for full poker encoder).

## Acceptance Criteria

- A benchmark command or test exists that measures strategy quality independent of the self-scoring path
- For Liar's Dice (which has a complete native solver with no encoder dependency): the benchmark produces an exploitability score for a trained checkpoint after N iterations
- For poker: either (a) richer encoder artifacts are shipped as test fixtures, or (b) the benchmark documents that poker quality measurement requires the full encoder and specifies the hardware requirements
- WORKLIST.md `MINER-QUAL-001` is updated with the chosen benchmark approach
- Minimum recommended training iterations are documented for at least Liar's Dice (poker may remain open if encoder artifacts are not yet available)

## Verification

```bash
# Liar's Dice quality benchmark
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- quality_benchmark
# Should report exploitability decreasing with training iterations

# Documentation check
test -f docs/adr/012-miner-quality-benchmark.md || test -f WORKLIST.md
# Updated with approach decision
```

## Dependencies

- Plan 008 (test gap closure) — ensures test infrastructure is in good shape before adding a benchmark suite
