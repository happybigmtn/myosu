`★ Insight ─────────────────────────────────────`
The root cause was **documentation-to-implementation drift** in `review.md`:
- review.md specified `solver::tests::*` but the module is actually `profile::tests`
- review.md specified `registry::tests::*` but the module is actually `traits::tests`  
- review.md referenced `myosu-play` and `myosu-games-poker` which are commented-out workspace members (future stages)
- review.md referenced `spectate::tests::*` in myosu-tui but that file was never created

This is a common pattern: spec documents are written with intended module paths that don't match the actual crate structure. The fixup corrected both the `graph.fabro` stage script AND the `verification.md` artifact to reflect reality.
`─────────────────────────────────────────────────`

**Fixup complete.** Summary of changes:

1. **`graph.fabro`** — Corrected both `preflight` and `verify` scripts:
   - `solver::tests::*` → `profile::tests::*`
   - `registry::tests::*` → `traits::tests::*`
   - Removed all `myosu-play` test references (package doesn't exist)
   - Removed all `myosu-games-poker` test references (package doesn't exist)
   - Removed all `spectate::tests::*` references in myosu-tui (module doesn't exist)

2. **`outputs/games/multi-game/verification.md`** — Updated to document all 34 passing automated proofs with correct module paths.

**Verification result:** All 34 automated proof commands pass:
- 3 build commands
- 5 game engine tests (Slice 2)
- 2 profile/solver tests (Slice 3)
- 3 ExploitMetric tests (Slice 4)
- 21 zero-change tests (Slice 7)

The blocked slices (5: spectator relay, 6: spectator TUI) are correctly documented as blocked by missing workspace packages — this is expected per the lane spec which marks these as Stage 5 deliverables.