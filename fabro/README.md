# Myosu Fabro Surface

This directory holds the checked-in Fabro execution-plane assets and the
Raspberry control-plane manifest for Myosu.

Execution plane:

- `workflows/` contains Graphviz workflow graphs
- `run-configs/` contains TOML run configs
- `prompts/` contains reusable prompt files
- `checks/` contains proof and readiness helper scripts

Control plane:

- `programs/` contains Raspberry program manifests

Curated lane deliverables do not live under `fabro/`. They live under
`outputs/`, where Raspberry milestones can point at stable artifact paths
without treating Fabro's internal run directories as the durable contract.

Bootstrap intent:

- keep and continue trusted leaf crates such as `myosu-games` and `myosu-tui`
- restart the chain runtime and pallet work as new Fabro-first lanes
- replace Malinka-era task tracking with unit/lane/artifact/milestone control
- promote trusted bootstrap lanes into real implementation lanes once their
  curated `spec.md` and `review.md` artifacts exist

Suggested local entrypoints once the Fabro CLI and Raspberry CLI are wired in:

    fabro run fabro/run-configs/bootstrap/game-traits.toml
    fabro run fabro/run-configs/bootstrap/tui-shell.toml
    fabro run fabro/run-configs/implement/game-traits.toml
    /home/r/coding/fabro/target-local/debug/raspberry plan --manifest fabro/programs/myosu.yaml
    /home/r/coding/fabro/target-local/debug/raspberry status --manifest fabro/programs/myosu.yaml
    /home/r/coding/fabro/target-local/debug/raspberry tui --manifest fabro/programs/myosu.yaml
