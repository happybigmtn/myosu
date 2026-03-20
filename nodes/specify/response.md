Both required artifacts are now in place.

`★ Insight ─────────────────────────────────────`
The foundations lane is a **meta-bootstrap** surface — it doesn't own any code itself, but produces the honest control-plane assessment that tells Raspberry and contributors which of the 4 bootstrap lanes are trustworthy, which need repair, and which need restart. The key finding is that `games:multi-game` has a **false submit**: its `review.md` says "KEEP" based on a spec-level judgment, but the actual `myosu-games-liars-dice` crate doesn't exist — so the control plane would be deceived if it trusted artifact presence over implementation truth.
`─────────────────────────────────────────────────`

**What was produced:**

- **`outputs/foundations/spec.md`** — Honest frontier status covering: (1) `games:traits` fully trusted, (2) `tui:shell` partially trusted (4 modules KEEP, 3 REOPEN), (3) `chain:runtime` needs RESTART, (4) `chain:pallet` needs RESTART, (5) `games:multi-game` is a false submit (spec reviewed but implementation not started). Defines 5 slices ordered by dependency and severity.

- **`outputs/foundations/review.md`** — Keep/reopen/restart judgment table for all 5 lanes, 5 concrete risks the implementation lanes must address, proof commands for each lane (noting that the current chain scripts are surface-only checks that don't prove compilation), and a direct mapping of the two active frontier tasks to the identified risks.