# Productize Play + TUI Experience

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `genesis/PLANS.md` and `genesis/DESIGN.md`.

## Purpose / Big Picture

The local play surface is already functional, but stage-0 requires a polished, resilient training + advisor experience that handles first-run, artifact-missing, and edge-input states cleanly. This plan defines and implements interaction architecture, accessibility behavior, and responsive layout intent for the TUI and pipe modes.

## Progress

- [x] (2026-03-28 21:42Z) Audited `myosu-play`, `myosu-tui`, and poker renderer flows for state coverage and layout behavior.
- [ ] Lock information architecture for first-time and returning users.
- [ ] Implement explicit loading/empty/error/partial/success states in shell and pipe surfaces.
- [ ] Add edge-case handling for long names, no results, and mid-action failures.
- [ ] Implement accessibility deliverables: keyboard-only flow, high contrast defaults, clear focus order, readable status lines.
- [ ] Implement responsive layout rules per viewport tier in shell rendering.
- [ ] Add focused tests for all interaction states and viewport layouts.

## Surprises & Discoveries

- Observation: many core interactions are already tested, but state taxonomy is implicit.
  Evidence: tests in `crates/myosu-tui/src/{shell.rs,pipe.rs,input.rs}` and `crates/myosu-play/src/main.rs`.
- Observation: artifact auto-discovery has rich diagnostics already and should be promoted into user-facing flow copy.
  Evidence: `auto_blueprint_assets_*` tests in `crates/myosu-play/src/main.rs`.

## Decision Log

- Decision: prioritize state-complete behavior over new feature scope.
  Rationale: predictable interaction is the weekly value wedge.
  Inversion (failure mode): adding commands/features before state handling will make the UX feel flaky under real use.
  Date/Author: 2026-03-28 / Genesis

- Decision: treat accessibility constraints as acceptance criteria, not polish.
  Rationale: TUI users depend on keyboard and readable textual hierarchy.
  Inversion (failure mode): if focus order and readability regress, power users abandon the surface despite solver quality.
  Date/Author: 2026-03-28 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Startup path | Missing artifacts produce silent fallback with no guidance | Show explicit startup status with actionable recovery text |
| Mid-hand interaction | Invalid input advances state incorrectly | Keep parse/clarify split and require explicit action acceptance |
| Small viewport layout | Panels overlap or hide action prompt | Add deterministic layout tiers and tests for minimum dimensions |

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

Owned files in this plan:
- `crates/myosu-play/src/main.rs`
- `crates/myosu-tui/src/shell.rs`
- `crates/myosu-tui/src/pipe.rs`
- `crates/myosu-tui/src/input.rs`
- `crates/myosu-tui/src/screens.rs`
- `crates/myosu-tui/src/theme.rs`
- `crates/myosu-games-poker/src/renderer.rs`
- `genesis/DESIGN.md`

Not owned here:
- Artifact security/versioning implementation (`008`)

## Information Architecture

Screen/order contract:
1. Startup summary: mode, artifact source, readiness state
2. Core interaction: state panel + declaration + input line
3. Feedback loop: transcript with error/clarification/success lines

ASCII IA mockups:

```text
Desktop (>=120 cols)
+---------------- Header: game | mode | source ----------------+
| Transcript (left)                | State + Advisor (right) |
| - latest system lines            | - board, street, stacks |
| - user inputs                    | - legal actions         |
| - error/help lines               | - recommendation        |
+---------------- Declaration / Context -----------------------+
| > input line (keyboard only, completion aware)              |
+--------------------------------------------------------------+

Compact (80-119 cols)
+---------------- Header ----------------+
| Transcript                            |
+---------------------------------------+
| State + Advisor                       |
+---------------------------------------+
| Declaration                           |
| > input                               |
+---------------------------------------+

Narrow (<80 cols)
+-------------- Header ---------------+
| Minimal transcript (tail only)      |
| Current action prompt               |
| > input                             |
+-------------------------------------+
```

## Interaction States

Required states:
- Loading: artifact discovery / startup resolution
- Empty: no transcript yet, no prior hand history
- Error: invalid action, missing files, decode failure
- Success: accepted action + next state rendered
- Partial: advisor unavailable but manual play available

Proof command:

    rg -n "loading|empty|error|success|partial|missing|clarify" crates/myosu-play/src/main.rs crates/myosu-tui/src/shell.rs crates/myosu-tui/src/pipe.rs crates/myosu-games-poker/src/renderer.rs

## Milestones

### Milestone 1: IA contract implementation

Encode startup -> interaction -> feedback IA in shell and play entry flow.

Proof command:

    cargo test -p myosu-tui shell_state_draw_game_screen_renders_all_panels --quiet
    cargo test -p myosu-play demo_renderer_starts_with_preflop_state --quiet

### Milestone 2: Full interaction-state handling

Implement/loading/empty/error/success/partial state rendering and transitions.

Proof command:

    cargo test -p myosu-tui pipe_output_idle_state --quiet
    cargo test -p myosu-play pipe_response_rejects_invalid_input --quiet
    cargo test -p myosu-play auto_blueprint_assets_reports_incomplete_root_when_nothing_loads --quiet

### Milestone 3: Edge-case behavior

Handle long names, zero-result advisor state, and network/artifact failure mid-action without crashing.

Proof command:

    cargo test -p myosu-tui shell_state_draw_too_small --quiet
    cargo test -p myosu-play empty_input_is_not_a_pipe_response --quiet

### Milestone 4: Accessibility and keyboard guarantees

Ensure full keyboard flow, readable contrast defaults, and deterministic focus order.

Proof command:

    cargo test -p myosu-tui shell_state_handle_key_help_toggle_only_when_input_is_empty --quiet
    cargo test -p myosu-tui ctrl_w_deletes_word --quiet

### Milestone 5: Responsive layout tiers

Implement and test explicit layout behavior for desktop/compact/narrow widths.

Proof command:

    cargo test -p myosu-tui shell_state_layout_calculates_correctly --quiet
    cargo test -p myosu-tui shell_state_layout_collapses_state_when_inactive --quiet

## Plan of Work

1. Lock IA and state taxonomy.
2. Implement robust transitions and edge handling.
3. Enforce accessibility/keyboard behavior.
4. Validate responsive layout tiers with tests.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,340p' crates/myosu-play/src/main.rs
    sed -n '1,380p' crates/myosu-tui/src/shell.rs
    cargo test -p myosu-tui -p myosu-play --quiet

## Validation and Acceptance

Accepted when:
- IA order is visible and consistent in both TUI and pipe outputs
- all five interaction states are explicitly handled
- keyboard-only flow and responsive tiers are tested

## Idempotence and Recovery

- Tests are repeatable.
- If a UX change regresses behavior, revert only that state transition and rerun the focused state tests before proceeding.

## Artifacts and Notes

- Update `outputs/play/tui/spec.md` and `outputs/play/tui/review.md`.

## Interfaces and Dependencies

Depends on: `006-game-traits-and-poker-boundaries.md`, `008-artifact-wire-checkpoint-hardening.md`
Blocks: `011-security-observability-release.md`

```text
play/main.rs startup resolution
            |
            v
tui/shell + tui/pipe state machine
            |
            v
poker renderer state/advisor projection
            |
            v
user action loop (keyboard + responsive layout)
```
