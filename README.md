# 묘수 myosu

A game-solving subnet chain. Fork of [Bittensor](https://github.com/opentensor/subtensor)
where miners compete to produce optimal game strategies via MCCFR and validators
verify quality through exploitability scoring.

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
- [malinka](https://github.com/happybigmtn/malinka) — autonomous development framework

## Development

Built and maintained autonomously via [malinka](https://github.com/happybigmtn/malinka).

```
cargo test                          # run all tests
cargo clippy -- -D warnings         # lint
cargo build --release               # build
```

## Name

묘수 (myosu) — Korean for "brilliant move" or "masterstroke." From the tradition
of Korean strategic gaming culture (baduk, hwatu, StarCraft).

## License

MIT
