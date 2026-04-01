# Local Advisor Playbook

## Goal

Exercise the currently proven local human/agent gameplay surface in
`myosu-play` without overstating it as a hosted or production service.

## Inputs

- built `myosu-play`
- optional local blueprint artifacts
- optional local chain endpoint if testing discovery-aware smoke

## Steps

1. Start with the built-in smoke:

   ```bash
   SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
   ```

2. Exercise the human-facing shell:

   ```bash
   SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- train
   ```

3. Exercise the agent-facing pipe contract:

   ```bash
   printf 'quit\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe
   ```

4. If you are checking the second-game local surface, use explicit game
   selection:

   ```bash
   SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test --game liars-dice
   ```

## Proof

- `--smoke-test` prints `SMOKE myosu-play ok`
- `train` starts with explicit startup/onboarding state instead of dropping
  directly into ambiguous failure
- `pipe` emits structured startup/status/info/state lines
- second-game smoke works through the shared shell/pipe surface when asked

## Known Constraints

- The primary current truth is still local consumption, not a hosted service.
- Live miner discovery and live HTTP advice are currently poker-only.
- Liar's Dice is a local second-game consumption proof; it does not yet claim
  poker-style live network advice.
