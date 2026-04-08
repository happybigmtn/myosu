# Poker Quality Benchmark

## Goal

Produce a poker exploitability benchmark that does not depend on the validator
same-checkpoint scoring path.

This flow is for the remaining poker-only follow-up after the completed
Liar's Dice `quality_benchmark`. It uses the robopoker clustering pipeline to
materialize a full `isomorphism` lookup in PostgreSQL, converts that lookup
into Myosu's manifest-backed encoder directory, and then measures exploitability
through `PokerSolver::exploitability()`.

## Why This Exists

The checked-in bootstrap artifacts are intentionally sparse. They are enough for
bounded smoke proofs, but any positive poker training iteration still fails
upstream with `isomorphism not found`.

The truthful poker benchmark path therefore needs the full abstraction, not the
repo-owned bootstrap fixture.

## Hardware

Use the robopoker system requirements as the generation floor:

- clustering/training host: `16 vCPU`, `120 GB RAM`
- PostgreSQL host: `8 vCPU`, `64 GB RAM`

Use the Myosu encoder-loading audit as the local import/benchmark floor:

- importing and benchmarking the manifest-backed encoder currently assumes a
  multi-gigabyte local artifact load budget
- the full in-memory encoder has historically required about `7-11 GB RAM`
  inside Myosu

The repo now sets the local poker artifact decode budget to `16 GiB` in
[artifacts.rs](/home/r/coding/myosu/crates/myosu-games-poker/src/artifacts.rs)
so full river and merged encoder artifacts are no longer rejected at `256 MiB`.
This is separate from the `1 MiB` network wire budget for miner/validator
payloads.

## One-Command Path

Run the repo-owned wrapper:

```bash
bash ops/poker_quality_benchmark.sh \
  --db-url postgres://user:pass@127.0.0.1:5432/robopoker \
  --robopoker-dir /path/to/robopoker \
  --encoder-dir "$PWD/target/poker-full-encoder"
```

Optional: override the benchmark ladder.

```bash
bash ops/poker_quality_benchmark.sh \
  --db-url postgres://user:pass@127.0.0.1:5432/robopoker \
  --robopoker-dir /path/to/robopoker \
  --encoder-dir "$PWD/target/poker-full-encoder" \
  --iterations "0 128 256 512 1024"
```

The script does four things:

1. Reuses an existing robopoker `isomorphism` table if one is already present,
   otherwise runs `cargo run -p trainer -- --cluster` in the supplied
   robopoker checkout.
2. Streams `SELECT obs, abs FROM isomorphism ORDER BY obs` through `psql`.
3. Converts that lookup dump into a Myosu manifest-backed encoder directory via
   `import_robopoker_lookup`.
4. Runs the Myosu poker exploitability benchmark via `quality_benchmark`.

## Manual Surfaces

If you need to run the steps separately:

```bash
cd /path/to/robopoker
DB_URL=postgres://user:pass@127.0.0.1:5432/robopoker cargo run --quiet -p trainer -- --cluster
```

```bash
psql postgres://user:pass@127.0.0.1:5432/robopoker -At -F $'\t' \
  -c 'SELECT obs, abs FROM isomorphism ORDER BY obs' \
  | env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker \
      --example import_robopoker_lookup -- - "$PWD/target/poker-full-encoder"
```

```bash
env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker \
  --example quality_benchmark -- "$PWD/target/poker-full-encoder" 0 128 256 512
```

Expected output shape:

- `LOOKUP_IMPORT encoder_dir=...`
- `POKER_BENCHMARK iterations=0 exploitability=...`
- `POKER_BENCHMARK iterations=128 exploitability=...`

## Current Truth

- The repo now has an actionable poker benchmark path that is independent of
  validator self-scoring.
- The repo still does **not** check in a full encoder fixture.
- The repo still does **not** publish a minimum poker training floor, because
  that recommendation should come from a real run on the full abstraction
  rather than the intentionally sparse bootstrap artifacts.

When an operator completes a benchmark run on full artifacts, record the
measured exploitability ladder before narrowing `F-007` or the operator docs to
a fixed poker minimum.
