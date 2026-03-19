# 묘수 myosu

A game-solving subnet chain. Fork of [Bittensor](https://github.com/opentensor/subtensor)
where miners compete to produce optimal game strategies via MCCFR and validators
verify quality through exploitability scoring.

## Start Here

Current doctrine and execution entrypoints:

- [SPEC.md](SPEC.md) for durable repo decisions and migration specs
- [PLANS.md](PLANS.md) for live executable implementation plans
- [specs/031626-00-master-index.md](specs/031626-00-master-index.md) for the
  canonical doctrine index
- [plans/031826-bootstrap-fabro-primary-executor-surface.md](plans/031826-bootstrap-fabro-primary-executor-surface.md)
  for the current Fabro/Raspberry cutover slice
- [fabro/programs/myosu-bootstrap.yaml](fabro/programs/myosu-bootstrap.yaml) as
  the current Raspberry control-plane entrypoint

Execution plane:

- workflows live under `fabro/workflows/`
- run configs live under `fabro/run-configs/`
- prompts live under `fabro/prompts/`
- checks live under `fabro/checks/`

Control plane:

- program manifests live under `fabro/programs/`
- curated lane deliverables live under `outputs/`
- historical-only material lives under `specsarchive/` and `ralph/archive/`

## Architecture

```
Chain (Substrate)     Miners (solvers)     Validators (oracles)     Gameplay (CLI)
┌──────────────┐     ┌──────────────┐     ┌──────────────────┐     ┌───────────┐
│ game-solver  │◄────│ robopoker    │◄────│ exploitability    │     │ human vs  │
│ pallet       │     │ MCCFR        │     │ scoring           │     │ best bot  │
│              │     │              │     │                   │     │           │
│ Yuma         │────►│ emissions    │     │ submit weights    │────►│ query     │
│ Consensus    │     │              │     │ to chain          │     │ best      │
└──────────────┘     └──────────────┘     └──────────────────┘     │ miner     │
                                                                    └───────────┘
```

## Subnets

Each subnet represents a game variant with its own solver market:

| Subnet | Game | Status |
|--------|------|--------|
| 1 | No-Limit Hold'em (Heads-Up) | Planned |
| 2 | No-Limit Hold'em (6-max) | Planned |
| 3 | Short Deck Hold'em | Planned |
| 4 | Pot-Limit Omaha | Planned |
| 5 | Backgammon | Planned |
| 6 | Mahjong (Riichi) | Planned |
| 7 | Bridge | Planned |
| 8 | Liar's Dice | Planned (architecture proof) |

## Core Dependencies

- [robopoker](https://github.com/krukah/robopoker) v1.0.0 — MCCFR poker solver
- [subtensor](https://github.com/opentensor/subtensor) — Substrate chain fork base
- [fabro](https://github.com/fabro-sh/fabro) — staged execution substrate
- Raspberry — Fabro-layered supervisory control plane for units, lanes,
  milestones, and curated outputs

## Development

Built and maintained via Fabro workflows plus Raspberry program supervision.

Current bootstrap loop:

```bash
fabro run fabro/run-configs/bootstrap/game-traits.toml
fabro run fabro/run-configs/bootstrap/tui-shell.toml
fabro run fabro/run-configs/bootstrap/chain-runtime-restart.toml
fabro run fabro/run-configs/bootstrap/chain-pallet-restart.toml

raspberry plan --manifest fabro/programs/myosu-bootstrap.yaml
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
raspberry execute --manifest fabro/programs/myosu-bootstrap.yaml
```

Useful proof commands:

```
cargo test -p myosu-games
cargo test -p myosu-tui
cargo check -p pallet-game-solver   # expected to fail until chain restart lands
```

## Name

묘수 (myosu) — Korean for "brilliant move" or "masterstroke." From the tradition
of Korean strategic gaming culture (baduk, hwatu, StarCraft).

## License

MIT
