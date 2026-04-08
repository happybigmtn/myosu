# Stage-0 Local Loop Playbook

## Goal

Run the smallest honest Myosu chain proof on one machine. The preferred surface
is the node-owned local loop, which now proves:

- a local node starts and authors blocks
- poker subnet registration, miner bootstrap, validator permit, weights, and
  gameplay smoke
- Liar's Dice subnet registration, miner bootstrap, validator permit, weights,
  and local gameplay smoke
- positive miner incentive, validator dividend, and miner emission for both
  games after the Bob -> Alice weight write

## Preferred Proof

Use the cargo-managed test wrapper first:

```bash
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
```

If you need the direct executable proof and summary output:

```bash
env SKIP_WASM_BUILD=1 "${CARGO_TARGET_DIR:-target}/debug/myosu-chain" --stage0-local-loop-smoke
```

## Proof

The direct smoke should end with:

- `STAGE0 myosu-chain local-loop ok`
- `poker_subnet=2`
- `poker_bob_weights=[(0, 65535)]`
- `poker_gameplay_final_state=complete`
- positive `poker_alice_miner_incentive`
- positive `poker_bob_validator_dividend`
- positive `poker_alice_miner_emission`
- `liars_dice_subnet=3`
- `liars_dice_bob_weights=[(0, 65535)]`
- `liars_dice_gameplay_final_state=static_demo`
- positive `liars_dice_alice_miner_incentive`
- positive `liars_dice_bob_validator_dividend`
- positive `liars_dice_alice_miner_emission`

## Manual Breakdown

Use the manual service-by-service path only when you need to isolate a specific
seam the node-owned proof does not already explain.

Useful component commands:

```bash
cargo run -p myosu-chain -- --smoke-test
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
cargo run -p myosu-games-poker --example bootstrap_artifacts -- /tmp/myosu-bootstrap-encoder /tmp/myosu-bootstrap-query.bin
```

If you drill into miner or validator behavior manually, keep the node-owned
proof as the source of truth for what the full loop is supposed to establish.

## Known Constraints

- The smoke entrypoints require a fresh embedded runtime wasm. If the cached
  runtime wasm is older than the runtime sources, the smoke fails fast instead
  of reporting misleading chain behavior.
- On this machine the normal cargo/test path still uses `SKIP_WASM_BUILD=1`
  for day-to-day proof; refreshing the embedded runtime wasm requires a
  wasm-capable toolchain path.
- The local chain cannot advertise `127.0.0.1` as an axon IP, so chain-visible
  miner endpoints can still appear as `0.0.0.0:<port>` while gameplay
  normalizes the actual connection to `127.0.0.1:<port>`.
- The preferred stage-0 story is the node-owned proof. Manual miner and
  validator bring-up is diagnostic support, not the primary operator flow.
