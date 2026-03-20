# `games:poker-engine` Integration

## Integration with `myosu-games`

`myosu-games-poker` is a consumer of `myosu-games` traits:

| From `myosu-games` | Used By | Purpose |
|---|---|---|
| `GameType` | `PokerSolver` | Discriminate NLHE heads-up game variant |
| `GameConfig` | `PokerSolver::new` | Solver configuration |
| `StrategyQuery` | `handle_query` | Miner-validator query envelope |
| `StrategyResponse` | `handle_query` | Miner-validator response envelope |

## Integration with Robopoker

`myosu-games-poker` wraps robopoker types at git rev `04716310143094ab41ec7172e6cea5a2a66744ef`:

| Robopoker Crate | Feature | Types Used |
|---|---|---|
| `rbp-core` | (none) | Base trait markers |
| `rbp-mccfr` | `serde` | `Profile`, `PluribusRegret`, `LinearWeight`, `PluribusSampling` |
| `rbp-nlhe` | `serde` | `NlheSolver`, `NlheInfo`, `NlheEdge`, `NlheEncoder`, `Flagship` |

## Slice 2 Integration Points

Slice 2 (`solver.rs`) is the currently compiled surface:

```
myosu-games::GameType::NlheHeadsUp
         │
         ▼
PokerSolver
  ├─ rbp_nlhe::Flagship  (NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>)
  ├─ NlheProfile
  ├─ NlheEncoder
  └─ checkpoint I/O (MYOS magic + version + bincode)
```

The remaining slices (query, wire, exploit, training) are on disk but not compiled in Slice 2.

## Cross-Crate Dependency Constraint

`rbp-nlhe` and `rbp-mccfr` git rev must match the rev used by `myosu-games`. The workspace enforces this by using the same pinned rev `04716310143094ab41ec7172e6cea5a2a66744ef` across all robopoker consumers.