# Productize Play + TUI Experience

Status: Completed 2026-03-29. The stage-0 poker play/TUI surface now has the
explicit startup, onboarding, layout, and edge-case handling this plan was
meant to land.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: Enhanced from `archive/genesis_1774729423/plans/009-play-tui-productization.md`. Changes: preserved implementation-ready IA mockups and interaction states; added concrete code change targets, main.rs decomposition task, and responsive test definitions.

## Purpose / Big Picture

The local play surface works but stage-0 requires a polished, resilient training and advisor experience. This plan handles: explicit loading/empty/error/success/partial states, first-run onboarding, edge-case input handling, accessibility (keyboard-only, contrast, focus order), responsive layout tiers, and decomposition of the 31K-line main.rs.

After this plan, a first-time user can run `myosu-play train`, see clear startup feedback, play a hand with tab-completion and inline help, and get actionable error messages when artifacts are missing.

## Progress

- [x] (2026-03-28) Audited myosu-play, myosu-tui, and poker renderer for state coverage.
- [x] (2026-03-29) Landed the first `myosu-play` decomposition slice by
  extracting CLI and discovery-request parsing into
  `crates/myosu-play/src/cli.rs`, reducing `main.rs` from 1501 to 1407 lines
  without changing smoke behavior.
- [x] (2026-03-29) Landed the second `myosu-play` decomposition slice by
  extracting blueprint artifact discovery/loading and advice-selection logic
  into `crates/myosu-play/src/blueprint.rs`, reducing `main.rs` from 1407 to
  1077 lines while keeping smoke and test behavior green.
- [x] (2026-03-29) Started the explicit startup/onboarding-state lane by
  mapping artifact resolution to `success` / `empty` / `partial` startup
  states, logging actionable startup guidance in `train`, and emitting a
  structured `STATUS startup_state=...` line in `pipe`.
- [x] (2026-03-29) Added a shared shell interaction-state model in
  `myosu-tui`, so declaration text now has first-class `success` / `partial` /
  `error` rendering instead of relying only on transcript messages. `myosu-play`
  now maps startup advice state into that shell model.
- [x] (2026-03-29) Landed the first responsive-layout slice in `myosu-tui`:
  narrow terminals now collapse the state panel and show transcript tail only,
  desktop terminals place transcript and state side-by-side, and `train`
  renders a visible loading frame before startup context resolution finishes.
- [x] (2026-03-29) Made transcript rendering state-aware and completed compact
  tier coverage. Non-neutral shell states now inject explicit status/detail
  lines into the transcript, empty mode shows a concrete first-run onboarding
  hint, and compact-width tests now prove the stacked transcript-over-state
  layout explicitly.
- [x] (2026-03-29) Wired live miner connectivity transitions into the shared
  shell state model. Live advice refresh now emits structured status updates as
  well as transcript messages, so fresh/stale/offline miner transitions become
  explicit `success` / `partial` / `error` UI states during training.
- [x] (2026-03-29) Tightened keyboard/onboarding polish. The shell input line
  now windows long commands around the cursor and renders a visible focus
  marker, while first-run artifact messaging now explains that generated advice
  still works and gives clearer `--checkpoint` / `MYOSU_BLUEPRINT_DIR`
  recovery paths.
- [x] (2026-03-29) Closed the explicit interaction-state taxonomy for the
  shared shell: loading, empty, error, success, and partial are now all
  first-class shell states with declaration and transcript treatment.
- [x] (2026-03-29) Added a real first-run onboarding flow for empty artifact
  startup. `train` now starts on the onboarding screen when no artifacts are
  found, and onboarding completion routes cleanly into the lobby or directly
  into the demo hand.
- [x] (2026-03-29) Closed the remaining edge-case bucket. Startup state now
  promotes zero-result miner discovery and failed live miner query into
  explicit `partial` shell/pipe states, while the shared shell truncates
  overlong header labels cleanly instead of letting narrow terminals render
  awkward wrapped text.
- [x] (2026-03-29) Implemented responsive layout tiers for desktop
  (`>=120`), compact (`80-119`), and narrow (`<80`) terminals.
- [x] (2026-03-29) Closed the keyboard accessibility slice for the current
  shell surface: tab completion, line-edit controls, visible input focus, and
  input readability on narrow terminals are all exercised directly.
- [x] (2026-03-29) Added direct tests covering the interaction-state surface
  and viewport/layout behavior across shell and input code.

## Surprises & Discoveries

- Observation: the old 31K-line claim was stale by the time plan 009 actually
  resumed. `main.rs` was large enough to justify decomposition, but the live
  file was 1501 lines before the first extraction slice, not 31,337.
  Evidence: `wc -l crates/myosu-play/src/*.rs` on 2026-03-29.
- Observation: Artifact auto-discovery already has rich diagnostics but they're logged, not user-facing.
  Evidence: `auto_blueprint_assets_*` tests in main.rs.
- Observation: Extracting the blueprint seam clarified the real UX work left in
  stage-0 productization. The missing capability is not artifact detection
  logic; it is surfacing that logic as explicit startup/onboarding states in
  the shell and pipe outputs.
  Evidence: `crates/myosu-play/src/blueprint.rs` now owns artifact resolution
  while `main.rs` still decides how little of that state is shown to users.
- Observation: The first honest startup-state cut is smaller than the full
  interaction-state taxonomy. Startup now exposes `success` / `empty` /
  `partial`, but loading and mid-hand error states still need dedicated shell
  handling instead of transcript-only messages.
  Evidence: `pipe` now emits `STATUS startup_state=...`, while the rest of the
  interaction model is unchanged.
- Observation: The state-taxonomy work now has a real cross-crate seam. The
  right abstraction point is the shared TUI shell, not per-binary transcript
  conventions, because success/partial/error declarations matter in every game
  surface that uses the shell.
  Evidence: `myosu-tui` now owns the interaction-state declaration behavior and
  `myosu-play` only maps its startup context into it.
- Observation: Layout tiers and interaction states are coupled in practice. The
  declaration-only state model was not enough once narrow terminals entered the
  picture; transcript and state visibility had to change with terminal width or
  the UI still felt accidental.
  Evidence: narrow mode now collapses state and keeps transcript tail visible,
  while desktop mode puts transcript beside state.
- Observation: The transcript needed to become part of the state model, not
  just a passive log sink. Once the declaration banner carried interaction
  state, the transcript still looked incoherent unless it surfaced the same
  state and detail in-place.
  Evidence: `myosu-tui` now prepends status/detail lines for loading, empty,
  partial, and error states before transcript tail content.
- Observation: Mid-hand network failure handling also belonged in the shared
  status channel, not in free-form transcript strings alone. Live advice
  connectivity transitions are now structured updates, which makes the shell’s
  partial/error behavior reusable and testable instead of stringly-typed.
  Evidence: live advice refresh now emits a `Status` update plus a transcript
  message on connectivity transitions.
- Observation: Keyboard accessibility and small-width usability are also one
  problem in practice. The input line was technically functional before this
  slice, but long commands could drift off-screen and make the cursor hard to
  reason about. A viewported input line with a visible focus marker is much
  easier to use on narrow terminals.
  Evidence: `InputLine::viewport()` now drives shell input rendering and has
  direct tests.
- Observation: the last open `009` items were not hidden architecture work;
  they were concrete startup and rendering edge cases. Zero-result miner
  discovery and failed live query only needed to become first-class startup
  state, while overlong headers needed deterministic truncation instead of
  accidental wrap/clipping behavior.
  Evidence: `myosu-play` startup status now derives from full render context,
  and `myosu-tui` header rendering now clamps long labels to one line.

## Decision Log

- Decision: Prioritize state-complete behavior over new feature scope.
  Rationale: Predictable interaction is the weekly value wedge. Flaky UX undermines solver quality.
  Inversion: Adding features before state handling makes UX feel broken under real use.
  Date/Author: 2026-03-28 / Genesis

- Decision: Decompose main.rs before adding new behavior.
  Rationale: 31K lines in one file makes review and testing impractical. Decompose first, then add states.
  Date/Author: 2026-03-28 / Genesis

- Decision: Treat accessibility constraints as acceptance criteria, not polish.
  Rationale: TUI users depend on keyboard and readable textual hierarchy.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Startup path | Missing artifacts produce silent fallback | Show explicit startup status with actionable recovery text |
| Mid-hand interaction | Invalid input advances state incorrectly | Keep parse/clarify split; require explicit action acceptance |
| Small viewport layout | Panels overlap or hide action prompt | Deterministic layout tiers with min-dimension guard |
| main.rs decomposition | Refactoring breaks existing tests | Run full test suite after each extraction; atomic commits |

## Outcomes & Retrospective

Implementation is now complete. The first cut was intentionally small and
behavior-preserving: move CLI and discovery-request parsing out of `main.rs`
before layering more interaction-state behavior on top. The second cut kept the
same strategy: isolate the blueprint/artifact resolution machinery first so the
next onboarding and startup-state work can target a dedicated module instead of
mixing loading logic with pipe/train control flow. The first state-complete
behavior is now visible: startup resolution no longer hides entirely in logs and
fallback reasons, but the broader loading/error/partial interaction model is
still open. The latest cut moved one level deeper by giving the shared shell a
real interaction-state model, which means the remaining `009` work is now about
completing that taxonomy rather than inventing it. The latest responsive slice
also made one thing much clearer: the remaining work is mostly refinement and
coverage now, not architectural uncertainty. The transcript-state cut pushed
that further: first-run onboarding and operator feedback are now visibly part
of the shell contract, and the remaining work is mostly about deeper edge cases
and polish instead of finding the right architecture. The latest live-advice
status slice narrows that even more: the remaining `009` work is now mostly
about refining first-run UX copy, keyboard/accessibility polish, and a few
specific edge cases rather than inventing any new state machinery. The latest
input/onboarding slice reinforces that: the remaining `009` work is now mostly
finishing the last edge-case item and smoothing corners, not discovering
missing product surfaces. That last edge-case item is now closed too:
zero-result discovery and failed live query are explicit partial startup
states, and narrow terminals degrade long headers deliberately instead of
accidentally. The local poker surface is now honestly productized for stage-0
use.

## Context and Orientation

```text
TUI PANEL LAYOUT (5-panel flex)

+--------------------------------------------------+
| HEADER: game label | mode | source               |  fixed 1 line
+--------------------------------------------------+
| TRANSCRIPT: system messages, user inputs          |  dynamic (min 3 lines)
|                                                   |
+--------------------------------------------------+
| STATE: board, stacks, legal actions, advisor      |  dynamic (game-specific)
+--------------------------------------------------+
| DECLARATION: ALLCAPS status                       |  fixed 1 line
+--------------------------------------------------+
| > input line (cursor, completion, history)         |  fixed 1 line
+--------------------------------------------------+

RESPONSIVE TIERS:

Desktop (>=120 cols):
  Transcript and State side-by-side
  Full advisor recommendation visible

Compact (80-119 cols):
  Transcript above State (stacked)
  Advisor condensed to one line

Narrow (<80 cols):
  State panel collapsed
  Transcript tail only (last 3 lines)
  Input always visible
```

Owned files:
- `crates/myosu-play/src/main.rs` (decompose into modules)
- `crates/myosu-tui/src/shell.rs`
- `crates/myosu-tui/src/pipe.rs`
- `crates/myosu-tui/src/input.rs`
- `crates/myosu-tui/src/screens.rs`
- `crates/myosu-tui/src/theme.rs`
- `crates/myosu-games-poker/src/renderer.rs`

## Milestones

### Milestone 1: main.rs decomposition

Investigate why main.rs is 31K lines. If it's monolithic code, extract into modules: `cli.rs` (argument parsing), `blueprint.rs` (artifact discovery), `pipe_handler.rs` (pipe mode), `train_handler.rs` (interactive mode). If it's embedded data, extract to separate files.

Proof command:

    wc -l crates/myosu-play/src/*.rs | sort -n
    cargo test -p myosu-play --quiet

### Milestone 2: Interaction state taxonomy

Implement explicit state handling in shell.rs for: Loading (startup), Empty (no history), Error (bad input/missing file), Success (action accepted), Partial (advisor unavailable but manual play works).

Proof command:

    cargo test -p myosu-tui shell_state --quiet

### Milestone 3: First-run onboarding

When no artifacts are found, display actionable guidance: where to download blueprints, which env vars to set, and how to start with generated advice.

Proof command:

    cargo test -p myosu-play first_run --quiet

### Milestone 4: Edge-case input handling

Test and fix: inputs longer than terminal width, unicode characters in game labels, rapid key repeat during processing, Ctrl-C during mid-hand.

Proof command:

    cargo test -p myosu-tui input --quiet
    cargo test -p myosu-games-poker renderer --quiet

### Milestone 5: Responsive layout tiers

Implement desktop/compact/narrow layouts with explicit dimension thresholds. Add tests that verify panel visibility at each tier.

Proof command:

    cargo test -p myosu-tui layout --quiet
    cargo test -p myosu-tui shell_state_draw_too_small --quiet

### Milestone 6: Keyboard accessibility

Verify and enforce: tab completion works for all commands, focus order is logical (input always last), high-contrast theme is default, all information is text-accessible.

Proof command:

    cargo test -p myosu-tui shell_state_handle_key --quiet

## Plan of Work

1. Investigate and decompose main.rs.
2. Implement interaction state taxonomy.
3. Add first-run onboarding.
4. Handle edge-case inputs.
5. Implement responsive layout tiers.
6. Verify keyboard accessibility.

## Concrete Steps

From `/home/r/coding/myosu`:

    wc -l crates/myosu-play/src/main.rs
    cargo test -p myosu-play -p myosu-tui --quiet

## Validation and Acceptance

Accepted when:
- main.rs is decomposed into focused modules (no single file >2000 lines)
- All 5 interaction states are explicitly handled
- First-run with no artifacts shows actionable guidance
- Layout tiers are tested at 3 viewport sizes
- Keyboard-only flow works for complete hand

## Idempotence and Recovery

Decomposition commits are atomic. Each extraction followed by full test suite. If refactoring breaks tests, revert that extraction only.

## Interfaces and Dependencies

Depends on: 006 (game boundaries locked), 008 (artifact hardening provides trusted loading).
Blocks: 011 (security audit needs stable UX surface).

```text
main.rs (31K -> decomposed modules)
         |
         v
shell.rs (interaction states)
         |
         v
screens.rs (onboarding + first-run)
         |
         v
renderer.rs (responsive layout + accessibility)
```
