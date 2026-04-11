# Design Assessment

Generated: 2026-04-11

## Applicability

Myosu has meaningful user-facing UX, but it is all terminal-based:

- `myosu-play` is the human and agent gameplay surface
- `myosu-miner`, `myosu-validator`, and `myosu-keys` are operator CLIs

There is no web UI, native app, or graphical desktop interface in the current
repo. Design review is therefore about information architecture, terminal
states, error clarity, keyboard-first flows, and machine-consumable output, not
about visual brand or responsive browser layout.

## Gameplay Surface (`myosu-play`)

### Information Architecture

The play surface is structurally clean:

- `myosu-tui::GameRenderer` keeps game-specific rendering isolated from the
  shared shell.
- `crates/myosu-play/src/blueprint.rs` chooses between artifact-backed advice,
  generated fallback advice, and smoke/demo surfaces.
- `crates/myosu-tui/src/renderer.rs` and `events.rs` define a small shared shell
  contract instead of per-game UI duplication.

The main IA win is that all games live inside one shell rather than each game
inventing its own terminal flow.

### States

The code exposes two state layers that matter for UX:

- `InteractionState` in `crates/myosu-tui/src/events.rs`: `Neutral`,
  `Loading`, `Empty`, `Partial`, `Error`, `Success`
- `AdviceStartupState` in `crates/myosu-play/src/blueprint.rs`: `Success`,
  `Empty`, `Partial`

That is materially better than a monolithic “running / crashed” surface, but
the state model is only partly surfaced to users. The shell is capable of
showing partial and empty states; the docs and plans should treat those as
first-class behavior instead of as implementation details.

### User Journeys

The repo currently supports three honest journeys:

1. `cargo run -p myosu-play -- --smoke-test`
   Fastest path to visible success. No chain or local artifacts required.
2. `cargo run -p myosu-play -- train ...`
   Interactive local play with checkpoint-backed or generated advice.
3. `printf 'quit\n' | cargo run -p myosu-play -- pipe`
   Agent-facing text protocol using the same renderer contract.

These journeys are coherent and progressive. The repo does not pretend there is
a polished live-network user flow for all games yet.

### Accessibility and Terminal Responsiveness

Visual browser-style responsiveness is not relevant, but terminal adaptability
is. The shell already handles resize events through `Event::Resize`, and game
renderers express layout needs via `desired_height()`. That is the right model
for a TUI.

What is **not** verified in this pass:

- screen-reader friendliness
- color/contrast choices across terminals
- copy/paste ergonomics for long operator output

Those are open design-quality risks, not proven strengths.

### AI-Slop Risk

Low. The visible gameplay surface is built from structured snapshots, typed
actions, and deterministic renderers. There is no LLM-generated prose driving
the UI.

The one caveat is semantic honesty: portfolio-routed games are explicitly
`engine_tier=rule-aware`, so the UI must keep distinguishing heuristic advice
from trained-solver advice. That is a product-language risk, not a visual one.

## Operator CLI Surface

### Strengths

- The report prefixes in `myosu-miner` and `myosu-validator` are consistent and
  machine-readable enough for line-oriented tooling.
- Error variants in the miner and validator are specific and path-aware.
- `myosu-keys` is cleanly separated and feels like a real operator tool rather
  than helper glue.

### Weaknesses

- There is still no JSON output mode for the major operator binaries.
- Some critical constraints live in docs instead of the CLI contract, for
  example the sparse poker artifacts that block positive-iteration training.
- The downstream agent surface is text-only; there is no typed export mode that
  lines up with the future policy-bundle contract.

## Design Gaps

### Gap 1: No Typed Machine Output Mode

Pipe mode is parseable, and miner/validator reports are structured, but there
is still no canonical JSON output option for automation-heavy workflows.

Why it matters:
Agent consumers and operator tooling have to parse free-form text instead of a
stable schema.

### Gap 2: Partial/Empty/Error States Exist but Are Under-Documented

The code clearly models `Empty`, `Partial`, and `Error`, but the corpus did not
initially treat them as product behavior worth preserving.

Why it matters:
The current repo succeeds partly because it degrades honestly. The plans should
protect that behavior instead of only describing success cases.

### Gap 3: Promotion Provenance Is Not Yet User-Visible

The next product step depends on canonical policy bundles, benchmark summaries,
and sampling proofs, but today those surfaces are not yet visible in the live
CLI/TUI product.

Why it matters:
Without visible provenance, users cannot tell the difference between “rendered
correctly” and “solver-backed credibly.”

## Design Conclusion

The repo does not need a visual redesign. It needs stronger product contracts:

- typed output for automation
- explicit empty/partial/error-state treatment in plans
- visible provenance once promotion work lands

That makes this a light design pass, not a UI rewrite program.
