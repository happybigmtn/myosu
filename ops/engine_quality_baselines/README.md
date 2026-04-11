# Engine Quality Baselines

This directory records the stable text contract for the portfolio engine
quality and latency budget examples. It is intentionally lightweight: the
workspace does not use a dedicated benchmark harness yet, so the current
surface is ordinary Rust examples that print key-value lines and exit non-zero
on budget failure.

Commands:

- `SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example engine_quality_budget`
- `MYOSU_ENGINE_BUDGET_MS=50 SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example engine_latency_budget`
- `MYOSU_ENGINE_BUDGET_MS=50 SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_roundtrip -- bridge target/e2e/strength-budget-bridge 32`

Current budget fields:

- `ENGINE_QUALITY` compares each portfolio rule-aware engine to the static
  compatibility baseline for the same typed challenge state. The default
  `MYOSU_ENGINE_MIN_SCORE` is `1.01`, so the budget requires either a
  different recommended action or a different action distribution from the
  baseline.
- `ENGINE_LATENCY` measures one typed strength query and quality report for
  each portfolio game.
- `STRENGTH budget_status` is `pass`, `fail`, or `skipped`; it is skipped only
  when `MYOSU_ENGINE_BUDGET_MS` is not configured.

The first default latency budget is 50 ms per representative heuristic query
on a development machine. That is a guardrail for the current compact
rule-aware engines, not a production SLA.
