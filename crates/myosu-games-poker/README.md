# myosu-games-poker

Poker-specific state and TUI integration surfaces for Myosu.

This crate currently provides:

- typed NLHE action parsing
- renderable NLHE snapshot state
- an `NlheRenderer` implementation of `myosu_tui::GameRenderer`
- a profile-backed robopoker query bridge
- request-side lowering into `Partial` and `NlheInfo`
- encoder artifact decoding for abstraction-bucket derivation
- manifest-backed abstraction bundle verification and loading
- robopoker lookup-dump import into manifest-backed encoder directories
- exploitability benchmark helpers for full poker encoder artifacts
- a `PokerSolver` wrapper with checkpoint, query, and training entrypoints

Full registry wiring and proof against a complete abstraction artifact are still
follow-on work.
