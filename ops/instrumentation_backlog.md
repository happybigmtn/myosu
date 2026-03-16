# Myosu Instrumentation Backlog

Missing truth surfaces that block readiness or measurement.

## Not Yet Instrumented

| Metric | Source | Blocks |
|--------|--------|--------|
| solver_exploitability_convergence | validator consensus | north star measurement |
| validator_determinism | cross-validator comparison | INV-003 enforcement |
| solver_gameplay_separation | cargo tree | INV-004 enforcement |
| miner_training_throughput | miner metrics | capacity planning |
| validator_evaluation_latency | validator metrics | tempo feasibility |
| chain_block_production_rate | chain metrics | basic health |

All metrics require code to exist before instrumentation is possible.
Instrumentation will be added as each stage lands.
