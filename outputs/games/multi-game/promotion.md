# `games:multi-game` Promotion — All Slices Complete

## Readiness Checklist

- [x] All 7 slices implemented and passing
- [x] All 121 tests pass across 4 crates
- [x] Zero-change property verified (no existing crate modified)
- [x] `implementation.md` created
- [x] `verification.md` created
- [x] `quality.md` created
- [x] All 22 proof commands from `review.md` exit 0
- [x] Build succeeds (`cargo build -p myosu-games -p myosu-games-liars-dice -p myosu-play -p myosu-tui`)

## Artifacts Produced

| Artifact | Location | Status |
|----------|----------|--------|
| Implementation record | `outputs/games/multi-game/implementation.md` | Done |
| Verification record | `outputs/games/multi-game/verification.md` | Done |
| Quality report | `outputs/games/multi-game/quality.md` | Done |
| Promotion checklist | `outputs/games/multi-game/promotion.md` | This file |

## AC Coverage

| AC | Description | Status |
|----|-------------|--------|
| MG-01 | Liar's Dice game engine runs from dice roll to challenge | ✓ Verified |
| MG-02 | Solver trains to exploitability < 0.001 | ✓ Verified |
| MG-03 | All existing tests pass without changes | ✓ Verified |
| CS-01 | ExploitMetric registered for all GameType variants | ✓ Verified |
| SP-01 | Spectator relay emits JSON events | ✓ Verified |
| SP-02 | Spectator TUI renders fog-of-war view | ✓ Verified |

## Promotion Action

Merge the `fabro/run/01KM6J42E4BMK1M9GCQCQ9WZKG` branch into `main`. The branch is clean (no conflicts with `main`).

```bash
git checkout main
git merge fabro/run/01KM6J42E4BMK1M9GCQCQ9WZKG
```

## Post-Promotion

After merging to `main`:

1. **games:traits lane** — No action needed. This lane is orthogonal and additive.
2. **games:poker-engine lane** — No action needed. Independent of this lane.
3. **services:miner lane** — Will eventually serve `LiarsDiceGame` queries. Not blocked by this lane.
4. **services:validator-oracle lane** — Can now integrate `ExploitMetric` for cross-game weight normalization.
5. **product:play-tui lane** — Spectator mode is implemented. Can proceed with integration.
