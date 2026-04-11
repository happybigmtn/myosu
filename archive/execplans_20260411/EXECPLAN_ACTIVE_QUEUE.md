# Myosu ExecPlan Active Queue

This file is an index for auto-corpus and auto-loop runs. The executable plans
live beside it in this directory and follow the repository-root `PLANS.md`
format.

Start here:

1. `execplans/EXECPLAN_CANONICAL_GAME_TRUTH_LAYER.md` — complete
2. `execplans/EXECPLAN_CANONICAL_TEN_CORE_GAME_LOGIC.md` — complete
3. `execplans/EXECPLAN_CANONICAL_TEN_PLAY_HARNESS.md` — complete
4. `execplans/EXECPLAN_RESEARCH_SOLVER_STRENGTH_UPGRADE.md` — active

Current state:

- The original canonical-ten queue is complete.
- The same bounded core/playtrace surface now covers all 22 distinct research
  games from `research/game-rules/`.
- Dedicated games are no longer excluded from the replayable core surface:
  `nlhe-heads-up` and `liars-dice` now have local core bootstrap/action
  machines alongside their existing dedicated solver crates.
- The next active queue item is solver quality, not more routing coverage. The
  research portfolio already answers every rules-corpus game, but most
  portfolio engines still consume only metadata-level challenge input. The new
  active ExecPlan upgrades those engines to typed state-aware challenge
  payloads.

The plans are intentionally ordered. First build the shared canonical game
truth model, then use it to add deterministic core state-transition logic for
the canonical ten portfolio games, then build the harness that plays each game
against the best available local strategy. Do not start the play harness before
the truth model has stable state IDs and action IDs; the harness is supposed to
prove that those IDs survive a full game.

The canonical-ten cut for this queue was the ten reference games already
called out by `plans/041026-strong-game-specific-engines.md` as the first
family coverage set: `nlhe-six-max`, `hanafuda-koi-koi`,
`riichi-mahjong`, `bridge`, `gin-rummy`, `stratego`,
`ofc-chinese-poker`, `dou-di-zhu`, `backgammon`, and `cribbage`.
That cut has since been expanded in-place to the full research corpus:
`nlhe-heads-up`, `plo`, `nlhe-tournament`, `short-deck`, `teen-patti`,
`hwatu-go-stop`, `spades`, `liars-dice`, `pusoy-dos`, `tien-len`,
`call-break`, and `hearts` now participate in the same core/playtrace harness.

The bitino reference used for this queue is:

- `/home/r/Coding/bitino/genesis/SPEC.md`
- `/home/r/Coding/bitino/genesis/DESIGN.md`
- `/home/r/Coding/bitino/genesis/ASSESSMENT.md`
- `/home/r/Coding/bitino/execplans/EXECPLAN_CANONICAL_WAGER_TRUTH_AND_VERIFICATION.md`
- `/home/r/Coding/bitino/specs/100426-canonical-wager-system.md`
- `/home/r/Coding/bitino/specs/100426-game-engine-and-settlement.md`
- `/home/r/Coding/bitino/crates/bitino-canonical/src/model.rs`
- `/home/r/Coding/bitino/crates/bitino-canonical/src/games/`

For Myosu, translate bitino's "one canonical wager truth" to "one canonical
decision/action/state truth." A solver action, renderer action, wire action,
validator challenge action, and replay action must all decode to the same
canonical action identity before any game-specific code acts on it.
